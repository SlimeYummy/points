use std::collections::hash_map::Entry;
use std::{fmt, mem, slice};

use crate::script::command::*;
use crate::script::config::{
    MAX_FUNCTION_ARGUMENTS, MAX_OFFSET, MAX_REGISTER, MUT_SEGMENT_COUNT, NUM_SEGMENT_COUNT,
    SEGMENT_CLOSURE, SEGMENT_CONSTANT, SEGMENT_IN_MAX, SEGMENT_IN_MIN, SEGMENT_NAMES,
    SEGMENT_OUT_MAX, SEGMENT_OUT_MIN, SEGMENT_REGISTER, SEGMENT_STRING,
};
use crate::utils::{Num, Symbol, SymbolMap, XError, XResult};

pub struct ScriptRunner {
    pc: usize,
    num_segs: NumberSegments,
    str_seg: StringSegment,
    registers: [Num; MAX_REGISTER],
    ext_args: [CmdAddr; MAX_FUNCTION_ARGUMENTS],
}

impl ScriptRunner {
    pub fn new() -> Box<ScriptRunner> {
        let mut runner = Box::new(ScriptRunner {
            pc: 0,
            num_segs: NumberSegments::default(),
            str_seg: StringSegment::default(),
            registers: [Num::default(); MAX_REGISTER],
            ext_args: [CmdAddr::default(); MAX_FUNCTION_ARGUMENTS],
        });
        runner.num_segs.set_segment(SEGMENT_REGISTER, unsafe {
            slice::from_raw_parts_mut(runner.registers.as_mut_ptr() as *mut u64, MAX_REGISTER)
        });
        return runner;
    }

    pub fn run_hook<const I: usize, const O: usize, C: ScriptEnv<I, O>>(
        &mut self,
        blocks: &ScriptBlocks,
        hook_typ: ScriptBlockType,
        context: &mut C,
    ) -> XResult<()> {
        let hook_block = match blocks.hook(hook_typ) {
            Some(block) => block,
            None => return Err(XError::ScriptNoHook),
        };

        self.num_segs
            .set_segment(SEGMENT_CLOSURE, context.closure_segment());
        self.num_segs
            .set_segment(SEGMENT_CONSTANT, blocks.constant_segment());
        self.str_seg.set_segment(blocks.string_segment());

        let out_segments = context.out_segments();
        for idx in 0..usize::min(O, SEGMENT_OUT_MAX as usize) {
            self.num_segs
                .set_segment(SEGMENT_OUT_MIN + idx as u8, out_segments[idx]);
        }

        let in_segments = context.in_segments();
        for idx in 0..usize::min(I, SEGMENT_IN_MAX as usize) {
            self.num_segs
                .set_segment(SEGMENT_IN_MIN + idx as u8, in_segments[idx]);
        }

        return self.execute_loop(&hook_block.code, context);
    }

