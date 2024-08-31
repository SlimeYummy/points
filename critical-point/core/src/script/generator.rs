use anyhow::{anyhow, Result};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::mem;

use crate::script::ast::{
    AstBlock, AstExpr, AstExprBranch, AstExprCall, AstExprCallExt, AstExprLogic, AstLogicType, AstStat, AstStatAssign,
    AstStatBranch, AstStatCall, AstStatCallExt, AstVar,
};
use crate::script::command::*;
use crate::script::config::{MAX_OFFSET, MAX_REGISTER, SEGMENT_CONSTANT, SEGMENT_REGISTER, SEGMENT_STRING};
use crate::script::parser::ParserResult;
use crate::utils::{Num, Symbol, XResult};

pub struct ScriptGenerator {
    reg_manager: RegisterManager,
    byte_code: ScriptByteCode,
    constant_writer: ConstantWriter,
    string_writer: StringWriter,
}

impl ScriptGenerator {
    pub fn new() -> ScriptGenerator {
        return ScriptGenerator {
            reg_manager: RegisterManager::default(),
            byte_code: ScriptByteCode::default(),
            constant_writer: ConstantWriter::default(),
            string_writer: StringWriter::default(),
        };
    }

    pub fn run(&mut self, pres: ParserResult) -> Result<ScriptBlocks> {
        self.constant_writer.clear();
        self.string_writer.clear();

        let mut blocks = Vec::new();
        for ast_block in &pres.blocks {
            self.reg_manager.reset();
            self.byte_code = ScriptByteCode::default();

            self.visit_block(&ast_block)?;

            if ast_block.typ.is_hook() {
                let block = ScriptBlock::new_hook(ast_block.typ, mem::take(&mut self.byte_code))?;
                blocks.push(block);
            } else {
                let block = ScriptBlock::new_timer(
                    ast_block.typ,
                    ast_block.arg.ok_or_else(|| anyhow!("Timer miss argument"))?,
                    mem::take(&mut self.byte_code),
                )?;
                blocks.push(block);
            }
        }

        let mut bs = ScriptBlocks::new(blocks)?;
        bs.arguments = pres.arguments;
        bs.closure_inits = pres.closure_inits;
        bs.constant_segment = self.constant_writer.take();
        bs.string_segment = self.string_writer.take()?;
        return Ok(bs);
    }

    fn visit_block(&mut self, block: &AstBlock) -> Result<()> {
        for stat in &block.stats {
            self.visit_stat(stat, NO_CTX)?;
        }
        return Ok(());
    }

    fn visit_stat(&mut self, stat: &AstStat, ctx: Context) -> Result<()> {
        return match stat {
            AstStat::Assign(assign) => self.visit_stat_assign(assign, ctx),
            AstStat::Call(call) => self.visit_stat_call(call, ctx),
            AstStat::CallExt(call_ext) => self.visit_stat_call_ext(call_ext, ctx),
            AstStat::Branch(branch) => self.visit_stat_branch(branch, ctx),
            AstStat::Return(_) => self.visit_stat_return(),
        };
    }

    fn visit_stat_assign(&mut self, assign: &AstStatAssign, ctx: Context) -> Result<()> {
        let var_addr = match assign.var {
            AstVar::Local(id) => self.reg_manager.find_or_alloc_local(id)?,
            AstVar::Closure(addr) => addr,
            AstVar::Output(addr) => addr,
        };

        let expr_addr = self.visit_expr(&assign.expr, Context::new(Some(var_addr), ctx.end))?;
        if expr_addr != var_addr {
            self.byte_code.write(&CmdCall::new(CmdOpt::Mov, [expr_addr], var_addr));
            self.reg_manager.free_register(expr_addr);
        }

        return Ok(());
    }

    fn visit_stat_call(&mut self, call: &AstStatCall, ctx: Context) -> Result<()> {
        let _ = match call.args.len() {
            0 => self.visit_stat_call_impl::<0>(call, ctx),
            1 => self.visit_stat_call_impl::<1>(call, ctx),
            2 => self.visit_stat_call_impl::<2>(call, ctx),
            3 => self.visit_stat_call_impl::<3>(call, ctx),
            4 => self.visit_stat_call_impl::<4>(call, ctx),
            5 => self.visit_stat_call_impl::<5>(call, ctx),
            6 => self.visit_stat_call_impl::<6>(call, ctx),
            7 => self.visit_stat_call_impl::<7>(call, ctx),
            8 => self.visit_stat_call_impl::<8>(call, ctx),
            _ => return Err(anyhow!("Too many arguments {:?}", call.opt)),
        };
        return Ok(());
    }

    fn visit_stat_call_impl<const N: usize>(&mut self, call: &AstStatCall, ctx: Context) -> Result<()> {
        let mut src: [CmdAddr; N] = [CmdAddr::default(); N];
        for idx in 0..N {
            src[idx] = self.visit_expr(&call.args[idx], ctx)?;
        }

        let cmd = CmdCall {
            opt: call.opt,
            src,
            dst: CmdAddr::default(),
        };
        self.reg_manager.free_registers(&cmd.src);

        self.byte_code.write(&cmd);
        return Ok(());
    }

