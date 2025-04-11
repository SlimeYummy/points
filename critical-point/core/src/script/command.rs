use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD_NO_PAD as b64_engine;
use base64::Engine;
use enum_iterator::Sequence;
use std::{fmt, mem, slice};

use crate::script::config::SEGMENT_NAMES;
use crate::utils::{Num, Symbol};

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct ScriptBlocks {
    blocks: Vec<ScriptBlock>,
    hook_indexes: [u8; ScriptBlockType::count()],
    timer_start: u8,
    pub(crate) constant_segment: Vec<u64>,
    pub(crate) string_segment: Vec<Symbol>,
    pub(crate) arguments: Vec<Symbol>,
    pub(crate) closure_inits: Vec<Num>,
}

impl ScriptBlocks {
    pub fn new(blocks: Vec<ScriptBlock>) -> Result<ScriptBlocks> {
        let mut sb = ScriptBlocks {
            blocks: Vec::new(),
            hook_indexes: [255; ScriptBlockType::count()],
            timer_start: 255,
            constant_segment: Vec::new(),
            string_segment: Vec::new(),
            arguments: Vec::new(),
            closure_inits: Vec::new(),
        };

        let mut hooks = Vec::new();
        let mut timers = Vec::new();

        for block in blocks {
            if block.is_hook() {
                if sb.hook_indexes[block.typ as usize] != 255 {
                    return Err(anyhow!("Duplicated hook"));
                }
                sb.hook_indexes[block.typ as usize] = hooks.len() as u8;
                hooks.push(block);
            } else {
                timers.push(block);
            }
        }

        sb.blocks.extend(hooks);
        sb.timer_start = sb.blocks.len() as u8;
        sb.blocks.extend(timers);
        Ok(sb)
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn blocks(&self) -> &[ScriptBlock] {
        &self.blocks
    }

    pub fn hook_indexes(&self) -> &[u8] {
        &self.hook_indexes
    }

    pub fn timer_start(&self) -> u8 {
        self.timer_start
    }

    pub fn hook(&self, typ: ScriptBlockType) -> Option<&ScriptBlock> {
        let index = self.hook_indexes[typ as usize];
        if index == 255 {
            return None;
        }
        Some(&self.blocks[index as usize])
    }

    pub fn timers(&self) -> &[ScriptBlock] {
        if self.timer_start == 255 {
            return &[];
        }
        &self.blocks[self.timer_start as usize..]
    }

    pub fn constant_segment(&self) -> &[u64] {
        &self.constant_segment
    }

    pub fn string_segment(&self) -> &[Symbol] {
        &self.string_segment
    }

    pub fn arguments(&self) -> &[Symbol] {
        &self.arguments
    }

    pub fn closure_inits(&self) -> &[Num] {
        &self.closure_inits
    }
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ScriptBlock {
    #[serde(rename = "type")]
    pub(crate) typ: ScriptBlockType,
    #[serde(default)]
    pub(crate) arg: Option<Num>,
    pub(crate) code: ScriptByteCode,
}

impl ScriptBlock {
    pub fn new_hook(typ: ScriptBlockType, code: ScriptByteCode) -> Result<ScriptBlock> {
        if !typ.is_hook() {
            return Err(anyhow!("Invalid hook type"));
        }
        Ok(ScriptBlock { typ, arg: None, code })
    }

    pub fn new_timer(typ: ScriptBlockType, time: Num, code: ScriptByteCode) -> Result<ScriptBlock> {
        if !typ.is_timer() {
            return Err(anyhow!("Invalid timer type"));
        }
        Ok(ScriptBlock {
            typ,
            arg: Some(time),
            code,
        })
    }

    pub fn is_hook(&self) -> bool {
        self.typ.is_hook()
    }

    pub fn is_timer(&self) -> bool {
        self.typ.is_timer()
    }

    pub fn typ(&self) -> ScriptBlockType {
        self.typ
    }

    pub fn arg(&self) -> Option<Num> {
        self.arg
    }

    pub fn code(&self) -> &ScriptByteCode {
        &self.code
    }
}

impl fmt::Debug for ScriptBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f
            .debug_struct("ScriptBlock")
            .field("typ", &self.typ)
            .field("code", &self.code.len())
            .finish();
    }
}

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Sequence,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ScriptBlockType {
    OnAssemble,
    AfterAssemble,
    OnStart,
    OnFinish,
    BeforeHit,
    AfterHit,
    BeforeInjure,
    AfterInjure,
    OnTreat,
    OnTimeout,
    OnInterval,
}