    pub fn execute_loop<const I: usize, const O: usize, C: ScriptEnv<I, O>>(
        &mut self,
        code: &ScriptByteCode,
        context: &mut C,
    ) -> XResult<()> {
        use CmdOpt::*;

        self.pc = 0;
        while self.pc < code.len() {
            let opt = code.peek_opt(self.pc);
            match opt {
                Jmp => self.jmp(code),
                JmpCmp => self.jmp_cmp(code),
                JmpSet => self.jmp_set(code),
                JmpCas0 | JmpCas1 => self.jmp_cas(code, opt == JmpCas1),
                Mov => self.call(code, |[x]| x),
                Neg => self.call(code, |[x]| -x),
                Not => self.call(code, |[x]| (x == 0.0) as u32 as Num),
                Add => self.call(code, |[x, y]| x + y),
                Sub => self.call(code, |[x, y]| x - y),
                Mul => self.call(code, |[x, y]| x * y),
                Pow => self.call(code, |[x, y]| x.powf(y)),
                Div => self.call(code, |[x, y]| x / y),
                Mod => self.call(code, |[x, y]| x % y),
                Le => self.call(code, |[x, y]| (x <= y) as u32 as Num),
                Lt => self.call(code, |[x, y]| (x < y) as u32 as Num),
                Ge => self.call(code, |[x, y]| (x >= y) as u32 as Num),
                Gt => self.call(code, |[x, y]| (x > y) as u32 as Num),
                Eq => self.call(code, |[x, y]| (x == y) as u32 as Num),
                Ne => self.call(code, |[x, y]| (x != y) as u32 as Num),
                IfElse0 | IfElse1 => self.call(code, |[c, x, y]| Self::if_else(opt, c, x, y)),
                XInit => self.call_s(code, |k, [v]| Self::x_init(context.global(), k, v)),
                XSet => self.call_s(code, |k, [v]| Self::x_set(context.global(), k, v)),
                XGet => self.call_s(code, |k, []| Self::x_get(context.global(), k)),
                XHas => self.call_s(code, |k, []| Self::x_has(context.global(), k)),
                XDel => self.call_s(code, |k, []| Self::x_del(context.global(), k)),
                IsNan => self.call(code, |[x]| x.is_nan() as u32 as Num),
                IsInf => self.call(code, |[x]| x.is_infinite() as u32 as Num),
                Abs => self.call(code, |[x]| x.abs()),
                Min => self.call(code, |[x, y]| Num::min(x, y)),
                Max => self.call(code, |[x, y]| Num::max(x, y)),
                Floor => self.call(code, |[x]| x.floor()),
                Ceil => self.call(code, |[x]| x.ceil()),
                Round => self.call(code, |[x]| x.round()),
                Clamp => self.call(code, |[x, min, max]| Num::clamp(x, min, max)),
                Saturate => self.call(code, |[x]| Num::clamp(x, 0 as Num, 1 as Num)),
                Lerp => self.call(code, |[src, dst, step]| src + step * (dst - src)),
                Sqrt => self.call(code, |[x]| x.sqrt()),
                Exp => self.call(code, |[x]| x.exp()),
                Degrees => self.call(code, |[x]| 180.0 / core::f64::consts::PI * x),
                Radians => self.call(code, |[x]| core::f64::consts::PI / 180.0 * x),
                Sin => self.call(code, |[x]| x.sin()),
                Cos => self.call(code, |[x]| x.cos()),
                Tan => self.call(code, |[x]| x.tan()),
                Ext0 => self.call_ext::<0, I, O, _>(code, context)?,
                Ext1 => self.call_ext::<1, I, O, _>(code, context)?,
                Ext2 => self.call_ext::<2, I, O, _>(code, context)?,
                Ext3 => self.call_ext::<3, I, O, _>(code, context)?,
                Ext4 => self.call_ext::<4, I, O, _>(code, context)?,
                Ext5 => self.call_ext::<5, I, O, _>(code, context)?,
                Ext6 => self.call_ext::<6, I, O, _>(code, context)?,
                Ext7 => self.call_ext::<7, I, O, _>(code, context)?,
                Ext8 => self.call_ext::<8, I, O, _>(code, context)?,
                _ => return Err(XError::ScriptBadCommand),
            };
        }
        return Ok(());
    }

    #[inline(always)]
    fn jmp(&mut self, code: &ScriptByteCode) {
        let cmd = code.jmp(self.pc);
        self.pc = self.read_pc(&cmd.pc);
        // println!("{:?}: >> {:?}", cmd.opt, self.pc);
    }

    #[inline(always)]
    fn jmp_cmp(&mut self, code: &ScriptByteCode) {
        let cmd = code.jmp_cmp(self.pc);
        let cond = self.read_num(&cmd.cond);
        if cond == 0.0 {
            self.pc = self.read_pc(&cmd.pc);
            // println!("{:?}: {:?} >> {:?}", cmd.opt, cond, self.pc);
        } else {
            self.pc += cmd.len();
            // println!("{:?}: {:?}", cmd.opt, cond);
        }
    }