    fn visit_stat_call_ext(&mut self, call_ext: &AstStatCallExt, ctx: Context) -> Result<()> {
        let _ = match call_ext.args.len() {
            0 => self.visit_stat_call_ext_impl::<0>(CmdOpt::Ext0, call_ext, ctx),
            1 => self.visit_stat_call_ext_impl::<1>(CmdOpt::Ext1, call_ext, ctx),
            2 => self.visit_stat_call_ext_impl::<2>(CmdOpt::Ext2, call_ext, ctx),
            3 => self.visit_stat_call_ext_impl::<3>(CmdOpt::Ext3, call_ext, ctx),
            4 => self.visit_stat_call_ext_impl::<4>(CmdOpt::Ext4, call_ext, ctx),
            5 => self.visit_stat_call_ext_impl::<5>(CmdOpt::Ext5, call_ext, ctx),
            6 => self.visit_stat_call_ext_impl::<6>(CmdOpt::Ext6, call_ext, ctx),
            7 => self.visit_stat_call_ext_impl::<7>(CmdOpt::Ext7, call_ext, ctx),
            8 => self.visit_stat_call_ext_impl::<8>(CmdOpt::Ext8, call_ext, ctx),
            _ => return Err(anyhow!("Too many arguments {:?}", call_ext.ext)),
        };
        return Ok(());
    }

    fn visit_stat_call_ext_impl<const N: usize>(
        &mut self,
        opt: CmdOpt,
        call_ext: &AstStatCallExt,
        ctx: Context,
    ) -> Result<()> {
        let mut src: [CmdAddr; N] = [CmdAddr::default(); N];
        for idx in 0..N {
            src[idx] = self.visit_expr(&call_ext.args[idx], ctx)?;
        }

        let cmd = CmdCallExt {
            opt,
            ext: call_ext.ext,
            src,
            dst: CmdAddr::default(),
        };
        self.reg_manager.free_registers(&cmd.src);

        self.byte_code.write(&cmd);
        return Ok(());
    }

    fn visit_stat_branch(&mut self, branch: &AstStatBranch, ctx: Context) -> Result<()> {
        self.reg_manager.push_scope();

        match (&branch.cond, &branch.next) {
            // if/elsif condition with next branch
            (Some(cond), Some(next)) => {
                let cmd_if = CmdJmpCmp::new(self.visit_expr(&cond, NO_CTX)?, self.constant_writer.write_pc(0)?);
                self.reg_manager.free_register(cmd_if.cond);
                self.byte_code.write(&cmd_if);

                let end = self.use_or_alloc_pc(ctx.end)?;
                for (idx, stat) in branch.stats.iter().enumerate() {
                    if idx != branch.stats.len() - 1 {
                        self.visit_stat(stat, NO_CTX)?;
                    } else {
                        self.visit_stat(stat, Context::new(None, Some(end)))?;
                    }
                }

                self.byte_code.write(&CmdJmp::new(end));
                self.constant_writer.update_pc(cmd_if.pc, self.byte_code.len() as u64)?; // to next

                self.visit_stat_branch(next, Context::new(None, Some(end)))?;
            }
            // if/elsif condition without next branch
            (Some(cond), None) => {
                let cmd_if = CmdJmpCmp::new(self.visit_expr(&cond, NO_CTX)?, self.use_or_alloc_pc(ctx.end)?);
                self.reg_manager.free_register(cmd_if.cond);
                self.byte_code.write(&cmd_if);

                for (idx, stat) in branch.stats.iter().enumerate() {
                    if idx != branch.stats.len() - 1 {
                        self.visit_stat(stat, NO_CTX)?;
                    } else {
                        self.visit_stat(stat, Context::new(None, Some(cmd_if.pc)))?;
                    }
                }

                self.constant_writer.update_pc(cmd_if.pc, self.byte_code.len() as u64)?;
                // to end
            }
            // else
            (None, None) => {
                for (idx, stat) in branch.stats.iter().enumerate() {
                    if idx != branch.stats.len() - 1 {
                        self.visit_stat(stat, NO_CTX)?;
                    } else {
                        self.visit_stat(stat, Context::new(None, ctx.end))?;
                    }
                }

                self.constant_writer
                    .update_pc(ctx.end.unwrap(), self.byte_code.len() as u64)?;
                // to end
            }
            _ => unreachable!(),
        }

        self.reg_manager.pop_scope();
        return Ok(());
    }

    fn visit_stat_return(&mut self) -> Result<()> {
        // we use write_num here to cache the special pc
        let cmd = CmdJmp::new(
            self.constant_writer
                .write_num(unsafe { mem::transmute(core::u64::MAX) })?,
        );

        self.byte_code.write(&cmd);
        return Ok(());
    }

