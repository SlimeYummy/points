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
        return AstBlock { typ, arg: None, stats };
    }

    pub fn new_timer(typ: ScriptBlockType, arg: Num, stats: Vec<AstStat>) -> AstBlock {
        return AstBlock {
            typ,
            arg: Some(arg),
            stats,
        };
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
        return AstStat::Assign(AstStatAssign::new(var, expr));
    }

    pub fn new_call(opt: CmdOpt, args: Vec<AstExpr>) -> AstStat {
        return AstStat::Call(AstStatCall::new(opt, args));
    }

    pub fn new_call_ext(ext: u16, args: Vec<AstExpr>) -> AstStat {
        return AstStat::CallExt(AstStatCallExt::new(ext, args));
    }

    pub fn new_branch(cond: Option<AstExpr>, stats: Vec<AstStat>, next: Option<AstStatBranch>) -> AstStat {
        return AstStat::Branch(AstStatBranch::new(cond, stats, next));
    }

    pub fn new_return() -> AstStat {
        return AstStat::Return(AstStatReturn::new());
    }

    pub fn into_branch(self) -> Option<AstStatBranch> {
        return match self {
            AstStat::Branch(branch) => Some(branch),
            _ => None,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatAssign {
    pub var: AstVar,
    pub expr: Box<AstExpr>,
}

impl AstStatAssign {
    pub fn new(var: AstVar, expr: AstExpr) -> AstStatAssign {
        return AstStatAssign {
            var,
            expr: Box::new(expr),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatCall {
    pub opt: CmdOpt,
    pub args: Vec<AstExpr>,
}

impl AstStatCall {
    pub fn new(opt: CmdOpt, args: Vec<AstExpr>) -> AstStatCall {
        return AstStatCall { opt, args };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatCallExt {
    pub ext: u16,
    pub args: Vec<AstExpr>,
}

impl AstStatCallExt {
    pub fn new(ext: u16, args: Vec<AstExpr>) -> AstStatCallExt {
        return AstStatCallExt { ext, args };
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
        return AstStatBranch {
            cond: cond.map(|c| Box::new(c)),
            stats,
            next: next.map(|n| Box::new(n)),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatReturn {}

impl AstStatReturn {
    pub fn new() -> AstStatReturn {
        return AstStatReturn {};
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
        return AstVar::Local(id);
    }

    pub fn new_output(addr: CmdAddr) -> AstVar {
        return AstVar::Output(addr);
    }

    pub fn new_closure(addr: CmdAddr) -> AstVar {
        return AstVar::Closure(addr);
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
        return AstExpr::Num(num);
    }

    pub fn new_str(val: &str) -> AstExpr {
        return AstExpr::Str(val.into());
    }

    pub fn new_local(ident: u32) -> AstExpr {
        return AstExpr::Local(ident);
    }

    pub fn new_closure(addr: CmdAddr) -> AstExpr {
        return AstExpr::Closure(addr);
    }

    pub fn new_argument(addr: CmdAddr) -> AstExpr {
        return AstExpr::Argument(addr);
    }

    pub fn new_input(addr: CmdAddr) -> AstExpr {
        return AstExpr::Input(addr);
    }

    pub fn new_output(addr: CmdAddr) -> AstExpr {
        return AstExpr::Output(addr);
    }

    pub fn new_call(opt: CmdOpt, args: Vec<AstExpr>) -> AstExpr {
        return AstExpr::Call(AstExprCall::new(opt, args));
    }

    pub fn new_call_ext(ext: u16, args: Vec<AstExpr>) -> AstExpr {
        return AstExpr::CallExt(AstExprCallExt::new(ext, args));
    }

    pub fn new_branch(cond: AstExpr, left: AstExpr, right: AstExpr) -> AstExpr {
        return AstExpr::Branch(AstExprBranch::new(cond, left, right));
    }

    pub fn new_logic(typ: AstLogicType, left: AstExpr, right: AstExpr) -> AstExpr {
        return AstExpr::Logic(AstExprLogic::new(typ, left, right));
    }

    pub fn from_var(var: &AstVar) -> AstExpr {
        return match var {
            AstVar::Local(ident) => AstExpr::new_local(ident.clone()),
            AstVar::Closure(ident) => AstExpr::new_closure(ident.clone()),
            AstVar::Output(ident) => AstExpr::new_output(ident.clone()),
        };
    }

    pub fn as_num(&self) -> Option<Num> {
        return match self {
            &AstExpr::Num(num) => Some(num),
            _ => None,
        };
    }

    pub fn as_str(&self) -> Option<&String> {
        return match self {
            &AstExpr::Str(ref str) => Some(str),
            _ => None,
        };
    }

    pub fn is_value(&self) -> bool {
        return matches!(
            self,
            &AstExpr::Num(_)
                | &AstExpr::Str(_)
                | &AstExpr::Local(_)
                | &AstExpr::Closure(_)
                | &AstExpr::Argument(_)
                | &AstExpr::Input(_)
                | &AstExpr::Output(_)
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprCall {
    pub opt: CmdOpt,
    pub args: Vec<AstExpr>,
}

impl AstExprCall {
    pub fn new(opt: CmdOpt, args: Vec<AstExpr>) -> AstExprCall {
        return AstExprCall { opt, args };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprCallExt {
    pub ext: u16,
    pub args: Vec<AstExpr>,
}

impl AstExprCallExt {
    pub fn new(ext: u16, args: Vec<AstExpr>) -> AstExprCallExt {
        return AstExprCallExt { ext, args };
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
        return AstExprBranch {
            cond: Box::new(cond),
            left: Box::new(left),
            right: Box::new(right),
        };
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
        return AstExprLogic {
            typ,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
}