    #[inline(always)]
    fn jmp_set(&mut self, code: &ScriptByteCode) {
        let cmd = code.jmp_set(self.pc);
        let val = self.read_num(&cmd.src);
        self.write_num(&cmd.dst, val);
        self.pc = self.read_pc(&cmd.pc);
        // println!("{:?}: => {:?} >> {:?}", cmd.opt, val, self.pc);
    }

    #[inline(always)]
    fn jmp_cas(&mut self, code: &ScriptByteCode, cmd_cond: bool) {
        let cmd = code.jmp_cas(self.pc);
        let cond = self.read_num(&cmd.cond);
        if (cond != 0.0) == cmd_cond {
            self.pc = self.read_pc(&cmd.pc);
            let val = self.read_num(&cmd.src);
            self.write_num(&cmd.dst, val.into());
            // println!("{:?}: {:?} => {:?} >> {:?}", cmd.opt, cond, val, self.pc);
        } else {
            self.pc += cmd.len();
            // println!("{:?}: {:?}", cmd.opt, cond);
        }
    }

    #[inline(always)]
    fn call<F, const N: usize>(&mut self, code: &ScriptByteCode, lambda: F)
    where
        F: FnOnce([Num; N]) -> Num,
    {
        let cmd = code.call_pc::<N>(&mut self.pc);
        let mut src = [Num::default(); N];
        for idx in 0..N {
            src[idx] = self.read_num(&cmd.src[idx]);
        }
        let dst = lambda(src);
        self.write_num(&cmd.dst, dst);
        // println!("{:?}: {:?} => {:?}", cmd.opt, src, dst);
    }

    #[inline(always)]
    fn call_s<F, const N: usize>(&mut self, code: &ScriptByteCode, lambda: F)
    where
        F: FnOnce(Symbol, [Num; N]) -> Num,
    {
        let cmd = code.call_pc::<N>(&mut self.pc);
        let str = self.read_str(&cmd.src[0]);
        let mut src = [Num::default(); N];
        for idx in 1..N {
            src[idx] = self.read_num(&cmd.src[idx]);
        }
        let dst = lambda(str, src);
        if cmd.dst.offset() < MAX_OFFSET as u16 {
            self.write_num(&cmd.dst, dst);
        }
        // println!("{:?}: {:?} => {:?}", cmd.opt, src, dst);
    }

    #[inline(always)]
    fn call_ext<const N: usize, const I: usize, const O: usize, C: ScriptEnv<I, O>>(
        &mut self,
        code: &ScriptByteCode,
        ctx: &mut C,
    ) -> XResult<()> {
        let cmd = code.call_ext_pc::<N>(&mut self.pc);
        let ext = cmd.ext;
        let dst = cmd.dst;

        let ce = ScriptCallExt {
            num_segs: &self.num_segs,
            str_seg: &self.str_seg,
        };
        for idx in 0..N {
            self.ext_args[idx] = cmd.src[idx];
        }
        let ret = ctx.call_ext(ce, ext, &self.ext_args[0..N])?;
        for idx in 0..N {
            self.ext_args[idx] = CmdAddr::default();
        }

        self.write_num(&dst, ret);
        // println!("{:?}.{}: {:?} => {:?}", cmd.opt, ext, cmd.src, cmd.dst);
        return Ok(());
    }

    #[inline(always)]
    fn read_pc(&self, addr: &CmdAddr) -> usize {
        let raw = self.num_segs.get(addr.segment(), addr.offset());
        return raw as usize;
    }

    #[inline(always)]
    fn read_num(&self, addr: &CmdAddr) -> Num {
        let raw = self.num_segs.get(addr.segment(), addr.offset());
        return unsafe { mem::transmute(raw) };
    }

    #[inline(always)]
    fn read_str(&self, addr: &CmdAddr) -> Symbol {
        return self.str_seg.get(addr.segment(), addr.offset());
    }

    #[inline(always)]
    fn write_num(&mut self, addr: &CmdAddr, val: Num) {
        let val = unsafe { mem::transmute(val) };
        self.num_segs.set(addr.segment(), addr.offset(), val);
    }

    ///// inner functions /////