    fn visit_expr(&mut self, expr: &AstExpr, ctx: Context) -> Result<CmdAddr> {
        return match expr {
            AstExpr::Num(num) => self.constant_writer.write_num(*num),
            AstExpr::Str(str) => self.string_writer.write(str),
            AstExpr::Local(id) => self.reg_manager.find_local(*id),
            AstExpr::Closure(addr) => Ok(*addr),
            AstExpr::Argument(addr) => Ok(*addr),
            AstExpr::Input(addr) => Ok(*addr),
            AstExpr::Output(addr) => Ok(*addr),
            AstExpr::Call(call) => self.visit_expr_call(call, ctx),
            AstExpr::CallExt(call_ext) => self.visit_expr_call_ext(call_ext, ctx),
            AstExpr::Branch(branch) => self.visit_expr_branch(branch, ctx),
            AstExpr::Logic(logic) => self.visit_expr_logic(logic, ctx),
        };
    }

    fn visit_expr_call(&mut self, call: &AstExprCall, ctx: Context) -> Result<CmdAddr> {
        return match call.args.len() {
            0 => self.visit_expr_call_impl::<0>(call, ctx),
            1 => self.visit_expr_call_impl::<1>(call, ctx),
            2 => self.visit_expr_call_impl::<2>(call, ctx),
            3 => self.visit_expr_call_impl::<3>(call, ctx),
            4 => self.visit_expr_call_impl::<4>(call, ctx),
            5 => self.visit_expr_call_impl::<5>(call, ctx),
            6 => self.visit_expr_call_impl::<6>(call, ctx),
            7 => self.visit_expr_call_impl::<7>(call, ctx),
            8 => self.visit_expr_call_impl::<8>(call, ctx),
            _ => Err(anyhow!("Too many arguments {:?}", call.opt)),
        };
    }

    fn visit_expr_call_impl<const N: usize>(&mut self, call: &AstExprCall, ctx: Context) -> Result<CmdAddr> {
        let mut src: [CmdAddr; N] = [CmdAddr::default(); N];
        for idx in 0..N {
            src[idx] = self.visit_expr(&call.args[idx], NO_CTX)?;
        }

        let mut cmd = CmdCall {
            opt: call.opt,
            src,
            dst: CmdAddr::default(),
        };
        self.reg_manager.free_registers(&cmd.src);

        cmd.dst = self.use_or_alloc_register(ctx.dst)?;
        self.byte_code.write(&cmd);
        return Ok(cmd.dst);
    }

    fn visit_expr_call_ext(&mut self, call_ext: &AstExprCallExt, ctx: Context) -> Result<CmdAddr> {
        return match call_ext.args.len() {
            0 => self.visit_expr_call_ext_impl::<0>(CmdOpt::Ext0, call_ext, ctx),
            1 => self.visit_expr_call_ext_impl::<1>(CmdOpt::Ext1, call_ext, ctx),
            2 => self.visit_expr_call_ext_impl::<2>(CmdOpt::Ext2, call_ext, ctx),
            3 => self.visit_expr_call_ext_impl::<3>(CmdOpt::Ext3, call_ext, ctx),
            4 => self.visit_expr_call_ext_impl::<4>(CmdOpt::Ext4, call_ext, ctx),
            5 => self.visit_expr_call_ext_impl::<5>(CmdOpt::Ext5, call_ext, ctx),
            6 => self.visit_expr_call_ext_impl::<6>(CmdOpt::Ext6, call_ext, ctx),
            7 => self.visit_expr_call_ext_impl::<7>(CmdOpt::Ext7, call_ext, ctx),
            8 => self.visit_expr_call_ext_impl::<8>(CmdOpt::Ext8, call_ext, ctx),
            _ => Err(anyhow!("Too many arguments {:?}", call_ext.ext)),
        };
    }

    fn visit_expr_call_ext_impl<const N: usize>(
        &mut self,
        opt: CmdOpt,
        call_ext: &AstExprCallExt,
        ctx: Context,
    ) -> Result<CmdAddr> {
        let mut src: [CmdAddr; N] = [CmdAddr::default(); N];
        for idx in 0..N {
            src[idx] = self.visit_expr(&call_ext.args[idx], NO_CTX)?;
        }

        let mut cmd = CmdCallExt {
            opt,
            ext: call_ext.ext,
            src,
            dst: CmdAddr::default(),
        };
        self.reg_manager.free_registers(&cmd.src);

        cmd.dst = self.use_or_alloc_register(ctx.dst)?;
        self.byte_code.write(&cmd);
        return Ok(cmd.dst);
    }

