use core::alloc::Layout;
use mmap_rs::{MmapMut, MmapOptions, ReservedMut};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use talc::TalcCell;
use talc::base::Talc;
use talc::base::binning::Binning;
use talc::source::Source;
use wasmtime::{LinearMemory, MemoryCreator, MemoryType};

use crate::consts::KB;
use crate::utils::{XResult, xerr, xres};

#[derive(Debug)]
struct VirtualMemory {
    uncommitted: Option<ReservedMut>,
    committed: Option<MmapMut>,
    max_size: usize,
    committed_size: usize,
    grow_size: usize,
    start_ptr: *mut u8,
}

unsafe impl Send for VirtualMemory {}
unsafe impl Sync for VirtualMemory {}

impl VirtualMemory {
    const PAGE_SIZE: usize = 64 * KB;

    fn new(uncommitted: ReservedMut, grow_size: usize) -> XResult<VirtualMemory> {
        let max_size = uncommitted.size();
        if grow_size == 0 || grow_size > max_size || grow_size % Self::PAGE_SIZE != 0 {
            return xres!(BadArgument; "grow size");
        }
        let start_ptr = uncommitted.start() as *mut u8;
        Ok(VirtualMemory {
            committed: None,
            uncommitted: Some(uncommitted),
            max_size,
            committed_size: 0,
            grow_size,
            start_ptr,
        })
    }

    fn commit(&mut self, size: usize) -> XResult<()> {
        if size == 0 {
            return Ok(());
        }

        let aligned_size = (size + self.grow_size - 1) & !(self.grow_size - 1);
        if aligned_size + self.committed_size > self.max_size {
            return xres!(OutOfMemory; "out of memory");
        }

        // Split
        let mut uncommitted = self.uncommitted.take().ok_or_else(|| xerr!(OutOfMemory; "none"))?;
        let to_commit = uncommitted
            .split_to(aligned_size)
            .map_err(|_| xerr!(OutOfMemory; "ReservedMut::split_to"))?;
        self.uncommitted = if uncommitted.size() > 0 {
            Some(uncommitted)
        }
        else {
            None
        };

        // Commit
        let new_committed = MmapMut::try_from(to_commit).map_err(|_| xerr!(OutOfMemory; "MmapMut::try_from"))?;

        // Merge
        if let Some(mut committed) = self.committed.take() {
            committed
                .merge(new_committed)
                .map_err(|_| xerr!(OutOfMemory; "MmapMut::merge"))?;
            self.committed = Some(committed);
        }
        else {
            self.committed = Some(new_committed);
        }

        self.committed_size += aligned_size;
        Ok(())
    }

    fn commit_to(&mut self, size: usize) -> XResult<()> {
        if size < self.committed_size {
            return Ok(());
        }
        self.commit(size - self.committed_size)
    }

    #[inline]
    fn committed_size(&self) -> usize {
        self.committed_size
    }

    #[inline]
    fn max_size(&self) -> usize {
        self.max_size
    }

    #[inline]
    fn grow_size(&self) -> usize {
        self.grow_size
    }

    #[inline]
    fn start_ptr(&self) -> *mut u8 {
        self.start_ptr
    }

    #[inline]
    fn end_ptr(&self) -> *mut u8 {
        unsafe { self.start_ptr.add(self.committed_size) }
    }
}

#[derive(Debug)]
pub struct TalcSource {
    arena: VirtualMemory,
}

impl TalcSource {
    fn new(arena: VirtualMemory) -> TalcSource {
        TalcSource { arena }
    }
}

unsafe impl Source for TalcSource {
    fn acquire<B: Binning>(talc: &mut Talc<Self, B>, layout: Layout) -> Result<(), ()> {
        debug_assert!(VirtualMemory::PAGE_SIZE % layout.align() == 0);

        let old_end = talc.source.arena.end_ptr();
        if let Err(err) = talc.source.arena.commit(layout.size()) {
            log::error!("TalcSource::acquire() commit failed: {}", err);
            return Err(());
        }
        let new_end = talc.source.arena.end_ptr();

        if old_end != new_end {
            unsafe { talc.extend(NonNull::new_unchecked(old_end), new_end) };
        }
        Ok(())
    }
}

pub(crate) struct WasmMemoryCreator {
    memory: Mutex<Option<Box<WasmLinearMemory>>>,
}

impl WasmMemoryCreator {
    fn new(stack_memory: MmapMut, arena: VirtualMemory, base_size: usize, base_ptr: *mut u8) -> WasmMemoryCreator {
        WasmMemoryCreator {
            memory: Mutex::new(Some(Box::new(WasmLinearMemory {
                _stack_memory: stack_memory,
                arena,
                base_size,
                base_ptr,
            }))),
        }
    }
}