    #[inline(always)]
    fn if_else(opt: CmdOpt, c: Num, x: Num, y: Num) -> Num {
        if (c != 0.0) == (opt == CmdOpt::IfElse1) {
            return x;
        } else {
            return y;
        }
    }

    #[inline(always)]
    fn x_init(global: &mut SymbolMap<Num>, k: Symbol, v: Num) -> Num {
        match global.entry(k) {
            Entry::Occupied(_) => return 0.0,
            Entry::Vacant(e) => {
                e.insert(v);
                return 1.0;
            }
        };
    }

    #[inline(always)]
    fn x_set(global: &mut SymbolMap<Num>, k: Symbol, v: Num) -> Num {
        return match global.insert(k, v) {
            Some(o) => o,
            None => 0.0,
        };
    }

    #[inline(always)]
    fn x_get(global: &mut SymbolMap<Num>, k: Symbol) -> Num {
        return match global.get(&k) {
            Some(&v) => v,
            None => 0.0,
        };
    }

    #[inline(always)]
    fn x_has(global: &mut SymbolMap<Num>, k: Symbol) -> Num {
        return match global.contains_key(&k) {
            true => 1.0,
            false => 0.0,
        };
    }

    #[inline(always)]
    fn x_del(global: &mut SymbolMap<Num>, k: Symbol) -> Num {
        return match global.remove(&k) {
            Some(_) => 1.0,
            None => 0.0,
        };
    }
}

pub trait ScriptEnv<const I: usize, const O: usize> {
    fn closure_segment(&mut self) -> &mut [u64];
    fn in_segments(&self) -> [&[u64]; I];
    fn out_segments(&mut self) -> [&mut [u64]; O];
    fn global(&mut self) -> &mut SymbolMap<Num>;
    fn call_ext<'t>(
        &'t mut self,
        _ce: ScriptCallExt<'t>,
        _opt: u16,
        _args: &[CmdAddr],
    ) -> XResult<Num> {
        return Err(XError::ScriptBadCommand);
    }
}

pub struct ScriptCallExt<'t> {
    num_segs: &'t NumberSegments,
    str_seg: &'t StringSegment,
}

impl<'t> ScriptCallExt<'t> {
    pub fn read_num(&self, addr: &CmdAddr) -> Num {
        let raw = self.num_segs.get(addr.segment(), addr.offset());
        return unsafe { mem::transmute(raw) };
    }

    pub fn read_str(&self, addr: &CmdAddr) -> Symbol {
        return self.str_seg.get(addr.segment(), addr.offset());
    }
}

union NumberSegments {
    num_segs: [&'static [u64]; NUM_SEGMENT_COUNT],
    mut_segs: [&'static mut [u64]; MUT_SEGMENT_COUNT],
}

impl Default for NumberSegments {
    fn default() -> Self {
        return NumberSegments {
            num_segs: [&[]; NUM_SEGMENT_COUNT],
        };
    }
}

impl NumberSegments {
    #[inline(always)]
    fn set_segment(&mut self, idx: u8, segment: &[u64]) {
        unsafe { self.num_segs[idx as usize] = mem::transmute(segment) };
    }

    #[inline(always)]
    fn get(&self, seg: u8, off: u16) -> u64 {
        return unsafe { self.num_segs }[seg as usize][off as usize];
    }

    #[inline(always)]
    fn set(&mut self, idx: u8, off: u16, val: u64) {
        unsafe { self.mut_segs[idx as usize][off as usize] = val };
    }
}

impl fmt::Debug for NumberSegments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_segs = unsafe { &self.num_segs };
        let mut ds = f.debug_struct("NumberSegments");
        for (idx, name) in SEGMENT_NAMES[0..NUM_SEGMENT_COUNT].iter().enumerate() {
            ds.field(name, &num_segs[idx].len());
        }
        return ds.finish();
    }
}

struct StringSegment {
    str_seg: &'static [Symbol],
}

impl Default for StringSegment {
    fn default() -> Self {
        return StringSegment { str_seg: &[] };
    }
}