    fn visit_expr_branch(&mut self, branch: &AstExprBranch, ctx: Context) -> Result<CmdAddr> {
        // 2 value
        if branch.left.is_value() && branch.right.is_value() {
            let mut cmd = CmdCall::new(
                CmdOpt::IfElse1,
                [
                    self.visit_expr(&branch.cond, NO_CTX)?,
                    self.visit_expr(&branch.left, NO_CTX)?,
                    self.visit_expr(&branch.right, NO_CTX)?,
                ],
                CmdAddr::default(),
            );
            self.reg_manager.free_registers(&cmd.src);

            cmd.dst = self.use_or_alloc_register(ctx.dst)?;
            self.byte_code.write(&cmd);
            return Ok(cmd.dst);

        // 1 value
        } else if branch.left.is_value() || branch.right.is_value() {
            let mut cmd = if branch.left.is_value() {
                CmdJmpCas::new(
                    CmdOpt::JmpCas1,
                    self.visit_expr(&branch.cond, NO_CTX)?,
                    self.visit_expr(&branch.left, NO_CTX)?,
                    CmdAddr::default(),
                    self.use_or_alloc_pc(ctx.end)?,
                )
            } else {
                CmdJmpCas::new(
                    CmdOpt::JmpCas0,
                    self.visit_expr(&branch.cond, NO_CTX)?,
                    self.visit_expr(&branch.right, NO_CTX)?,
                    CmdAddr::default(),
                    self.use_or_alloc_pc(ctx.end)?,
                )
            };
            self.reg_manager.free_registers(&[cmd.cond, cmd.src]);

            cmd.dst = self.use_or_alloc_register(ctx.dst)?;
            self.byte_code.write(&cmd);

            if branch.left.is_value() {
                self.visit_expr(&branch.right, Context::new(Some(cmd.dst), Some(cmd.pc)))?;
            } else {
                self.visit_expr(&branch.left, Context::new(Some(cmd.dst), Some(cmd.pc)))?;
            }

            self.constant_writer.update_pc(cmd.pc, self.byte_code.len() as u64)?; // to end
            return Ok(cmd.dst);

        // 0 value
        } else {
            let cmd_if = CmdJmpCmp::new(
                self.visit_expr(&branch.cond, NO_CTX)?,
                self.constant_writer.write_pc(0)?,
            );
            self.reg_manager.free_register(cmd_if.cond);
            self.byte_code.write(&cmd_if);

            let end = self.use_or_alloc_pc(ctx.end)?;
            let dst = self.visit_expr(&branch.left, Context::new(ctx.dst, Some(end)))?;
            self.constant_writer.update_pc(cmd_if.pc, self.byte_code.len() as u64)?; // to next

            let cmd_left = CmdJmp::new(end);
            self.byte_code.write(&cmd_left);

            self.visit_expr(&branch.right, Context::new(Some(dst), Some(end)))?;
            self.constant_writer
                .update_pc(cmd_left.pc, self.byte_code.len() as u64)?; // to end

            return Ok(dst);
        }
    }

    fn visit_expr_logic(&mut self, logic: &AstExprLogic, ctx: Context) -> Result<CmdAddr> {
        let left_expr = self.visit_expr(&logic.left, NO_CTX)?;

        // * || value
        if logic.right.is_value() {
            let opt = match logic.typ {
                AstLogicType::And => CmdOpt::IfElse0,
                AstLogicType::Or => CmdOpt::IfElse1,
            };

            let mut cmd = CmdCall::new(
                opt,
                [left_expr, left_expr, self.visit_expr(&logic.right, NO_CTX)?],
                CmdAddr::default(),
            );
            self.reg_manager.free_registers(&cmd.src);

            cmd.dst = self.use_or_alloc_register(ctx.dst)?;
            self.byte_code.write(&cmd);
            return Ok(cmd.dst);

        // * || expr
        } else {
            let opt = match logic.typ {
                AstLogicType::And => CmdOpt::JmpCas0,
                AstLogicType::Or => CmdOpt::JmpCas1,
            };

            let mut cmd = CmdJmpCas::new(
                opt,
                left_expr,
                left_expr,
                CmdAddr::default(),
                self.use_or_alloc_pc(ctx.end)?,
            );
            self.reg_manager.free_register(left_expr);

            cmd.dst = self.use_or_alloc_register(ctx.dst)?;
            self.byte_code.write(&cmd);

            self.visit_expr(&logic.right, Context::new(Some(cmd.dst), None))?;
            self.constant_writer.update_pc(cmd.pc, self.byte_code.len() as u64)?; // to end

            return Ok(cmd.dst);
        }
    }

    fn use_or_alloc_register(&mut self, dst: Option<CmdAddr>) -> Result<CmdAddr> {
        return match dst {
            Some(dst) => Ok(dst),
            None => self.reg_manager.alloc_register(),
        };
    }

