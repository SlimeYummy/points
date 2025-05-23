use std::alloc::{AllocError, Allocator, Layout};
use std::cell::Cell;
use std::collections::{BinaryHeap, VecDeque};
use std::ptr::NonNull;
use std::{alloc, ptr};

const PRE_ALLOCATION_ALIGN: usize = 16;

#[derive(Debug)]
pub struct PreAllocator {
    buffer: Cell<*mut u8>,
    size: Cell<usize>,
    using: Cell<bool>,
}

unsafe impl Allocator for &'_ mut PreAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if self.using.get() {
            return Err(AllocError);
        }
        if layout.align() > PRE_ALLOCATION_ALIGN {
            return Err(AllocError);
        }
        if layout.size() > self.size.get() {
            unsafe {
                alloc::dealloc(
                    self.buffer.get(),
                    Layout::from_size_align_unchecked(self.size.get(), PRE_ALLOCATION_ALIGN),
                );
                self.buffer.set(alloc::alloc(layout));
            }
        }
        self.using.set(true);
        Ok(unsafe { NonNull::slice_from_raw_parts(NonNull::new_unchecked(self.buffer.get()), self.size.get()) })
    }

    unsafe fn grow(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if new_layout.align() > PRE_ALLOCATION_ALIGN {
            return Err(AllocError);
        }
        if new_layout.size() > self.size.get() {
            unsafe {
                alloc::dealloc(
                    self.buffer.get(),
                    Layout::from_size_align_unchecked(self.size.get(), PRE_ALLOCATION_ALIGN),
                );
                self.buffer.set(alloc::alloc(new_layout));
            }
        }
        Ok(unsafe { NonNull::slice_from_raw_parts(NonNull::new_unchecked(self.buffer.get()), self.size.get()) })
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let new_ptr = self.grow(ptr, old_layout, new_layout)?;
        ptr::write_bytes(
            self.buffer.get().add(old_layout.size()),
            0,
            new_layout.size() - old_layout.size(),
        );
        Ok(new_ptr)
    }

    unsafe fn shrink(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        Ok(unsafe { NonNull::slice_from_raw_parts(NonNull::new_unchecked(self.buffer.get()), self.size.get()) })
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        self.using.set(false);
    }
}

impl PreAllocator {
    pub fn new(size: usize) -> Self {
        let buffer = unsafe { alloc::alloc(Layout::from_size_align_unchecked(size, PRE_ALLOCATION_ALIGN)) };
        Self {
            buffer: Cell::new(buffer),
            size: Cell::new(size),
            using: Cell::new(false),
        }
    }

    pub fn vec<T>(&mut self) -> Vec<T, &mut PreAllocator> {
        Vec::new_in(self)
    }

    pub fn vec_with_capacity<T>(&mut self, capacity: usize) -> Vec<T, &mut PreAllocator> {
        Vec::with_capacity_in(capacity, self)
    }

    pub fn vec_deque<T>(&mut self) -> VecDeque<T, &mut PreAllocator> {
        VecDeque::new_in(self)
    }

    pub fn vec_deque_with_capacity<T>(&mut self, capacity: usize) -> VecDeque<T, &mut PreAllocator> {
        VecDeque::with_capacity_in(capacity, self)
    }

    pub fn binary_heap<T: Ord>(&mut self) -> BinaryHeap<T, &mut PreAllocator> {
        BinaryHeap::new_in(self)
    }

    pub fn binary_heap_with_capacity<T: Ord>(&mut self, capacity: usize) -> BinaryHeap<T, &mut PreAllocator> {
        BinaryHeap::with_capacity_in(capacity, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_pre_allocator() {
        let mut allocator = PreAllocator::new(0);

        let mut vec = allocator.vec::<i32>();
        for i in 0..32 {
            vec.push(i);
        }
        mem::drop(vec);

        let mut vec_deque = allocator.vec_deque::<i64>();
        vec_deque.reserve(4);
        mem::drop(vec_deque);

        let mut binary_heap = allocator.binary_heap_with_capacity::<u64>(20);
        for i in 0..32 {
            binary_heap.push(i);
        }
        mem::drop(binary_heap);
    }
}