unsafe impl Send for WasmMemoryCreator {}
unsafe impl Sync for WasmMemoryCreator {}

unsafe impl MemoryCreator for WasmMemoryCreator {
    fn new_memory(
        &self,
        _ty: MemoryType,
        minimum: usize,
        maximum: Option<usize>,
        _reserved_size_in_bytes: Option<usize>,
        _guard_size_in_bytes: usize,
    ) -> Result<Box<dyn LinearMemory>, String> {
        // WasmMemoryCreator is specially designed, we will ensure that LinearMemory is created only once.
        let mut memory = self
            .memory
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| "No arena available".to_string())?;
        if let Some(maximum) = maximum {
            if memory.byte_capacity() < maximum {
                return Err("Insufficient memory capacity".to_string());
            }
        }
        memory.grow_to(minimum).map_err(|e| e.to_string())?;
        Ok(memory)
    }
}

struct WasmLinearMemory {
    _stack_memory: MmapMut,
    arena: VirtualMemory,
    base_size: usize,
    base_ptr: *mut u8,
}

unsafe impl Send for WasmLinearMemory {}
unsafe impl Sync for WasmLinearMemory {}

unsafe impl LinearMemory for WasmLinearMemory {
    fn byte_size(&self) -> usize {
        self.base_size + self.arena.committed_size()
    }

    fn byte_capacity(&self) -> usize {
        self.base_size + self.arena.max_size()
    }

    fn grow_to(&mut self, new_size: usize) -> Result<(), wasmtime::Error> {
        debug_assert!(new_size >= self.base_size);

        if let Err(err) = self.arena.commit_to(new_size - self.base_size) {
            log::error!("WasmLinearMemory::grow_to() commit failed: {}", err);
            return Err(wasmtime::Error::msg("out of memory"));
        }
        Ok(())
    }

    fn as_ptr(&self) -> *mut u8 {
        self.base_ptr
    }
}