impl ScriptBlockType {
    pub const fn count() -> usize {
        ScriptBlockType::OnTreat as usize + 1
    }

    pub fn is_hook(&self) -> bool {
        !self.is_timer()
    }

    pub fn is_timer(&self) -> bool {
        match self {
            ScriptBlockType::OnTimeout | ScriptBlockType::OnInterval => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ScriptByteCode(pub(crate) Vec<u16>);

impl ScriptByteCode {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub(super) fn write<C: Command>(&mut self, cmd: &C) -> usize {
        cmd.write(&mut self.0);
        self.len() - 1
    }

    #[inline(always)]
    pub(crate) fn peek_opt(&self, pc: usize) -> CmdOpt {
        unsafe { mem::transmute::<u16, CmdOpt>(self.0[pc]) }
    }

    #[inline(always)]
    pub(crate) fn jmp(&self, pc: usize) -> &CmdJmp {
        unsafe { &*(self.0.as_ptr().add(pc) as *const CmdJmp) }
    }

    #[inline(always)]
    pub(crate) fn jmp_pc(&self, pc: &mut usize) -> &CmdJmp {
        let cmd = self.jmp(*pc);
        *pc += cmd.len();
        cmd
    }

    #[inline(always)]
    pub(crate) fn jmp_cmp(&self, pc: usize) -> &CmdJmpCmp {
        unsafe { &*(self.0.as_ptr().add(pc) as *const CmdJmpCmp) }
    }

    #[inline(always)]
    pub(crate) fn jmp_cmp_pc(&self, pc: &mut usize) -> &CmdJmpCmp {
        let cmd = self.jmp_cmp(*pc);
        *pc += cmd.len();
        cmd
    }

    #[inline(always)]
    pub(crate) fn jmp_set(&self, pc: usize) -> &CmdJmpSet {
        unsafe { &*(self.0.as_ptr().add(pc) as *const CmdJmpSet) }
    }

    #[inline(always)]
    pub(crate) fn jmp_set_pc(&self, pc: &mut usize) -> &CmdJmpSet {
        let cmd = self.jmp_set(*pc);
        *pc += cmd.len();
        cmd
    }

    #[inline(always)]
    pub(crate) fn jmp_cas(&self, pc: usize) -> &CmdJmpCas {
        unsafe { &*(self.0.as_ptr().add(pc) as *const CmdJmpCas) }
    }

    #[inline(always)]
    pub(crate) fn jmp_cas_pc(&self, pc: &mut usize) -> &CmdJmpCas {
        let cmd = self.jmp_cas(*pc);
        *pc += cmd.len();
        cmd
    }

    #[inline(always)]
    pub(crate) fn call<const N: usize>(&self, pc: usize) -> &CmdCall<N> {
        unsafe { &*(&self.0[pc] as *const _ as *const CmdCall<N>) }
    }

    #[inline(always)]
    pub(crate) fn call_pc<const N: usize>(&self, pc: &mut usize) -> &CmdCall<N> {
        let cmd = self.call(*pc);
        *pc += cmd.len();
        cmd
    }

    #[inline(always)]
    pub(crate) fn call_ext<const N: usize>(&self, pc: usize) -> &CmdCallExt<N> {
        unsafe { &*(&self.0[pc] as *const _ as *const CmdCallExt<N>) }
    }

    #[inline(always)]
    pub(crate) fn call_ext_pc<const N: usize>(&self, pc: &mut usize) -> &CmdCallExt<N> {
        let cmd = self.call_ext(*pc);
        *pc += cmd.len();
        cmd
    }

    pub fn as_slice(&self) -> &[u8] {
        return unsafe { slice::from_raw_parts(self.0.as_ptr() as *const u8, 2 * self.0.len()) };
    }

    pub fn to_base64(&self) -> String {
        return b64_engine.encode(self.as_slice());
    }
}

const _: () = {
    use base64::decoded_len_estimate;
    use serde::de::{self, Deserializer, Visitor};
    use serde::{Deserialize, Serialize};

    impl Serialize for ScriptByteCode {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_base64())
        }
    }

    impl<'de> Deserialize<'de> for ScriptByteCode {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<ScriptByteCode, D::Error> {
            return deserializer.deserialize_str(ScriptByteCodeVisitor);
        }
    }

    struct ScriptByteCodeVisitor;

    impl<'de> Visitor<'de> for ScriptByteCodeVisitor {
        type Value = ScriptByteCode;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a base64 string")
        }

        fn visit_str<E: de::Error>(self, b64: &str) -> Result<Self::Value, E> {
            let buf_len = decoded_len_estimate(b64.len());
            let mut code = Vec::<u16>::with_capacity((buf_len + 1) / 2);
            let raw = unsafe { slice::from_raw_parts_mut(code.as_mut_ptr() as *mut u8, buf_len) };
            let real_len = match b64_engine.decode_slice(b64, raw) {
                Ok(real_len) => real_len,
                Err(err) => return Err(E::custom(format!("Decode base64 {:?}", err))),
            };
            unsafe { code.set_len((real_len + 1) / 2) };
            Ok(ScriptByteCode(code))
        }
    }
};