impl StringSegment {
    #[inline(always)]
    fn set_segment(&mut self, segment: &[Symbol]) {
        unsafe { self.str_seg = mem::transmute(segment) };
    }

    #[inline(always)]
    fn get(&self, seg: u8, off: u16) -> Symbol {
        if seg != SEGMENT_STRING {
            panic!("Not string segment: {}", seg);
        }
        return self.str_seg[off as usize].clone();
    }
}

impl fmt::Debug for StringSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ds = f.debug_struct("StringSegment");
        ds.field("string", &self.str_seg.len());
        return ds.finish();
    }
}

#[cfg(test)]
mod tests {
    use ScriptBlockType::*;

    use super::*;
    use crate::script::test::*;

    pub fn new_blocks(typ: ScriptBlockType, code: ScriptByteCode) -> ScriptBlocks {
        return ScriptBlocks::new(vec![ScriptBlock::new_hook(typ, code).unwrap()]).unwrap();
    }

    #[test]
    fn test_executor_call() {
        let mut ctx = TestEnv::default();
        let mut runner = ScriptRunner::new();
        let mut code = ScriptByteCode::default();

        code.write(&CmdCall::new(
            CmdOpt::Add,
            [
                CmdAddr::new(SEGMENT_IN_MIN, 1),
                CmdAddr::new(SEGMENT_CLOSURE, 0),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        let blocks = new_blocks(BeforeHit, mem::take(&mut code));
        ctx.in1.bb = 3.0;
        ctx.closure_segment[0] = unsafe { mem::transmute(-5.0) };
        runner.run_hook(&blocks, BeforeHit, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, -2.0);

        code.write(&CmdCall::new(
            CmdOpt::Abs,
            [CmdAddr::new(SEGMENT_IN_MIN + 1, 1)],
            CmdAddr::new(SEGMENT_OUT_MIN, 1),
        ));
        let blocks = new_blocks(BeforeHit, mem::take(&mut code));
        ctx.in2.ee = -10.0;
        runner.run_hook(&blocks, BeforeHit, &mut ctx).unwrap();
        assert_eq!(ctx.out.yy, 10.0);

        code.write(&CmdCall::new(
            CmdOpt::Clamp,
            [
                CmdAddr::new(SEGMENT_CONSTANT, 0),
                CmdAddr::new(SEGMENT_REGISTER, 2),
                CmdAddr::new(SEGMENT_IN_MIN + 1, 4),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 2),
        ));
        let mut blocks = new_blocks(AfterAssemble, mem::take(&mut code));
        blocks
            .constant_segment
            .push(unsafe { mem::transmute(111.0) });
        runner.registers[2] = 32.0;
        ctx.in2.gg = 55.5;
        runner.run_hook(&blocks, AfterAssemble, &mut ctx).unwrap();
        assert_eq!(ctx.out.zz, 55.5);

        code.write(&CmdCall::new(
            CmdOpt::IfElse0,
            [
                CmdAddr::new(SEGMENT_REGISTER, 12),
                CmdAddr::new(SEGMENT_REGISTER, 13),
                CmdAddr::new(SEGMENT_REGISTER, 14),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        let mut blocks = new_blocks(AfterAssemble, code);
        blocks
            .constant_segment
            .push(unsafe { mem::transmute(111.0) });
        runner.registers[12] = 1.0;
        runner.registers[13] = 21.0;
        runner.registers[14] = 31.0;
        ctx.in2.gg = 55.5;
        runner.run_hook(&blocks, AfterAssemble, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, 31.0);
    }

    #[test]
    fn test_executor_jmp() {
        let mut ctx = TestEnv::default();
        let mut runner = ScriptRunner::new();
        let mut code = ScriptByteCode::default();

        code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 0)));
        code.write(&CmdCall::new(
            CmdOpt::Add,
            [
                CmdAddr::new(SEGMENT_IN_MIN, 1),
                CmdAddr::new(SEGMENT_CLOSURE, 0),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        code.write(&CmdCall::new(
            CmdOpt::Mov,
            [CmdAddr::new(SEGMENT_IN_MIN, 1)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        let mut blocks = new_blocks(OnAssemble, code);
        blocks.constant_segment.push(6);
        ctx.in1.bb = 3.0;
        ctx.closure_segment[0] = unsafe { mem::transmute(-5.0) };
        runner.run_hook(&blocks, OnAssemble, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, 3.0);
    }

    #[test]
    fn test_executor_jmp_cmp() {
        let mut ctx = TestEnv::default();
        let mut runner = ScriptRunner::new();
        let mut code = ScriptByteCode::default();

        code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_CLOSURE, 20),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
        ));
        code.write(&CmdCall::new(
            CmdOpt::Add,
            [
                CmdAddr::new(SEGMENT_OUT_MIN, 0),
                CmdAddr::new(SEGMENT_IN_MIN, 1),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        code.write(&CmdCall::new(
            CmdOpt::Add,
            [
                CmdAddr::new(SEGMENT_OUT_MIN, 0),
                CmdAddr::new(SEGMENT_IN_MIN, 1),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        let mut blocks = new_blocks(OnTreat, code);
        ctx.in1.bb = 2.0;
        blocks.constant_segment.push(7);

        ctx.closure_segment[20] = unsafe { mem::transmute(0.0) };
        ctx.out.xx = 0.0;
        runner.run_hook(&blocks, OnTreat, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, 2.0);

        ctx.closure_segment[20] = unsafe { mem::transmute(1.0) };
        ctx.out.xx = 0.0;
        runner.run_hook(&blocks, OnTreat, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, 4.0);
    }

    #[test]
    fn test_executor_jmp_set() {
        let mut ctx = TestEnv::default();
        let mut runner = ScriptRunner::new();
        let mut code = ScriptByteCode::default();

        code.write(&CmdJmpSet::new(
            CmdAddr::new(SEGMENT_CLOSURE, 0),
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
        ));
        code.write(&CmdCall::new(
            CmdOpt::Add,
            [
                CmdAddr::new(SEGMENT_OUT_MIN, 0),
                CmdAddr::new(SEGMENT_IN_MIN, 1),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        let mut blocks = new_blocks(OnTreat, code);
        ctx.in1.bb = 2.0;
        blocks.constant_segment.push(8);
        ctx.closure_segment[0] = unsafe { mem::transmute(-3.0) };
        ctx.out.xx = 0.0;

        runner.run_hook(&blocks, OnTreat, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, -3.0);
    }

    #[test]
    fn test_executor_jmp_cas() {
        let mut ctx = TestEnv::default();
        let mut runner = ScriptRunner::new();
        let mut code = ScriptByteCode::default();

        code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas1,
            CmdAddr::new(SEGMENT_CLOSURE, 15),
            CmdAddr::new(SEGMENT_IN_MIN, 0),
            CmdAddr::new(SEGMENT_REGISTER, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
        ));
        code.write(&CmdCall::new(
            CmdOpt::Add,
            [
                CmdAddr::new(SEGMENT_REGISTER, 0),
                CmdAddr::new(SEGMENT_IN_MIN, 2),
            ],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        code.write(&CmdCall::new(
            CmdOpt::Add,
            [
                CmdAddr::new(SEGMENT_REGISTER, 0),
                CmdAddr::new(SEGMENT_IN_MIN, 0),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        let mut blocks = new_blocks(OnTreat, code);
        ctx.in1.aa = -1.0;
        ctx.in1.cc = 5.0;
        blocks.constant_segment.push(9);

        ctx.closure_segment[15] = unsafe { mem::transmute(0.0) };
        ctx.out.xx = 0.0;
        runner.run_hook(&blocks, OnTreat, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, 4.0);

        ctx.closure_segment[15] = unsafe { mem::transmute(1.0) };
        ctx.out.xx = 0.0;
        runner.run_hook(&blocks, OnTreat, &mut ctx).unwrap();
        assert_eq!(ctx.out.xx, -2.0);
    }
}