    fn use_or_alloc_pc(&mut self, dst: Option<CmdAddr>) -> Result<CmdAddr> {
        return match dst {
            Some(dst) => Ok(dst),
            None => self.constant_writer.write_pc(0),
        };
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Context {
    dst: Option<CmdAddr>,
    end: Option<CmdAddr>,
}

const NO_CTX: Context = Context { dst: None, end: None };

impl Context {
    fn new(dst: Option<CmdAddr>, end: Option<CmdAddr>) -> Context {
        return Context { dst, end };
    }
}

#[derive(Debug, Default)]
struct RegisterManager {
    register_max: u16,
    register_heap: BinaryHeap<Reverse<u16>>,
    registers: HashSet<CmdAddr>,
    locals: HashMap<u32, (CmdAddr, u32)>,
    scope_depth: u32,
}

impl RegisterManager {
    fn alloc_register(&mut self) -> Result<CmdAddr> {
        let addr;
        if let Some(offset) = self.register_heap.pop() {
            addr = CmdAddr::new(SEGMENT_REGISTER, offset.0);
        } else {
            if self.register_max as usize >= MAX_REGISTER {
                return Err(anyhow!("Register segment overflow"));
            }
            addr = CmdAddr::new(SEGMENT_REGISTER, self.register_max);
            self.register_max += 1;
        }
        self.registers.insert(addr);
        return Ok(addr);
    }

    fn free_register(&mut self, addr: CmdAddr) {
        if addr.segment() == SEGMENT_REGISTER && self.registers.contains(&addr) {
            self.registers.remove(&addr);
            self.register_heap.push(Reverse(addr.offset()));
        }
    }

    fn free_registers(&mut self, addrs: &[CmdAddr]) {
        for addr in addrs {
            self.free_register(*addr);
        }
    }

    // fn is_register(&self, addr: &CmdAddr) -> bool {
    //     return self.registers.contains(addr);
    // }

    fn find_or_alloc_local(&mut self, id: u32) -> Result<CmdAddr> {
        if let Some((addr, _)) = self.locals.get(&id) {
            return Ok(*addr);
        }

        let addr;
        if let Some(offset) = self.register_heap.pop() {
            addr = CmdAddr::new(SEGMENT_REGISTER, offset.0);
        } else {
            if self.register_max as usize >= MAX_REGISTER {
                return Err(anyhow!("Register segment overflow"));
            }
            addr = CmdAddr::new(SEGMENT_REGISTER, self.register_max);
        }

        self.register_max += 1;
        self.locals.insert(id, (addr, self.scope_depth));
        return Ok(addr);
    }

    fn find_local(&mut self, id: u32) -> Result<CmdAddr> {
        if let Some((addr, _)) = self.locals.get(&id) {
            return Ok(*addr);
        }
        return Err(anyhow!("Local variable not found"));
    }

    fn push_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn pop_scope(&mut self) {
        self.scope_depth -= 1;
        self.locals.retain(|_, (addr, depth)| {
            let remove = *depth > self.scope_depth;
            if remove {
                self.register_heap.push(Reverse(addr.offset()));
            }
            !remove
        });
    }

    fn reset(&mut self) {
        self.register_max = 0;
        self.register_heap.clear();
        self.locals.clear();
        self.scope_depth = 0;
    }
}

#[derive(Debug, Default)]
struct ConstantWriter {
    nums: HashMap<u64, u16>,
    buffer: Vec<u64>,
}

impl ConstantWriter {
    fn write_num(&mut self, num: Num) -> Result<CmdAddr> {
        let num: u64 = unsafe { mem::transmute(num) };
        if let Some(offset) = self.nums.get(&num) {
            return Ok(CmdAddr::new(SEGMENT_CONSTANT, *offset));
        }
        if self.buffer.len() >= MAX_OFFSET {
            return Err(anyhow!("Constant segment overflow"));
        }
        self.buffer.push(num);
        let offset = self.buffer.len() as u16 - 1;
        self.nums.insert(num, offset);
        return Ok(CmdAddr::new(SEGMENT_CONSTANT, offset));
    }

    fn write_pc(&mut self, pc: u64) -> Result<CmdAddr> {
        if self.buffer.len() >= MAX_OFFSET {
            return Err(anyhow!("Constant segment overflow"));
        }
        self.buffer.push(pc);
        let offset = self.buffer.len() as u16 - 1;
        return Ok(CmdAddr::new(SEGMENT_CONSTANT, offset));
    }

    fn update_pc(&mut self, addr: CmdAddr, pc: u64) -> Result<()> {
        if addr.segment() != SEGMENT_CONSTANT {
            return Err(anyhow!("Invalid constant segment"));
        }
        self.buffer[addr.offset() as usize] = pc;
        return Ok(());
    }

    fn take(&mut self) -> Vec<u64> {
        let buffer = self.buffer.clone();
        self.buffer.clear();
        self.nums.clear();
        return buffer;
    }

    fn clear(&mut self) {
        self.buffer.clear();
        self.nums.clear();
    }
}

#[derive(Debug, Default)]
struct StringWriter {
    strs: HashMap<String, u16>,
    buffer: Vec<String>,
}

impl StringWriter {
    fn write(&mut self, str: &str) -> Result<CmdAddr> {
        if let Some(offset) = self.strs.get(str) {
            return Ok(CmdAddr::new(SEGMENT_STRING, *offset));
        }
        if self.buffer.len() >= MAX_OFFSET {
            return Err(anyhow!("String segment overflow"));
        }
        self.buffer.push(str.into());
        let offset = self.buffer.len() as u16 - 1;
        self.strs.insert(str.into(), offset);
        return Ok(CmdAddr::new(SEGMENT_STRING, offset));
    }

    fn take(&mut self) -> XResult<Vec<Symbol>> {
        let symbols = self.buffer.iter().map(|s| Symbol::try_from(s)).collect();
        self.buffer.clear();
        self.strs.clear();
        return symbols;
    }

    fn clear(&mut self) {
        self.buffer.clear();
        self.strs.clear();
    }
}

#[cfg(test)]
mod tests {
    use ScriptBlockType::*;

    use super::*;
    use crate::script::config::{SEGMENT_CLOSURE, SEGMENT_IN_MIN, SEGMENT_OUT_MIN};
    use crate::script::test::*;

    #[test]
    fn test_generator_empty() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = "on_start {}";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(
            blocks.hook(OnStart).unwrap(),
            &ScriptBlock::new_hook(OnStart, ScriptByteCode::default()).unwrap()
        );
    }

    #[test]
    fn test_generator_stat_assign() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            const TWO = 2
            var closure = 3
            on_start {
                out.xx = TWO + closure
                out.zz *= in.ee
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(blocks.closure_inits, vec![3.0]);
        assert_eq!(blocks.constant_segment, vec![f64_u64(2.0)]);

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_CONSTANT, 0), CmdAddr::new(SEGMENT_CLOSURE, 0)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Mul,
            [CmdAddr::new(SEGMENT_OUT_MIN, 2), CmdAddr::new(SEGMENT_IN_MIN + 1, 1)],
            CmdAddr::new(SEGMENT_OUT_MIN, 2),
        ));
        assert_eq!(
            blocks.hook(OnStart).unwrap(),
            &ScriptBlock::new_hook(OnStart, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_stat_call() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            on_finish {
                const NUM = 123
                G.init('key-1', math.PI + math.max(math.PI, NUM))
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(
            blocks.constant_segment,
            vec![f64_u64(core::f64::consts::PI), f64_u64(123.0)]
        );
        assert_eq!(blocks.string_segment, vec!["key-1".to_string(),]);

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdCall::new(
            CmdOpt::Max,
            [CmdAddr::new(SEGMENT_CONSTANT, 0), CmdAddr::new(SEGMENT_CONSTANT, 1)],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_CONSTANT, 0), CmdAddr::new(SEGMENT_REGISTER, 0)],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::XInit,
            [CmdAddr::new(SEGMENT_STRING, 0), CmdAddr::new(SEGMENT_REGISTER, 0)],
            CmdAddr::default(),
        ));
        assert_eq!(
            blocks.hook(OnFinish).unwrap(),
            &ScriptBlock::new_hook(OnFinish, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_stat_branch_basic() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = "
            on_finish {
                if in.cc > 0 {
                    const N = 3
                    out.xx = N
                } elsif 7 {
                } else {
                    const N = 9
                    out.yy -= N
                }
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(
            blocks.constant_segment,
            vec![f64_u64(0.0), 12, 21, f64_u64(3.0), f64_u64(7.0), 17, f64_u64(9.0),]
        );

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdCall::new(
            CmdOpt::Gt,
            [CmdAddr::new(SEGMENT_IN_MIN, 2), CmdAddr::new(SEGMENT_CONSTANT, 0)],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_REGISTER, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Mov,
            [CmdAddr::new(SEGMENT_CONSTANT, 3)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        byte_code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 2)));
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_CONSTANT, 4),
            CmdAddr::new(SEGMENT_CONSTANT, 5),
        ));
        byte_code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 2)));
        byte_code.write(&CmdCall::new(
            CmdOpt::Sub,
            [CmdAddr::new(SEGMENT_OUT_MIN, 1), CmdAddr::new(SEGMENT_CONSTANT, 6)],
            CmdAddr::new(SEGMENT_OUT_MIN, 1),
        ));
        assert_eq!(
            blocks.hook(OnFinish).unwrap(),
            &ScriptBlock::new_hook(OnFinish, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_stat_branch_to_end() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = "
            on_finish {
                if in.aa {
                    if in.cc != 0 {
                        out.xx = 6
                    } else {
                        out.xx = 5
                    }
                }
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(
            blocks.constant_segment,
            vec![18, f64_u64(0.0), 15, f64_u64(6.0), f64_u64(5.0),]
        );

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_IN_MIN, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Ne,
            [CmdAddr::new(SEGMENT_IN_MIN, 2), CmdAddr::new(SEGMENT_CONSTANT, 1)],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_REGISTER, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 2),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Mov,
            [CmdAddr::new(SEGMENT_CONSTANT, 3)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        byte_code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 0)));
        byte_code.write(&CmdCall::new(
            CmdOpt::Mov,
            [CmdAddr::new(SEGMENT_CONSTANT, 4)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        assert_eq!(
            blocks.hook(OnFinish).unwrap(),
            &ScriptBlock::new_hook(OnFinish, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_stat_return() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            on_finish { return }
            on_timeout(10) {
                if in.bb { return }
                out.xx = -4.5
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(blocks.constant_segment, vec![core::u64::MAX, 5, f64_u64(-4.5),]);

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 0)));
        assert_eq!(
            blocks.hook(OnFinish).unwrap(),
            &ScriptBlock::new_hook(OnFinish, byte_code).unwrap()
        );

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_IN_MIN, 1),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
        ));
        byte_code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 0)));
        byte_code.write(&CmdCall::new(
            CmdOpt::Mov,
            [CmdAddr::new(SEGMENT_CONSTANT, 2)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        assert_eq!(
            blocks.timers(),
            &[ScriptBlock::new_timer(OnTimeout, 10.0, byte_code).unwrap()]
        );
    }

    #[test]
    fn test_generator_expr_local() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            on_timeout(10) {
                var v1 = 10
                if in.bb == 1 {
                    var v2 = v1 + 20
                }
                var v3 = 30
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdCall::new(
            CmdOpt::Mov,
            [CmdAddr::new(SEGMENT_CONSTANT, 0)],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Eq,
            [CmdAddr::new(SEGMENT_IN_MIN, 1), CmdAddr::new(SEGMENT_CONSTANT, 1)],
            CmdAddr::new(SEGMENT_REGISTER, 1),
        ));
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_REGISTER, 1),
            CmdAddr::new(SEGMENT_CONSTANT, 2),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_REGISTER, 0), CmdAddr::new(SEGMENT_CONSTANT, 3)],
            CmdAddr::new(SEGMENT_REGISTER, 1),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Mov,
            [CmdAddr::new(SEGMENT_CONSTANT, 4)],
            CmdAddr::new(SEGMENT_REGISTER, 1),
        ));
        assert_eq!(
            blocks.timers(),
            &[ScriptBlock::new_timer(OnTimeout, 10.0, byte_code).unwrap()]
        );
    }

    #[test]
    fn test_generator_expr_call() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            before_hit {
                var v1 = 3 * (in.gg + 5.55)
                out.xx = math.abs((v1))
                out.xx = math.ceil(v1) - 4
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_IN_MIN + 1, 4), CmdAddr::new(SEGMENT_CONSTANT, 1)],
            CmdAddr::new(SEGMENT_REGISTER, 1),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Mul,
            [CmdAddr::new(SEGMENT_CONSTANT, 0), CmdAddr::new(SEGMENT_REGISTER, 1)],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Abs,
            [CmdAddr::new(SEGMENT_REGISTER, 0)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Ceil,
            [CmdAddr::new(SEGMENT_REGISTER, 0)],
            CmdAddr::new(SEGMENT_REGISTER, 1),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Sub,
            [CmdAddr::new(SEGMENT_REGISTER, 1), CmdAddr::new(SEGMENT_CONSTANT, 2)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        assert_eq!(
            blocks.hook(BeforeHit).unwrap(),
            &ScriptBlock::new_hook(BeforeHit, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_expr_branch_value() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            before_hit {
                var v1 = if in.aa { -1 } else { -2 }
            }
            after_hit {
                out.xx = if in.aa { in.aa + -3 } else { -2 }
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(
            blocks.constant_segment,
            vec![f64_u64(-1.0), f64_u64(-2.0), 9, f64_u64(-3.0)]
        );

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdCall::new(
            CmdOpt::IfElse1,
            [
                CmdAddr::new(SEGMENT_IN_MIN, 0),
                CmdAddr::new(SEGMENT_CONSTANT, 0),
                CmdAddr::new(SEGMENT_CONSTANT, 1),
            ],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        assert_eq!(
            blocks.hook(BeforeHit).unwrap(),
            &ScriptBlock::new_hook(BeforeHit, byte_code).unwrap()
        );

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas0,
            CmdAddr::new(SEGMENT_IN_MIN, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 2),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_IN_MIN, 0), CmdAddr::new(SEGMENT_CONSTANT, 3)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        assert_eq!(
            blocks.hook(AfterHit).unwrap(),
            &ScriptBlock::new_hook(AfterHit, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_expr_branch_no_value() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            on_finish {
                out.xx = if in.cc { in.cc ** -1 } elsif in.bb { -2 } else { -4 }
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(
            blocks.constant_segment,
            vec![7, 14, f64_u64(-1.0), f64_u64(-2.0), f64_u64(-4.0)]
        );

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_IN_MIN, 2),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Pow,
            [CmdAddr::new(SEGMENT_IN_MIN, 2), CmdAddr::new(SEGMENT_CONSTANT, 2)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        byte_code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 1)));
        byte_code.write(&CmdCall::new(
            CmdOpt::IfElse1,
            [
                CmdAddr::new(SEGMENT_IN_MIN, 1),
                CmdAddr::new(SEGMENT_CONSTANT, 3),
                CmdAddr::new(SEGMENT_CONSTANT, 4),
            ],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        assert_eq!(
            blocks.hook(OnFinish).unwrap(),
            &ScriptBlock::new_hook(OnFinish, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_expr_branch_to_expr_end() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            on_timeout(12) {
                out.xx = if in.bb {
                    100
                } else {
                    if in.cc { 1 + 2 } else { 25 }
                }
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(
            blocks.constant_segment,
            vec![f64_u64(100.0), 14, f64_u64(25.0), f64_u64(1.0), f64_u64(2.0)]
        );

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas1,
            CmdAddr::new(SEGMENT_IN_MIN, 1),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
        ));
        byte_code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas0,
            CmdAddr::new(SEGMENT_IN_MIN, 2),
            CmdAddr::new(SEGMENT_CONSTANT, 2),
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_CONSTANT, 3), CmdAddr::new(SEGMENT_CONSTANT, 4)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        assert_eq!(
            blocks.timers(),
            &[ScriptBlock::new_timer(OnTimeout, 12.0, byte_code).unwrap()]
        );
    }

    #[test]
    fn test_generator_expr_branch_to_stat_end() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            on_finish {
                if in.aa {
                    out.xx = if in.gg { 6 + in.aa } else { 5 - in.aa }
                }
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(blocks.constant_segment, vec![16, 10, f64_u64(6.0), f64_u64(5.0),]);

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_IN_MIN, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
        ));
        byte_code.write(&CmdJmpCmp::new(
            CmdAddr::new(SEGMENT_IN_MIN + 1, 4),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_CONSTANT, 2), CmdAddr::new(SEGMENT_IN_MIN, 0)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        byte_code.write(&CmdJmp::new(CmdAddr::new(SEGMENT_CONSTANT, 0)));
        byte_code.write(&CmdCall::new(
            CmdOpt::Sub,
            [CmdAddr::new(SEGMENT_CONSTANT, 3), CmdAddr::new(SEGMENT_IN_MIN, 0)],
            CmdAddr::new(SEGMENT_OUT_MIN, 0),
        ));
        assert_eq!(
            blocks.hook(OnFinish).unwrap(),
            &ScriptBlock::new_hook(OnFinish, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_expr_logic_basic() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            before_hit {
                var v1 = in.bb && (1 || 0)
                var v2 = in.cc || (1 && 0)
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(blocks.constant_segment, vec![10, f64_u64(1.0), f64_u64(0.0), 20]);

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas0,
            CmdAddr::new(SEGMENT_IN_MIN, 1),
            CmdAddr::new(SEGMENT_IN_MIN, 1),
            CmdAddr::new(SEGMENT_REGISTER, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::IfElse1,
            [
                CmdAddr::new(SEGMENT_CONSTANT, 1),
                CmdAddr::new(SEGMENT_CONSTANT, 1),
                CmdAddr::new(SEGMENT_CONSTANT, 2),
            ],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        byte_code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas1,
            CmdAddr::new(SEGMENT_IN_MIN, 2),
            CmdAddr::new(SEGMENT_IN_MIN, 2),
            CmdAddr::new(SEGMENT_REGISTER, 1),
            CmdAddr::new(SEGMENT_CONSTANT, 3),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::IfElse0,
            [
                CmdAddr::new(SEGMENT_CONSTANT, 1),
                CmdAddr::new(SEGMENT_CONSTANT, 1),
                CmdAddr::new(SEGMENT_CONSTANT, 2),
            ],
            CmdAddr::new(SEGMENT_REGISTER, 1),
        ));
        assert_eq!(
            blocks.hook(BeforeHit).unwrap(),
            &ScriptBlock::new_hook(BeforeHit, byte_code).unwrap()
        );
    }

    #[test]
    fn test_generator_expr_logic_to_end() {
        let mut parser = new_parser();
        let mut generator = ScriptGenerator::new();

        let code = r"
            before_hit {
                var tt = if in.bb {
                    in.aa || in.aa + 30
                } else {
                    100
                }
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        let blocks = generator.run(res).unwrap();

        assert_eq!(blocks.constant_segment, vec![f64_u64(100.0), 14, f64_u64(30.0),]);

        let mut byte_code = ScriptByteCode::default();
        byte_code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas0,
            CmdAddr::new(SEGMENT_IN_MIN, 1),
            CmdAddr::new(SEGMENT_CONSTANT, 0),
            CmdAddr::new(SEGMENT_REGISTER, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
        ));
        byte_code.write(&CmdJmpCas::new(
            CmdOpt::JmpCas1,
            CmdAddr::new(SEGMENT_IN_MIN, 0),
            CmdAddr::new(SEGMENT_IN_MIN, 0),
            CmdAddr::new(SEGMENT_REGISTER, 0),
            CmdAddr::new(SEGMENT_CONSTANT, 1),
        ));
        byte_code.write(&CmdCall::new(
            CmdOpt::Add,
            [CmdAddr::new(SEGMENT_IN_MIN, 0), CmdAddr::new(SEGMENT_CONSTANT, 2)],
            CmdAddr::new(SEGMENT_REGISTER, 0),
        ));
        assert_eq!(
            blocks.hook(BeforeHit).unwrap(),
            &ScriptBlock::new_hook(BeforeHit, byte_code).unwrap()
        );
    }
}