impl fmt::Debug for ScriptByteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CmdOpt::*;

        let mut dl = f.debug_list();
        let mut pc = 0;
        while pc < self.0.len() {
            match self.peek_opt(pc) {
                Jmp => dl.entry(self.jmp_pc(&mut pc)),
                JmpCmp => dl.entry(self.jmp_cmp_pc(&mut pc)),
                JmpSet => dl.entry(self.jmp_set_pc(&mut pc)),
                JmpCas0 | JmpCas1 => dl.entry(self.jmp_cas_pc(&mut pc)),
                Mov | Neg | Not => dl.entry(self.call_pc::<1>(&mut pc)),
                Add | Sub | Mul | Pow | Div | Mod | Le | Lt | Ge | Gt | Eq | Ne => dl.entry(self.call_pc::<2>(&mut pc)),
                IfElse0 | IfElse1 => dl.entry(self.call_pc::<3>(&mut pc)),
                XInit | XSet => dl.entry(self.call_pc::<2>(&mut pc)),
                XGet | XHas | XDel => dl.entry(self.call_pc::<1>(&mut pc)),
                IsNan | IsInf => dl.entry(self.call_pc::<1>(&mut pc)),
                Abs => dl.entry(self.call_pc::<1>(&mut pc)),
                Min | Max => dl.entry(self.call_pc::<2>(&mut pc)),
                Floor | Ceil | Round => dl.entry(self.call_pc::<1>(&mut pc)),
                Clamp => dl.entry(self.call_pc::<3>(&mut pc)),
                Saturate => dl.entry(self.call_pc::<1>(&mut pc)),
                Lerp => dl.entry(self.call_pc::<3>(&mut pc)),
                Sqrt | Exp => dl.entry(self.call_pc::<1>(&mut pc)),
                Degrees | Radians => dl.entry(self.call_pc::<1>(&mut pc)),
                Sin | Cos | Tan => dl.entry(self.call_pc::<1>(&mut pc)),
                Ext0 => dl.entry(self.call_ext_pc::<0>(&mut pc)),
                Ext1 => dl.entry(self.call_ext_pc::<1>(&mut pc)),
                Ext2 => dl.entry(self.call_ext_pc::<2>(&mut pc)),
                Ext3 => dl.entry(self.call_ext_pc::<3>(&mut pc)),
                Ext4 => dl.entry(self.call_ext_pc::<4>(&mut pc)),
                Ext5 => dl.entry(self.call_ext_pc::<5>(&mut pc)),
                Ext6 => dl.entry(self.call_ext_pc::<6>(&mut pc)),
                Ext7 => dl.entry(self.call_ext_pc::<7>(&mut pc)),
                Ext8 => dl.entry(self.call_ext_pc::<8>(&mut pc)),
                Invalid => return Err(serde::ser::Error::custom("Invalid command")),
            };
        }

        dl.finish()
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum CmdOpt {
    // jump
    Jmp,
    JmpCmp,  // if !expr { pc = addr; }
    JmpSet,  // stack.push(val); pc = addr;
    JmpCas0, // if !expr { stack.push(val); pc = addr; }
    JmpCas1, // if expr { stack.push(val); pc = addr; }

    // unary
    Mov,
    Neg,
    Not,

    // binary
    Add,
    Sub,
    Mul,
    Pow,
    Div,
    Mod,
    Le,
    Lt,
    Ge,
    Gt,
    Eq,
    Ne,

    // bits
    // BitNot,
    // BitAnd,
    // BitOr,
    // BitXor,
    // BitLeft,
    // BitRight,

    // ternary
    IfElse0, // if !expr { stack.push(x) } else { stack.push(y) }
    IfElse1, // if expr { stack.push(x) } else { stack.push(y) }

    // global
    XInit,
    XSet,
    XGet,
    XHas,
    XDel,

    // numeric functions
    IsNan,
    IsInf,
    Abs, // x => |x|
    Min, // a, b => a < b ? b : b
    Max, // a, b => a > b ? a : b
    Floor,
    Ceil,
    Round,
    Clamp,    // x, min, max => x in [min, max]
    Saturate, // x => x in [0, 1]
    Lerp,     // x, y, s => x + s(y - x)

    // exponential functions
    Sqrt,
    Exp,

    // circular functions
    Degrees,
    Radians,
    Sin,
    Cos,
    Tan,

    // ext call
    Ext0,
    Ext1,
    Ext2,
    Ext3,
    Ext4,
    Ext5,
    Ext6,
    Ext7,
    Ext8,

    #[default]
    Invalid,
}

