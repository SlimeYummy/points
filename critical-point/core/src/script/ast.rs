use crate::script::command::{CmdAddr, CmdOpt, ScriptBlockType};
use crate::utils::Num;

#[derive(Debug, Clone, PartialEq)]
pub struct AstBlock {
    pub typ: ScriptBlockType,
    pub arg: Option<Num>,
    pub stats: Vec<AstStat>,
}

impl AstBlock {
    pub fn new_hook(typ: ScriptBlockType, stats: Vec<AstStat>) -> AstBlock {
        AstBlock { typ, arg: None, stats }
    }

    pub fn new_timer(typ: ScriptBlockType, arg: Num, stats: Vec<AstStat>) -> AstBlock {
        AstBlock {
            typ,
            arg: Some(arg),
            stats,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstStat {
    Assign(AstStatAssign),
    Call(AstStatCall),
    CallExt(AstStatCallExt),
    Branch(AstStatBranch),
    Return(AstStatReturn),
}

impl AstStat {
    pub fn new_assign(var: AstVar, expr: AstExpr) -> AstStat {
        AstStat::Assign(AstStatAssign::new(var, expr))
    }

    pub fn new_call(opt: CmdOpt, args: Vec<AstExpr>) -> AstStat {
        AstStat::Call(AstStatCall::new(opt, args))
    }

    pub fn new_call_ext(ext: u16, args: Vec<AstExpr>) -> AstStat {
        AstStat::CallExt(AstStatCallExt::new(ext, args))
    }

    pub fn new_branch(cond: Option<AstExpr>, stats: Vec<AstStat>, next: Option<AstStatBranch>) -> AstStat {
        AstStat::Branch(AstStatBranch::new(cond, stats, next))
    }

    pub fn new_return() -> AstStat {
        AstStat::Return(AstStatReturn::new())
    }

    pub fn into_branch(self) -> Option<AstStatBranch> {
        match self {
            AstStat::Branch(branch) => Some(branch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatAssign {
    pub var: AstVar,
    pub expr: Box<AstExpr>,
}

impl AstStatAssign {
    pub fn new(var: AstVar, expr: AstExpr) -> AstStatAssign {
        AstStatAssign {
            var,
            expr: Box::new(expr),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatCall {
    pub opt: CmdOpt,
    pub args: Vec<AstExpr>,
}

impl AstStatCall {
    pub fn new(opt: CmdOpt, args: Vec<AstExpr>) -> AstStatCall {
        AstStatCall { opt, args }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatCallExt {
    pub ext: u16,
    pub args: Vec<AstExpr>,
}

impl AstStatCallExt {
    pub fn new(ext: u16, args: Vec<AstExpr>) -> AstStatCallExt {
        AstStatCallExt { ext, args }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatBranch {
    pub cond: Option<Box<AstExpr>>,
    pub stats: Vec<AstStat>,
    pub next: Option<Box<AstStatBranch>>,
}

impl AstStatBranch {
    pub fn new(cond: Option<AstExpr>, stats: Vec<AstStat>, next: Option<AstStatBranch>) -> AstStatBranch {
        AstStatBranch {
            cond: cond.map(Box::new),
            stats,
            next: next.map(Box::new),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatReturn {}

impl AstStatReturn {
    pub fn new() -> AstStatReturn {
        AstStatReturn {}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstVar {
    Local(u32),
    Closure(CmdAddr),
    Output(CmdAddr),
}

impl AstVar {
    pub fn new_local(id: u32) -> AstVar {
        AstVar::Local(id)
    }

    pub fn new_output(addr: CmdAddr) -> AstVar {
        AstVar::Output(addr)
    }

    pub fn new_closure(addr: CmdAddr) -> AstVar {
        AstVar::Closure(addr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstExpr {
    Num(Num),
    Str(String),
    Local(u32),
    Closure(CmdAddr),
    Argument(CmdAddr),
    Input(CmdAddr),
    Output(CmdAddr),
    Call(AstExprCall),
    CallExt(AstExprCallExt),
    Branch(AstExprBranch),
    Logic(AstExprLogic),
}

impl AstExpr {
    pub fn new_num(num: Num) -> AstExpr {
        AstExpr::Num(num)
    }

    pub fn new_str(val: &str) -> AstExpr {
        AstExpr::Str(val.into())
    }

    pub fn new_local(ident: u32) -> AstExpr {
        AstExpr::Local(ident)
    }

    pub fn new_closure(addr: CmdAddr) -> AstExpr {
        AstExpr::Closure(addr)
    }

    pub fn new_argument(addr: CmdAddr) -> AstExpr {
        AstExpr::Argument(addr)
    }

    pub fn new_input(addr: CmdAddr) -> AstExpr {
        AstExpr::Input(addr)
    }

    pub fn new_output(addr: CmdAddr) -> AstExpr {
        AstExpr::Output(addr)
    }

    pub fn new_call(opt: CmdOpt, args: Vec<AstExpr>) -> AstExpr {
        AstExpr::Call(AstExprCall::new(opt, args))
    }

    pub fn new_call_ext(ext: u16, args: Vec<AstExpr>) -> AstExpr {
        AstExpr::CallExt(AstExprCallExt::new(ext, args))
    }

    pub fn new_branch(cond: AstExpr, left: AstExpr, right: AstExpr) -> AstExpr {
        AstExpr::Branch(AstExprBranch::new(cond, left, right))
    }

    pub fn new_logic(typ: AstLogicType, left: AstExpr, right: AstExpr) -> AstExpr {
        AstExpr::Logic(AstExprLogic::new(typ, left, right))
    }

    pub fn from_var(var: &AstVar) -> AstExpr {
        match var {
            AstVar::Local(ident) => AstExpr::new_local(*ident),
            AstVar::Closure(ident) => AstExpr::new_closure(*ident),
            AstVar::Output(ident) => AstExpr::new_output(*ident),
        }
    }

    pub fn as_num(&self) -> Option<Num> {
        match self {
            &AstExpr::Num(num) => Some(num),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&String> {
        match self {
            AstExpr::Str(str) => Some(str),
            _ => None,
        }
    }

    pub fn is_value(&self) -> bool {
        matches!(
            self,
            &AstExpr::Num(_)
                | &AstExpr::Str(_)
                | &AstExpr::Local(_)
                | &AstExpr::Closure(_)
                | &AstExpr::Argument(_)
                | &AstExpr::Input(_)
                | &AstExpr::Output(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprCall {
    pub opt: CmdOpt,
    pub args: Vec<AstExpr>,
}

impl AstExprCall {
    pub fn new(opt: CmdOpt, args: Vec<AstExpr>) -> AstExprCall {
        AstExprCall { opt, args }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprCallExt {
    pub ext: u16,
    pub args: Vec<AstExpr>,
}

impl AstExprCallExt {
    pub fn new(ext: u16, args: Vec<AstExpr>) -> AstExprCallExt {
        AstExprCallExt { ext, args }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprBranch {
    pub cond: Box<AstExpr>,
    pub left: Box<AstExpr>,
    pub right: Box<AstExpr>,
}

impl AstExprBranch {
    pub fn new(cond: AstExpr, left: AstExpr, right: AstExpr) -> AstExprBranch {
        AstExprBranch {
            cond: Box::new(cond),
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstLogicType {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprLogic {
    pub typ: AstLogicType,
    pub left: Box<AstExpr>,
    pub right: Box<AstExpr>,
}

impl AstExprLogic {
    pub fn new(typ: AstLogicType, left: AstExpr, right: AstExpr) -> AstExprLogic {
        AstExprLogic {
            typ,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}