/// Memory structure:
/// - [0 .. stack_size) => stack
/// - [stack_size .. stack_size+host_size) => host
/// - [stack_size+host_size .. max_size) => wasm
pub(crate) fn new_allocators(
    max_size: usize,
    stack_size: usize,
    host_size: usize,
    host_grow_size: usize,
) -> XResult<(TalcCell<TalcSource>, WasmMemoryCreator, usize)> {
    if stack_size == 0 || stack_size % VirtualMemory::PAGE_SIZE != 0 {
        return xres!(BadArgument; "stack_size");
    }
    if host_size == 0 || host_size % VirtualMemory::PAGE_SIZE != 0 {
        return xres!(BadArgument; "host_size");
    }
    if max_size <= stack_size + host_size || max_size % VirtualMemory::PAGE_SIZE != 0 {
        return xres!(BadArgument; "max_size");
    }
    if host_grow_size == 0 || host_grow_size % VirtualMemory::PAGE_SIZE != 0 {
        return xres!(BadArgument; "host_grow_size");
    }

    // Reserve entire memory region
    let mut reserved = MmapOptions::new(max_size)
        .map_err(|_| xerr!(OutOfMemory; "MmapOptions::new"))?
        .reserve_mut()
        .map_err(|_| xerr!(OutOfMemory; "MmapOptions::reserve_mut"))?;
    let base_ptr = reserved.start() as *mut u8;

    // Split: stack/host/wasm
    let stack_memory = reserved
        .split_to(stack_size)
        .map_err(|_| xerr!(OutOfMemory; "ReservedMut::split_to"))?;
    let host_memory = reserved
        .split_to(host_size)
        .map_err(|_| xerr!(OutOfMemory; "ReservedMut::split_to"))?;
    let wasm_memory = reserved;

    // stack
    let stack_memory = MmapMut::try_from(stack_memory).map_err(|_| xerr!(OutOfMemory; "MmapMut::try_from"))?;

    // wasm
    let wasm_arena = VirtualMemory::new(wasm_memory, host_grow_size)?;
    let wasm_creator = WasmMemoryCreator::new(stack_memory, wasm_arena, stack_size + host_size, base_ptr);

    // host
    let mut host_arena = VirtualMemory::new(host_memory, VirtualMemory::PAGE_SIZE)?;
    let start_ptr = host_arena.start_ptr();
    let start_size = host_arena.grow_size();
    host_arena.commit(start_size)?;
    let source = TalcSource::new(host_arena);
    let talc = TalcCell::new(source);
    unsafe { talc.claim(start_ptr, start_size) };

    Ok((talc, wasm_creator, base_ptr as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::MB;

    #[test]
    fn test_memory_arena_commit() {
        let reserved = mmap_rs::MmapOptions::new(1 * MB).unwrap().reserve_mut().unwrap();
        let mut arena = VirtualMemory::new(reserved, 64 * KB).unwrap();

        assert_eq!(arena.committed_size(), 0);
        assert_eq!(arena.end_ptr(), arena.start_ptr());

        arena.commit(64 * KB).unwrap();
        assert_eq!(arena.committed_size(), 64 * KB);
        assert_eq!(arena.end_ptr(), unsafe { arena.start_ptr().add(64 * KB) });

        arena.commit(1).unwrap();
        assert_eq!(arena.committed_size(), 128 * KB);
        assert_eq!(arena.end_ptr(), unsafe { arena.start_ptr().add(128 * KB) });

        arena.commit(0).unwrap();
        assert_eq!(arena.committed_size(), 128 * KB);
        assert_eq!(arena.end_ptr(), unsafe { arena.start_ptr().add(128 * KB) });

        arena.commit_to(128 * KB).unwrap();
        assert_eq!(arena.committed_size(), 128 * KB);
        assert_eq!(arena.end_ptr(), unsafe { arena.start_ptr().add(128 * KB) });

        arena.commit_to(200 * KB).unwrap();
        assert!(arena.committed_size() == 256 * KB);
        assert_eq!(arena.end_ptr(), unsafe { arena.start_ptr().add(256 * KB) });

        arena.commit_to(100 * KB).unwrap();
        assert!(arena.committed_size() == 256 * KB);
        assert_eq!(arena.end_ptr(), unsafe { arena.start_ptr().add(256 * KB) });

        assert!(arena.commit_to(1 * MB + 1).is_err());
        assert_eq!(arena.end_ptr(), unsafe { arena.start_ptr().add(256 * KB) });
    }

    #[test]
    fn test_virtual_memory_commit() {
        let reserved = mmap_rs::MmapOptions::new(256 * KB).unwrap().reserve_mut().unwrap();
        let mut arena = VirtualMemory::new(reserved, 64 * KB).unwrap();

        // Before commit: writing would segfault (we don't test this)

        // After commit: should be writable
        arena.commit(64 * KB).unwrap();
        assert_eq!(arena.committed_size(), 64 * KB);
        let ptr = arena.start_ptr();
        unsafe {
            std::ptr::write_volatile(ptr, 0xDE);
            std::ptr::write_volatile(ptr.add(64 * KB - 1), 0xAD);
            assert_eq!(std::ptr::read_volatile(ptr), 0xDE);
            assert_eq!(std::ptr::read_volatile(ptr.add(64 * KB - 1)), 0xAD);
        }

        // Grow more and verify continuity
        arena.commit(64 * KB).unwrap();
        assert_eq!(arena.committed_size(), 128 * KB);
        unsafe {
            assert_eq!(std::ptr::read_volatile(ptr), 0xDE); // Read old
            std::ptr::write_volatile(ptr.add(64 * KB), 0x11);
            std::ptr::write_volatile(ptr.add(128 * KB - 1), 0x22);
        }
    }

    #[test]
    fn test_new_allocators_arguments() {
        // stack_size = 0
        assert!(new_allocators(4 * MB, 0, 1 * MB, 64 * KB).is_err());

        // host_size = 0
        assert!(new_allocators(4 * MB, 128 * KB, 0, 64 * KB).is_err());

        // max_size too small
        assert!(new_allocators(128 * KB, 64 * KB, 64 * KB, 64 * KB).is_err());

        // wasm_size = 0 (max_size == stack + host)
        assert!(new_allocators(2 * MB, 1 * MB, 1 * MB, 64 * KB).is_err());

        // stack_size not aligned to PAGE_SIZE
        assert!(new_allocators(4 * MB, 100 * KB, 1 * MB, 64 * KB).is_err());

        // host_size not aligned to PAGE_SIZE
        assert!(new_allocators(4 * MB, 128 * KB, 100 * KB, 64 * KB).is_err());

        // max_size not aligned to PAGE_SIZE
        assert!(new_allocators(4 * MB + 1, 128 * KB, 1 * MB, 64 * KB).is_err());

        // grow_step = 0
        assert!(new_allocators(4 * MB, 128 * KB, 1 * MB, 0).is_err());

        // grow_step not aligned to PAGE_SIZE
        assert!(new_allocators(4 * MB, 128 * KB, 1 * MB, 100 * KB).is_err());
    }

    #[test]
    fn test_new_allocators_returns() {
        let (mut talc, wasm_creator, _) = new_allocators(4 * MB, 128 * KB, 1 * MB, 64 * KB).unwrap();

        // host

        assert_eq!(talc.get_mut().source.arena.committed_size(), 64 * KB);
        assert_eq!(talc.get_mut().source.arena.max_size(), 1 * MB);

        let v1 = Vec::<u32, _>::with_capacity_in(32, &talc);
        drop(v1);
        assert_eq!(talc.get_mut().source.arena.committed_size(), 64 * KB);

        let v2 = Vec::<i8, _>::with_capacity_in(64 * KB, &talc);
        drop(v2);
        assert_eq!(talc.get_mut().source.arena.committed_size(), 64 * KB * 2);

        // wasm

        let wasm_lock = wasm_creator.memory.lock().unwrap();
        let wasm_mem = wasm_lock.as_ref().unwrap();
        assert_eq!(wasm_mem.arena.committed_size(), 0);
        assert_eq!(wasm_mem.arena.max_size(), 4 * MB - 128 * KB - 1 * MB);

        let stack_start = wasm_mem._stack_memory.start() as *mut u8;
        let wasm_start = wasm_mem.arena.start_ptr();
        assert_eq!(wasm_start, unsafe { stack_start.add(128 * KB + 1 * MB) });
    }

    #[test]
    fn test_memory() {
        use std::sync::Arc;
        use wasmtime::{Config, Engine, Instance, Module, Store};

        let max_size = 4 * MB;
        let stack_size = 128 * KB;
        let host_size = 1 * MB;
        let grow_step = 64 * KB;

        let (talc, wasm_creator, base_ptr) = new_allocators(max_size, stack_size, host_size, grow_step).unwrap();

        let mut config = Config::new();
        config.with_host_memory(Arc::new(wasm_creator));
        let engine = Engine::new(&config).unwrap();
        let mut store = Store::new(&engine, ());

        let wasm_start = (stack_size + host_size) as u32;
        let wat = format!(
            r#"(module
                (memory (export "memory") 19 64)

                ;; data segment
                (data (i32.const {}) "Hello")
                
                (func (export "test_stack") (result i32 i32)
                    (i32.store (i32.const 16) (i32.const 42))
                    (i32.load (i32.const 16))
                    (i32.const 16)
                )

                (func (export "get_static_addr") (result i32)
                    (i32.const {})
                )

                (func (export "trigger_grow") (param $pages i32) (result i32)
                    (local $old_pages i32)
                    (local $new_addr i32)
                    (local.set $old_pages (memory.grow (local.get $pages)))
                    (local.set $new_addr (i32.mul (local.get $old_pages) (i32.const 65536)))
                    (i32.store (local.get $new_addr) (i32.const 99))
                    (i32.load (local.get $new_addr))
                )

                (func (export "read_host") (param $addr i32) (result i32)
                    (i32.load (local.get $addr))
                )
            )"#,
            wasm_start, wasm_start
        );

        let module = Module::new(&engine, &wat).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let test_stack = instance
            .get_typed_func::<(), (i32, i32)>(&mut store, "test_stack")
            .unwrap();
        let (val, stack_addr) = test_stack.call(&mut store, ()).unwrap();
        assert_eq!(val, 42);
        assert_eq!(stack_addr as usize, 16);
        assert_eq!(
            unsafe { *((base_ptr as *const u8).add(stack_addr as usize) as *const i32) },
            42
        );

        let get_static_addr = instance
            .get_typed_func::<(), i32>(&mut store, "get_static_addr")
            .unwrap();
        let static_addr = get_static_addr.call(&mut store, ()).unwrap();
        assert_eq!(static_addr, wasm_start as i32);

        let trigger_grow = instance.get_typed_func::<i32, i32>(&mut store, "trigger_grow").unwrap();
        let grow_val = trigger_grow.call(&mut store, 1).unwrap();
        assert_eq!(grow_val, 99);

        let talc = Rc::new(talc);
        let mut v = Vec::<i32, _>::with_capacity_in(10, talc.clone());
        v.push(2026);
        let v_ptr = v.as_ptr() as usize;
        let wasm_addr = (v_ptr - base_ptr) as u32;
        let read_host = instance.get_typed_func::<i32, i32>(&mut store, "read_host").unwrap();
        let host_val = read_host.call(&mut store, wasm_addr as i32).unwrap();
        assert_eq!(host_val, 2026);
    }
}