impl From<u16> for CmdOpt {
    fn from(val: u16) -> CmdOpt {
        if val > CmdOpt::Invalid as u16 {
            return CmdOpt::Invalid;
        }
        unsafe { mem::transmute(val) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdType {
    Num,
    Str,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CmdAddr(u16);

impl Default for CmdAddr {
    fn default() -> CmdAddr {
        CmdAddr(0xFFF)
    }
}

impl fmt::Debug for CmdAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f
            .debug_tuple("CmdAddr")
            .field(&SEGMENT_NAMES[self.segment() as usize])
            .field(&self.offset())
            .finish();
    }
}

impl CmdAddr {
    pub fn new(segment: u8, offset: u16) -> CmdAddr {
        let segment = segment as u16;
        let offset = offset.min(0xFFF);
        CmdAddr((segment << 12) | offset)
    }

    pub fn segment(&self) -> u8 {
        (self.0 >> 12) as u8
    }

    pub fn offset(&self) -> u16 {
        self.0 & 0xFFF
    }

    pub const fn max_offset() -> u16 {
        0xFFF
    }
}

pub trait Command {
    fn write(&self, code: &mut Vec<u16>);
    fn len(&self) -> usize;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdCall<const N: usize> {
    pub opt: CmdOpt,
    pub src: [CmdAddr; N],
    pub dst: CmdAddr,
}

impl<const N: usize> CmdCall<N> {
    pub fn new(opt: CmdOpt, src: [CmdAddr; N], dst: CmdAddr) -> CmdCall<N> {
        CmdCall { opt, src, dst }
    }
}

impl<const N: usize> Command for CmdCall<N> {
    fn write(&self, code: &mut Vec<u16>) {
        unsafe {
            code.push(mem::transmute::<_, u16>(self.opt));
            for idx in 0..N {
                code.push(mem::transmute::<_, u16>(self.src[idx]));
            }
            code.push(mem::transmute::<_, u16>(self.dst));
        }
    }

    #[inline(always)]
    fn len(&self) -> usize {
        mem::size_of::<CmdCall<N>>() / mem::size_of::<u16>()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdCallExt<const N: usize> {
    pub opt: CmdOpt,
    pub ext: u16,
    pub src: [CmdAddr; N],
    pub dst: CmdAddr,
}

impl<const N: usize> CmdCallExt<N> {
    pub fn new(opt: CmdOpt, ext: u16, src: [CmdAddr; N], dst: CmdAddr) -> CmdCallExt<N> {
        CmdCallExt { opt, ext, src, dst }
    }
}

impl<const N: usize> Command for CmdCallExt<N> {
    fn write(&self, code: &mut Vec<u16>) {
        unsafe {
            code.push(mem::transmute::<_, u16>(self.opt));
            for idx in 0..N {
                code.push(mem::transmute::<_, u16>(self.src[idx]));
            }
            code.push(mem::transmute::<_, u16>(self.dst));
        }
    }

    #[inline(always)]
    fn len(&self) -> usize {
        mem::size_of::<CmdCallExt<N>>() / mem::size_of::<u16>()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdJmp {
    pub opt: CmdOpt,
    pub pc: CmdAddr,
}

impl CmdJmp {
    pub fn new(pc: CmdAddr) -> CmdJmp {
        CmdJmp { opt: CmdOpt::Jmp, pc }
    }
}

impl Command for CmdJmp {
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 2]>(*self)) };
    }

    #[inline(always)]
    fn len(&self) -> usize {
        mem::size_of::<CmdJmp>() / mem::size_of::<u16>()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdJmpCmp {
    pub opt: CmdOpt,
    pub cond: CmdAddr,
    pub pc: CmdAddr,
}

impl CmdJmpCmp {
    pub fn new(cond: CmdAddr, pc: CmdAddr) -> CmdJmpCmp {
        CmdJmpCmp {
            opt: CmdOpt::JmpCmp,
            cond,
            pc,
        }
    }
}

impl Command for CmdJmpCmp {
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 3]>(*self)) };
    }

    #[inline(always)]
    fn len(&self) -> usize {
        mem::size_of::<CmdJmpCmp>() / mem::size_of::<u16>()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdJmpSet {
    pub opt: CmdOpt,
    pub src: CmdAddr,
    pub dst: CmdAddr,
    pub pc: CmdAddr,
}

impl CmdJmpSet {
    pub fn new(src: CmdAddr, dst: CmdAddr, pc: CmdAddr) -> CmdJmpSet {
        CmdJmpSet {
            opt: CmdOpt::JmpSet,
            src,
            dst,
            pc,
        }
    }
}

impl Command for CmdJmpSet {
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 4]>(*self)) };
    }

    #[inline(always)]
    fn len(&self) -> usize {
        mem::size_of::<CmdJmpSet>() / mem::size_of::<u16>()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdJmpCas {
    pub opt: CmdOpt,
    pub cond: CmdAddr,
    pub src: CmdAddr,
    pub dst: CmdAddr,
    pub pc: CmdAddr,
}

impl CmdJmpCas {
    pub fn new(opt: CmdOpt, cond: CmdAddr, src: CmdAddr, dst: CmdAddr, pc: CmdAddr) -> CmdJmpCas {
        CmdJmpCas {
            opt,
            cond,
            src,
            dst,
            pc,
        }
    }
}

impl Command for CmdJmpCas {
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 5]>(*self)) };
    }

    #[inline(always)]
    fn len(&self) -> usize {
        mem::size_of::<CmdJmpCas>() / mem::size_of::<u16>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::sb;

    #[test]
    fn test_script_blocks_deserde() {
        let code = r#"{
            "blocks": [
                { "type": "OnStart", "code": "AA" },
                { "type": "OnTreat", "code": "AAA" },
                { "type": "OnTimeout", "arg": 10, "code": "AAAA" },
                { "type": "OnInterval", "arg": 20, "code": "" }
            ],
            "hook_indexes": [255, 255, 0, 255, 255, 255, 255, 255, 1],
            "timer_start": 2,
            "constant_segment": [1, 2, 3, 4],
            "string_segment": ["aaa", "bbb", "ccc"],
            "arguments": ["ax", "bx", "cx"],
            "closure_inits": [-10.0, 20.0, 30.0]
        }"#;
        let blocks: ScriptBlocks = serde_json::from_str(code).unwrap();
        assert_eq!(blocks.blocks.len(), 4);
        assert_eq!(blocks.blocks[1].typ, ScriptBlockType::OnTreat);
        assert_eq!(blocks.blocks[3].typ, ScriptBlockType::OnInterval);
        assert_eq!(
            blocks.hook_indexes,
            [255u8, 255u8, 0u8, 255u8, 255u8, 255u8, 255u8, 255u8, 1u8]
        );
        assert_eq!(blocks.timer_start, 2);
        assert_eq!(blocks.constant_segment, &[1, 2, 3, 4]);
        assert_eq!(blocks.string_segment, &[sb!("aaa"), sb!("bbb"), sb!("ccc")]);
        assert_eq!(blocks.arguments, &[sb!("ax"), sb!("bx"), sb!("cx")]);
        assert_eq!(blocks.closure_inits, &[-10.0, 20.0, 30.0]);
    }
}
