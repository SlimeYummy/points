use anyhow::Result;
use lazy_static::lazy_static;
use pest::error::{Error, ErrorVariant};
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::Parser as ParserTrait;
use pest_derive::Parser;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

use crate::script::ast::{AstBlock, AstExpr, AstLogicType, AstStat, AstVar};
use crate::script::builtin;
use crate::script::command::{CmdAddr, CmdOpt, CmdType, ScriptBlockType};
use crate::script::config::{
    MAX_CLOSURE, MAX_FUNCTION_ARGUMENTS, MAX_LOCAL, SEGMENT_CLOSURE, SEGMENT_IN_MAX, SEGMENT_IN_MIN, SEGMENT_OUT_MAX,
    SEGMENT_OUT_MIN,
};
use crate::script::utils::ScriptOutType;
use crate::utils::{Num, Symbol, XResult};

#[derive(Parser)]
#[grammar = "./script/script.pest"]
pub struct PestParser;

pub struct ScriptParser {
    pratt: Rc<PrattParser<Rule>>,
    idmgr: IdentManager,
}

pub struct ParserResult {
    pub blocks: Vec<AstBlock>,
    pub arguments: Vec<Symbol>,
    pub closure_inits: Vec<Num>,
}

pub type ScriptInputMap = HashMap<String, u16>;
pub type ScriptOutputMap = HashMap<String, (u16, ScriptOutType)>;

impl ScriptParser {
    pub fn new(
        global_consts: &HashMap<String, Num>,
        all_inputs: &HashMap<(ScriptBlockType, u8), ScriptInputMap>,
        all_outputs: &HashMap<(ScriptBlockType, u8), ScriptOutputMap>,
        all_funcs: &HashMap<ScriptBlockType, HashMap<String, (u16, Vec<CmdType>)>>,
    ) -> Result<ScriptParser> {
        use pest::pratt_parser::*;
        let pratt = PrattParser::new()
            .op(Op::infix(Rule::Or, Assoc::Left))
            .op(Op::infix(Rule::And, Assoc::Left))
            .op(Op::infix(Rule::Eq, Assoc::Left)
                | Op::infix(Rule::Ne, Assoc::Left)
                | Op::infix(Rule::Lt, Assoc::Left)
                | Op::infix(Rule::Le, Assoc::Left)
                | Op::infix(Rule::Gt, Assoc::Left)
                | Op::infix(Rule::Ge, Assoc::Left))
            .op(Op::infix(Rule::Add, Assoc::Left) | Op::infix(Rule::Sub, Assoc::Left))
            .op(Op::infix(Rule::Mul, Assoc::Left)
                | Op::infix(Rule::Div, Assoc::Left)
                | Op::infix(Rule::Mod, Assoc::Left))
            .op(Op::infix(Rule::Pow, Assoc::Right))
            .op(Op::prefix(Rule::Pos) | Op::prefix(Rule::Neg) | Op::prefix(Rule::Not));

        let mut idmgr = IdentManager::new();
        idmgr
            .add_consts(&builtin::consts())
            .map_err(|msg| anyhow::anyhow!(msg))?;
        idmgr
            .add_funcs(&builtin::functions())
            .map_err(|msg| anyhow::anyhow!(msg))?;
        idmgr.add_consts(global_consts).map_err(|msg| anyhow::anyhow!(msg))?;
        idmgr.add_inputs(all_inputs).map_err(|msg| anyhow::anyhow!(msg))?;
        idmgr.add_outputs(all_outputs).map_err(|msg| anyhow::anyhow!(msg))?;
        idmgr.add_func_exts(all_funcs).map_err(|msg| anyhow::anyhow!(msg))?;
        idmgr.sync_block_path().map_err(|msg| anyhow::anyhow!(msg))?;

        Ok(ScriptParser {
            pratt: Rc::new(pratt),
            idmgr,
        })
    }

    pub fn run(&mut self, code: &str, args: &[&str]) -> Result<ParserResult> {
        self.idmgr.reset();
        self.idmgr.add_arguments(args).map_err(|msg| anyhow::anyhow!(msg))?;

        let mut pairs = PestParser::parse(Rule::Script, code)?;
        let script_pair = pairs.next().expect("Unexpected token");
        let script_pairs = script_pair.clone().into_inner();

        let mut block_pairs = Vec::new();
        for pair in script_pairs {
            match pair.as_rule() {
                Rule::OutConst => self.parse_const(pair)?,
                Rule::OutVar => self.parse_closure(pair)?,
                Rule::Block => block_pairs.push(pair),
                Rule::EOI => {}
                _ => return Err(Self::err(&pair, "Invalid token")),
            }
        }

        let mut blocks = Vec::<AstBlock>::new();
        for pair in block_pairs {
            let block = self.parse_block(pair.clone())?;
            if block.typ.is_hook() && blocks.iter().any(|item| item.typ == block.typ) {
                return Err(Self::err(&pair, "Duplicate hook block"));
            }
            blocks.push(block);
        }

        Ok(ParserResult {
            blocks,
            arguments: self.idmgr.arguments()?,
            closure_inits: self.idmgr.closure_inits(),
        })
    }

    fn parse_const(&mut self, pair: Pair<'_, Rule>) -> Result<()> {
        let mut pairs = pair.clone().into_inner();
        let word_pair = Self::next(&mut pairs, &pair)?;
        let val_pair = Self::next(&mut pairs, &pair)?;

        match val_pair.as_rule() {
            Rule::Hex | Rule::Percent | Rule::Time | Rule::Float => {
                let num = self.parse_number(val_pair)?.as_num().unwrap();
                Self::map_err(self.idmgr.add_number(word_pair.as_str(), num), &word_pair)?;
            }
            Rule::String => {
                let str = self.parse_string(val_pair)?.as_str().unwrap().into();
                Self::map_err(self.idmgr.add_string(word_pair.as_str(), str), &word_pair)?;
            }
            _ => return Err(Self::err(&val_pair, "Invalid token")),
        };
        Ok(())
    }

    fn parse_closure(&mut self, pair: Pair<'_, Rule>) -> Result<()> {
        let mut pairs = pair.clone().into_inner();
        let word_pair = Self::next(&mut pairs, &pair)?;
        let num_pair = Self::next(&mut pairs, &pair)?;

        let num = self.parse_number(num_pair)?.as_num().unwrap();
        Self::map_err(self.idmgr.add_closure(word_pair.as_str(), num), &word_pair)?;
        Ok(())
    }

    fn parse_block(&mut self, pair: Pair<'_, Rule>) -> Result<AstBlock> {
        let mut pairs = pair.clone().into_inner();
        let type_pair = Self::next(&mut pairs, &pair)?;

        use ScriptBlockType::*;
        let (typ, time) = match type_pair.as_rule() {
            Rule::OnAssemble => (OnAssemble, None),
            Rule::AfterAssemble => (AfterAssemble, None),
            Rule::OnStart => (OnStart, None),
            Rule::OnFinish => (OnFinish, None),
            Rule::BeforeHit => (BeforeHit, None),
            Rule::AfterHit => (AfterHit, None),
            Rule::BeforeInjure => (BeforeInjure, None),
            Rule::AfterInjure => (AfterInjure, None),
            Rule::OnTreat => (OnTreat, None),
            Rule::OnTimeout => {
                let time_pair = Self::next(&mut pairs, &type_pair)?;
                let time = match time_pair.as_rule() {
                    Rule::Time => self.parse_time(time_pair)?.as_num().unwrap(),
                    Rule::Float => self.parse_float(time_pair)?.as_num().unwrap(),
                    _ => return Err(Self::err(&type_pair, "Invalid block")),
                };
                (OnTimeout, Some(time))
            }
            Rule::OnInterval => {
                let time_pair = Self::next(&mut pairs, &type_pair)?;
                let time = match time_pair.as_rule() {
                    Rule::Time => self.parse_time(time_pair)?.as_num().unwrap(),
                    Rule::Float => self.parse_float(time_pair)?.as_num().unwrap(),
                    _ => return Err(Self::err(&type_pair, "Invalid block")),
                };
                (OnInterval, Some(time))
            }
            _ => return Err(Self::err(&type_pair, "Invalid block")),
        };

        self.idmgr.push_scope(Some(typ));
        let mut stats = Vec::new();
        for pair in pairs {
            if let Some(stat) = self.parse_stat(pair)? {
                stats.push(stat);
            }
        }
        self.idmgr.pop_scope();

        if let Some(time) = time {
            Ok(AstBlock::new_timer(typ, time, stats))
        } else {
            Ok(AstBlock::new_hook(typ, stats))
        }
    }

    fn parse_stat(&mut self, pair: Pair<'_, Rule>) -> Result<Option<AstStat>> {
        let stat = match pair.as_rule() {
            Rule::InConst => {
                self.parse_const(pair)?;
                return Ok(None);
            }
            Rule::InVar => self.parse_local(pair)?,
            Rule::Assign => self.parse_assign(pair)?,
            Rule::CallStat => self.parse_call_stat(pair)?,
            Rule::IfStat => self.parse_if_stat(pair)?,
            Rule::Return => self.parse_return_stat(pair)?,
            _ => return Err(Self::err(&pair, "Invalid statment")),
        };
        Ok(Some(stat))
    }

    fn parse_local(&mut self, pair: Pair<'_, Rule>) -> Result<AstStat> {
        let mut pairs = pair.clone().into_inner();
        let word_pair = Self::next(&mut pairs, &pair)?;
        let num_pair = Self::next(&mut pairs, &pair)?;

        let ident = word_pair.as_str();
        let id = Self::map_err(self.idmgr.add_local(ident), &word_pair)?;
        Ok(AstStat::new_assign(AstVar::new_local(id), self.parse_expr(num_pair)?))
    }

    fn parse_assign(&mut self, pair: Pair<'_, Rule>) -> Result<AstStat> {
        let mut pairs = pair.clone().into_inner();
        let var_pair = Self::next(&mut pairs, &pair)?;
        let assign_pair = Self::next(&mut pairs, &pair)?;
        let expr_pair = Self::next(&mut pairs, &pair)?;

        let opt = match assign_pair.as_rule() {
            Rule::RawAssign => CmdOpt::Mov,
            Rule::AddAssign => CmdOpt::Add,
            Rule::SubAssign => CmdOpt::Sub,
            Rule::MulAssign => CmdOpt::Mul,
            Rule::DivAssign => CmdOpt::Div,
            _ => return Err(Self::err(&assign_pair, "Invalid assign")),
        };

        let var = match self.idmgr.get(var_pair.as_str()) {
            Some(IdentType::Local(id)) => AstVar::new_local(*id),
            Some(IdentType::Closure(addr)) => AstVar::new_closure(*addr),
            Some(IdentType::Output(addr, out_type)) => {
                if !out_type.check_opt(opt) {
                    return Err(Self::err(&assign_pair, "Assign not support"));
                }
                AstVar::new_output(*addr)
            }
            _ => return Err(Self::err(&var_pair, "Ident not found")),
        };

        let expr = self.parse_expr(expr_pair)?;
        if opt == CmdOpt::Mov {
            Ok(AstStat::new_assign(var, expr))
        } else {
            let val_expr = AstExpr::new_call(opt, vec![AstExpr::from_var(&var), expr]);
            Ok(AstStat::new_assign(var, val_expr))
        }
    }

    fn parse_call_stat(&mut self, pair: Pair<'_, Rule>) -> Result<AstStat> {
        match self.parse_call_impl(pair)? {
            (Some(opt), None, args) => Ok(AstStat::new_call(opt, args)),
            (None, Some(ext), args) => Ok(AstStat::new_call_ext(ext, args)),
            _ => unreachable!(),
        }
    }

    fn parse_if_stat(&mut self, pair: Pair<'_, Rule>) -> Result<AstStat> {
        self.idmgr.push_scope(None);

        let mut pairs = pair.clone().into_inner();

        let mut cond = None;
        if pair.as_rule() != Rule::ElseStat {
            let cond_pair = Self::next(&mut pairs, &pair)?;
            cond = Some(self.parse_expr(cond_pair)?);
        }

        let mut stats = Vec::new();
        let mut next_pair = None;
        for iter_pair in pairs {
            match iter_pair.as_rule() {
                Rule::InConst | Rule::InVar | Rule::Assign | Rule::CallStat | Rule::IfStat | Rule::Return => {
                    if let Some(stat) = self.parse_stat(iter_pair)? {
                        stats.push(stat);
                    }
                }
                Rule::ElsifStat | Rule::ElseStat => {
                    next_pair = Some(iter_pair);
                    break;
                }
                _ => return Err(Self::err(&pair, "Invalid if-elsif-else statment")),
            };
        }

        self.idmgr.pop_scope();
        let mut next = None;
        if let Some(next_pair) = next_pair {
            next = self.parse_if_stat(next_pair)?.into_branch();
        }

        Ok(AstStat::new_branch(cond, stats, next))
    }

    fn parse_return_stat(&mut self, _: Pair<'_, Rule>) -> Result<AstStat> {
        Ok(AstStat::new_return())
    }

    fn parse_expr(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let pratt = self.pratt.clone();
        return pratt
            .map_primary(|primary| match primary.as_rule() {
                Rule::Expr => self.parse_expr(primary),
                Rule::Group => self.parse_expr(primary),
                Rule::IfExpr => self.parse_if_expr(primary),
                Rule::CallExpr => self.parse_call_expr(primary),
                Rule::Ident => self.parse_ident(primary),
                Rule::Hex => self.parse_hex(primary),
                Rule::Float => self.parse_float(primary),
                Rule::Time => self.parse_time(primary),
                Rule::Percent => self.parse_percent(primary),
                _ => Err(Self::err(&pair, "Invalid Expression")),
            })
            .map_prefix(|op, rsh| match op.as_rule() {
                Rule::Pos => rsh,
                Rule::Neg => Ok(AstExpr::new_call(CmdOpt::Neg, vec![rsh?])),
                Rule::Not => Ok(AstExpr::new_call(CmdOpt::Not, vec![rsh?])),
                _ => Err(Self::err(&pair, "Invalid Expression")),
            })
            .map_infix(|lhs, op, rhs| match op.as_rule() {
                Rule::Add => Ok(AstExpr::new_call(CmdOpt::Add, vec![lhs?, rhs?])),
                Rule::Sub => Ok(AstExpr::new_call(CmdOpt::Sub, vec![lhs?, rhs?])),
                Rule::Mul => Ok(AstExpr::new_call(CmdOpt::Mul, vec![lhs?, rhs?])),
                Rule::Pow => Ok(AstExpr::new_call(CmdOpt::Pow, vec![lhs?, rhs?])),
                Rule::Div => Ok(AstExpr::new_call(CmdOpt::Div, vec![lhs?, rhs?])),
                Rule::Mod => Ok(AstExpr::new_call(CmdOpt::Mod, vec![lhs?, rhs?])),
                Rule::Le => Ok(AstExpr::new_call(CmdOpt::Le, vec![lhs?, rhs?])),
                Rule::Lt => Ok(AstExpr::new_call(CmdOpt::Lt, vec![lhs?, rhs?])),
                Rule::Ge => Ok(AstExpr::new_call(CmdOpt::Ge, vec![lhs?, rhs?])),
                Rule::Gt => Ok(AstExpr::new_call(CmdOpt::Gt, vec![lhs?, rhs?])),
                Rule::Eq => Ok(AstExpr::new_call(CmdOpt::Eq, vec![lhs?, rhs?])),
                Rule::Ne => Ok(AstExpr::new_call(CmdOpt::Ne, vec![lhs?, rhs?])),
                Rule::And => Ok(AstExpr::new_logic(AstLogicType::And, lhs?, rhs?)),
                Rule::Or => Ok(AstExpr::new_logic(AstLogicType::Or, lhs?, rhs?)),
                _ => Err(Self::err(&pair, "Invalid Expression")),
            })
            .parse(pair.clone().into_inner());
    }

    fn parse_call_expr(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        match self.parse_call_impl(pair)? {
            (Some(opt), None, args) => Ok(AstExpr::new_call(opt, args)),
            (None, Some(ext), args) => Ok(AstExpr::new_call_ext(ext, args)),
            _ => unreachable!(),
        }
    }

    fn parse_if_expr(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let mut pairs = pair.clone().into_inner();

        let cond_pair = Self::next(&mut pairs, &pair)?;
        let cond = self.parse_expr(cond_pair)?;

        let expr_pair = Self::next(&mut pairs, &pair)?;
        let left = self.parse_expr(expr_pair)?;

        let right;
        let next_pair = Self::next(&mut pairs, &pair)?;
        if next_pair.as_rule() == Rule::ElseExpr {
            let mut else_pairs = next_pair.clone().into_inner();
            let else_expr_pair = Self::next(&mut else_pairs, &next_pair)?;
            right = self.parse_expr(else_expr_pair)?;
        } else {
            right = self.parse_if_expr(next_pair)?;
        }

        Ok(AstExpr::new_branch(cond, left, right))
    }

    fn parse_ident(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let expr = match self.idmgr.get(pair.as_str()) {
            Some(IdentType::Number(num)) => AstExpr::new_num(*num),
            Some(IdentType::Local(id)) => AstExpr::new_local(*id),
            Some(IdentType::Closure(addr)) => AstExpr::new_closure(*addr),
            Some(IdentType::Argument(addr)) => AstExpr::new_argument(*addr),
            Some(IdentType::Input(addr)) => AstExpr::new_input(*addr),
            _ => return Err(Self::err(&pair, "Ident not found")),
        };
        Ok(expr)
    }

    fn parse_number(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        match pair.as_rule() {
            Rule::Hex => self.parse_hex(pair),
            Rule::Float => self.parse_float(pair),
            Rule::Time => self.parse_time(pair),
            Rule::Percent => self.parse_percent(pair),
            _ => Err(Self::err(&pair, "Invalid number")),
        }
    }

    fn parse_hex(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let num = i64::from_str_radix(&pair.as_str()[2..], 16).map_err(|_| Self::err(&pair, "Invalid number"))?;
        Ok(AstExpr::new_num(num as Num))
    }

    fn parse_float(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let num = pair
            .as_str()
            .parse::<f64>()
            .map_err(|_| Self::err(&pair, "Invalid number"))?;
        Ok(AstExpr::new_num(num))
    }

    fn parse_time(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let time_str = pair.as_str();
        let num = match time_str.chars().last() {
            Some('s') => time_str[..time_str.len() - 1].parse::<Num>().map(|n| n * 20.0),
            Some('m') => time_str[..time_str.len() - 1].parse::<Num>().map(|n| n * 20.0 * 60.0),
            Some('h') => time_str[..time_str.len() - 1]
                .parse::<Num>()
                .map(|n| n * 20.0 * 60.0 * 60.0),
            _ => unreachable!(),
        }
        .map_err(|_| Self::err(&pair, "Invalid time"))?;
        Ok(AstExpr::new_num(num))
    }

    fn parse_percent(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let percent_str = pair.as_str();
        let num = percent_str[..percent_str.len() - 1]
            .parse::<Num>()
            .map(|n| n / 100.0)
            .map_err(|_| Self::err(&pair, "Invalid percent"))?;
        Ok(AstExpr::new_num(num))
    }

    fn parse_string(&mut self, pair: Pair<'_, Rule>) -> Result<AstExpr> {
        let str = pair.as_str();
        Ok(AstExpr::new_str(&str[1..str.len() - 1]))
    }

    //
    // impl
    //

    fn parse_call_impl(&mut self, pair: Pair<'_, Rule>) -> Result<(Option<CmdOpt>, Option<u16>, Vec<AstExpr>)> {
        let mut pairs = pair.clone().into_inner();

        let ident_pair = Self::next(&mut pairs, &pair)?;
        let (opt, ext, args) = match self.idmgr.get(ident_pair.as_str()) {
            Some(IdentType::Function(opt, args)) => (Some(*opt), None, args.clone()),
            Some(IdentType::FunctionExt(ext, args)) => (None, Some(*ext), args.clone()),
            _ => return Err(Self::err(&pair, "Function not found")),
        };

        let mut exprs = Vec::with_capacity(args.len());
        for typ in args {
            match typ {
                CmdType::Str => {
                    let str_pair = Self::next(&mut pairs, &pair)?;
                    if str_pair.as_rule() == Rule::String {
                        let str = str_pair.as_str();
                        exprs.push(AstExpr::new_str(&str[1..str.len() - 1]));
                    } else if str_pair.as_rule() != Rule::Ident {
                        match self.idmgr.get(str_pair.as_str()) {
                            Some(IdentType::String(val)) => exprs.push(AstExpr::new_str(val)),
                            _ => return Err(Self::err(&pair, "Invalid string")),
                        }
                    }
                }
                CmdType::Num => {
                    let expr_pair = Self::next(&mut pairs, &pair)?;
                    exprs.push(self.parse_expr(expr_pair)?);
                }
            };
        }

        Ok((opt, ext, exprs))
    }

    //
    // utils
    //

    fn err(pair: &Pair<'_, Rule>, msg: &str) -> anyhow::Error {
        let rule_err = Error::<Rule>::new_from_span(ErrorVariant::CustomError { message: msg.into() }, pair.as_span());
        anyhow::Error::from(rule_err)
    }

    fn map_err<T>(err: Result<T, String>, pair: &Pair<'_, Rule>) -> Result<T> {
        return err.map_err(|message| {
            let rule_err = Error::<Rule>::new_from_span(ErrorVariant::CustomError { message }, pair.as_span());
            anyhow::Error::from(rule_err)
        });
    }

    fn next<'t>(pairs: &mut Pairs<'t, Rule>, pair: &Pair<'_, Rule>) -> Result<Pair<'t, Rule>> {
        pairs.next().ok_or(Self::err(pair, "Invalid token end"))
    }
}

#[derive(Debug, Clone)]
enum IdentType {
    Path,
    Number(Num),
    String(String),
    Local(u32),
    Closure(CmdAddr),
    Argument(CmdAddr),
    Input(CmdAddr),
    Output(CmdAddr, ScriptOutType),
    Function(CmdOpt, Vec<CmdType>),
    FunctionExt(u16, Vec<CmdType>),
}

lazy_static! {
    static ref KEYWORDS: HashSet<&'static str> = HashSet::from(["const", "var", "if", "elsif", "else", "return"]);
    static ref RE_WORDS: Regex = Regex::new(r"^(?:\$|[a-zA-Z_][[:word:]]*)(?:\.[a-zA-Z_][[:word:]]*)*$").unwrap();
    static ref RE_WORDS_2: Regex = Regex::new(r"^(?:\$|[a-zA-Z_][[:word:]]*)(?:\.[a-zA-Z_][[:word:]]*)+$").unwrap();
    static ref RE_WORD: Regex = Regex::new(r"^[a-zA-Z_][[:word:]]*$").unwrap();
}

#[derive(Debug, Default)]
struct IdentManager {
    idents: IdentMap,
    block_idents: HashMap<ScriptBlockType, IdentMap>,
    arguments: Vec<String>,
    closures: Vec<(String, Num)>,
    temp_consts: Vec<(String, u32)>,
    locals: Vec<(String, u32)>,
    local_counter: u32,
    scope_depth: u32,
    current_block: Option<ScriptBlockType>,
}

impl IdentManager {
    fn new() -> IdentManager {
        let mut imgr = IdentManager::default();
        for typ in enum_iterator::all() {
            imgr.block_idents.insert(typ, IdentMap::default());
        }
        imgr.idents.add_paths("A").unwrap();
        imgr
    }

    fn add_consts(&mut self, consts: &HashMap<String, Num>) -> Result<(), String> {
        for (ident, value) in consts {
            if !RE_WORDS.is_match(ident) {
                return Err(format!("Invaild ident: {}", ident));
            }
            self.idents.add_paths(ident)?;
            self.idents.add_ident(ident, IdentType::Number(*value))?;
        }
        Ok(())
    }

    fn add_funcs(&mut self, funcs: &HashMap<String, (CmdOpt, Vec<CmdType>)>) -> Result<(), String> {
        for (ident, (opt, args)) in funcs.iter() {
            if !RE_WORDS.is_match(ident) {
                return Err(format!("Invaild ident: {}", ident));
            }
            if args.len() > MAX_FUNCTION_ARGUMENTS {
                return Err(format!("Too many function arguments: {}", ident));
            }
            self.idents.add_paths(ident)?;
            self.idents.add_ident(ident, IdentType::Function(*opt, args.clone()))?;
        }
        Ok(())
    }

    fn add_inputs(&mut self, all_inputs: &HashMap<(ScriptBlockType, u8), ScriptInputMap>) -> Result<(), String> {
        for ((block, segment), inputs) in all_inputs.iter() {
            let block_idents = self.block_idents.get_mut(block).unwrap();

            if *segment < SEGMENT_IN_MIN || SEGMENT_IN_MAX < *segment {
                return Err(format!("Invaild in segment: {}", segment));
            }

            for (ident, offset) in inputs.iter() {
                if !RE_WORDS_2.is_match(ident) {
                    return Err(format!("Invaild ident: {}", ident));
                }
                if !self.idents.check(ident) {
                    return Err(format!("Ident conflict: {}", ident));
                }
                block_idents.add_paths(ident)?;
                block_idents.add_ident(ident, IdentType::Input(CmdAddr::new(*segment, *offset)))?;
            }
        }
        Ok(())
    }

    fn add_outputs(&mut self, all_outputs: &HashMap<(ScriptBlockType, u8), ScriptOutputMap>) -> Result<(), String> {
        for ((block, segment), outputs) in all_outputs.iter() {
            let block_idents = self.block_idents.get_mut(block).unwrap();

            if *segment < SEGMENT_OUT_MIN || SEGMENT_OUT_MAX < *segment {
                return Err(format!("Invaild out segment: {}", segment));
            }

            for (ident, (offset, out_type)) in outputs.iter() {
                if !RE_WORDS_2.is_match(ident) {
                    return Err(format!("Invaild ident: {}", ident));
                }
                if !self.idents.check(ident) {
                    return Err(format!("Ident conflict: {}", ident));
                }
                block_idents.add_paths(ident)?;
                block_idents.add_ident(ident, IdentType::Output(CmdAddr::new(*segment, *offset), *out_type))?;
            }
        }
        Ok(())
    }

    fn add_func_exts(
        &mut self,
        all_funcs: &HashMap<ScriptBlockType, HashMap<String, (u16, Vec<CmdType>)>>,
    ) -> Result<(), String> {
        for (block, funcs) in all_funcs.iter() {
            let block_idents = self.block_idents.get_mut(block).unwrap();

            for (ident, (ext, args)) in funcs.iter() {
                if !RE_WORDS_2.is_match(ident) {
                    return Err(format!("Invaild ident: {}", ident));
                }
                if !self.idents.check(ident) {
                    return Err(format!("Ident conflict: {}", ident));
                }
                if args.len() > MAX_FUNCTION_ARGUMENTS {
                    return Err(format!("Too many arguments: {}", ident));
                }
                block_idents.add_paths(ident)?;
                block_idents.add_ident(ident, IdentType::FunctionExt(*ext, args.clone()))?;
            }
        }
        Ok(())
    }

    fn sync_block_path(&mut self) -> Result<(), String> {
        for block_idents in self.block_idents.values() {
            for (ident, typ) in block_idents.idents.iter() {
                if let IdentType::Path = typ {
                    if self.idents.get(ident).is_none() {
                        self.idents.add_ident(ident, IdentType::Path)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn add_number(&mut self, ident: &str, value: Num) -> Result<(), String> {
        self.idents.add_ident(ident, IdentType::Number(value))?;
        self.temp_consts.push((ident.into(), self.scope_depth));
        Ok(())
    }

    fn add_string(&mut self, ident: &str, value: String) -> Result<(), String> {
        self.idents.add_ident(ident, IdentType::String(value))?;
        self.temp_consts.push((ident.into(), self.scope_depth));
        Ok(())
    }

    fn add_arguments(&mut self, args: &[&str]) -> Result<(), String> {
        for ident in args.iter() {
            if !RE_WORD.is_match(ident) {
                return Err(format!("Invaild argument: {}", ident));
            }
            if self.arguments.len() + self.closures.len() >= MAX_CLOSURE {
                return Err("Too many arguments and self".into());
            }

            let arg_ident = format!("A.{}", ident);
            let addr = CmdAddr::new(SEGMENT_CLOSURE, self.arguments.len() as u16);
            self.idents.add_ident(&arg_ident, IdentType::Argument(addr))?;
            self.arguments.push(arg_ident.clone());
        }
        Ok(())
    }

    fn add_closure(&mut self, ident: &str, init: Num) -> Result<(), String> {
        if self.arguments.len() + self.closures.len() >= MAX_CLOSURE {
            return Err("Too many arguments and closures".into());
        }
        let addr = CmdAddr::new(SEGMENT_CLOSURE, (self.arguments.len() + self.closures.len()) as u16);
        self.idents.add_ident(ident, IdentType::Closure(addr))?;
        self.closures.push((ident.into(), init));
        Ok(())
    }

    fn add_local(&mut self, ident: &str) -> Result<u32, String> {
        if self.locals.len() >= MAX_LOCAL {
            return Err("Too many locals".into());
        }
        self.local_counter += 1;
        self.idents.add_ident(ident, IdentType::Local(self.local_counter))?;
        self.locals.push((ident.into(), self.scope_depth));
        Ok(self.local_counter)
    }

    fn get(&self, ident: &str) -> Option<&IdentType> {
        if let Some(typ) = self.current_block {
            if let Some(block_idents) = self.block_idents.get(&typ) {
                if let Some(meta) = block_idents.get(ident) {
                    return Some(meta);
                }
            }
        }
        return match self.idents.get(ident) {
            Some(meta) => Some(meta),
            _ => None,
        };
    }

    fn arguments(&self) -> XResult<Vec<Symbol>> {
        // remove prefix "A."
        return self.arguments.iter().map(|arg| Symbol::try_from(&arg[2..])).collect();
    }

    fn closure_inits(&self) -> Vec<Num> {
        return self.closures.iter().map(|(_, init)| *init).collect();
    }

    fn push_scope(&mut self, block: Option<ScriptBlockType>) {
        self.scope_depth += 1;
        if self.current_block.is_none() {
            self.current_block = block;
        }
    }

    fn pop_scope(&mut self) {
        self.scope_depth -= 1;
        if self.scope_depth <= 0 {
            self.scope_depth = 0;
            self.current_block = None;
        }

        while let Some(last) = self.locals.last() {
            if last.1 <= self.scope_depth {
                break;
            }
            self.idents.remove(&last.0);
            self.locals.pop();
        }
        while let Some(last) = self.temp_consts.last() {
            if last.1 <= self.scope_depth {
                break;
            }
            self.idents.remove(&last.0);
            self.temp_consts.pop();
        }
    }

    fn reset(&mut self) {
        self.local_counter = 0;
        self.scope_depth = 0;
        self.current_block = None;

        for (key, _) in &self.locals {
            self.idents.remove(key);
        }
        self.locals.clear();

        for (key, _) in &self.temp_consts {
            self.idents.remove(key);
        }
        self.temp_consts.clear();

        for key in &self.arguments {
            self.idents.remove(key);
        }
        self.arguments.clear();

        for (key, _) in &self.closures {
            self.idents.remove(key);
        }
        self.closures.clear();
    }
}

#[derive(Debug, Default)]
struct IdentMap {
    idents: HashMap<String, IdentType>,
}

impl IdentMap {
    fn add_paths(&mut self, ident: &str) -> Result<(), String> {
        let mut last = 0;
        for (idx, _) in ident.match_indices('.') {
            if KEYWORDS.contains(&ident[last..idx]) {
                return Err("Unexpected keyword".into());
            }
            last = idx + 1;
            match self.idents.get(&ident[..idx]) {
                Some(IdentType::Path) => {}
                None => {
                    self.idents.insert(ident[..idx].into(), IdentType::Path);
                }
                _ => return Err(format!("Ident conflict: {}", ident)),
            }
        }
        if KEYWORDS.contains(&ident[last..]) {
            return Err("Unexpected keyword".into());
        }
        Ok(())
    }

    fn add_ident(&mut self, ident: &str, meta: IdentType) -> Result<(), String> {
        if KEYWORDS.contains(&ident) {
            return Err("Unexpected keyword".into());
        }
        if self.idents.contains_key(ident) {
            return Err(format!("Ident conflict: {}", ident));
        }
        self.idents.insert(ident.into(), meta);
        Ok(())
    }

    fn get(&self, ident: &str) -> Option<&IdentType> {
        return match self.idents.get(ident) {
            Some(meta) => Some(meta),
            _ => None,
        };
    }

    fn check(&self, ident: &str) -> bool {
        for (idx, _) in ident.match_indices('.') {
            match self.idents.get(&ident[..idx]) {
                None => return true,
                _ => return false,
            }
        }
        !self.idents.contains_key(ident)
    }

    fn remove(&mut self, ident: &str) {
        self.idents.remove(ident);
    }
}

#[cfg(test)]
mod tests {
    use ScriptBlockType::*;

    use super::*;
    use crate::script::ast::AstStatBranch;
    use crate::script::test::*;

    #[test]
    fn test_ident_manager_inner() {
        let mut idmgr = IdentManager::new();

        assert!(idmgr.add_consts(&HashMap::new()).is_ok());
        assert!(idmgr.add_consts(&builtin::consts()).is_ok());
        assert!(idmgr
            .add_consts(&HashMap::from([
                ("X".into(), 1.0),
                ("B.C".into(), 2.0),
                ("_.D.E".into(), 2.0),
            ]))
            .is_ok());
        assert!(idmgr.add_consts(&HashMap::from([("X.X".into(), 1.0)])).is_err());
        assert!(idmgr.add_consts(&HashMap::from([("B".into(), 1.0)])).is_err());
        assert!(idmgr.add_consts(&HashMap::from([("var".into(), 1.0)])).is_err());
        assert!(idmgr.add_consts(&HashMap::from([("B.var".into(), 1.0)])).is_err());

        use CmdOpt::*;
        use CmdType::*;
        assert!(idmgr.add_funcs(&HashMap::new()).is_ok());
        assert!(idmgr.add_funcs(&builtin::functions()).is_ok());
        assert!(idmgr
            .add_funcs(&HashMap::from([("X".into(), (Add, vec![Num, Num])),]))
            .is_err());
    }

    #[test]
    fn test_ident_manager_outer() {
        let mut idmgr = IdentManager::new();
        assert!(idmgr.add_funcs(&builtin::functions()).is_ok());

        let mut inputs = HashMap::new();
        inputs.insert((OnAssemble, SEGMENT_IN_MIN - 1), HashMap::new());
        assert!(idmgr.add_inputs(&inputs).is_err());

        let mut inputs = HashMap::new();
        inputs.insert(
            (OnAssemble, SEGMENT_IN_MIN),
            HashMap::from([("inx.x".into(), 0), ("in.a".into(), 0)]),
        );
        inputs.insert(
            (AfterAssemble, SEGMENT_IN_MIN),
            HashMap::from([("inx.x".into(), 0), ("in.a".into(), 0)]),
        );
        assert!(idmgr.add_inputs(&inputs).is_ok());

        let mut inputs = HashMap::new();
        inputs.insert((OnAssemble, SEGMENT_IN_MIN), HashMap::from([("inx.x".into(), 0)]));
        assert!(idmgr.add_inputs(&inputs).is_err());

        inputs.insert((OnAssemble, SEGMENT_IN_MIN), HashMap::from([("math.abs".into(), 0)]));
        assert!(idmgr.add_inputs(&inputs).is_err());

        let mut outputs = HashMap::new();
        outputs.insert((OnAssemble, SEGMENT_OUT_MAX + 2), HashMap::new());
        assert!(idmgr.add_outputs(&outputs).is_err());

        let mut outputs = HashMap::new();
        outputs.insert(
            (OnAssemble, SEGMENT_OUT_MIN),
            HashMap::from([("aa.out".into(), (0, ScriptOutType::Mov))]),
        );
        assert!(idmgr.add_outputs(&outputs).is_ok());

        let mut outputs = HashMap::new();
        outputs.insert(
            (AfterAssemble, SEGMENT_OUT_MIN),
            HashMap::from([("in.a".into(), (0, ScriptOutType::All))]),
        );
        assert!(idmgr.add_outputs(&outputs).is_err());

        let mut outputs = HashMap::new();
        outputs.insert(
            (OnTimeout, SEGMENT_OUT_MIN + 2),
            HashMap::from([("math.abs".into(), (0, ScriptOutType::All))]),
        );
        assert!(idmgr.add_outputs(&outputs).is_err());

        use CmdType::*;

        let funcs = HashMap::new();
        assert!(idmgr.add_func_exts(&funcs).is_ok());

        let mut funcs = HashMap::new();
        funcs.insert(
            OnAssemble,
            HashMap::from([
                ("ext.init".into(), (1, vec![Str, Num])),
                ("ext.get".into(), (2, vec![Str])),
            ]),
        );
        assert!(idmgr.add_func_exts(&funcs).is_ok());

        let mut funcs = HashMap::new();
        funcs.insert(OnAssemble, HashMap::from([("ext.init".into(), (3, vec![Str]))]));
        assert!(idmgr.add_func_exts(&funcs).is_err());

        let mut funcs = HashMap::new();
        funcs.insert(AfterHit, HashMap::from([("ext.init".into(), (3, vec![Str]))]));
        assert!(idmgr.add_func_exts(&funcs).is_ok());

        let mut funcs = HashMap::new();
        funcs.insert(OnInterval, HashMap::from([("in.a".into(), (3, vec![Str]))]));
        assert!(idmgr.add_func_exts(&funcs).is_ok());
    }

    #[test]
    fn test_ident_manager_scope() {
        let mut idmgr = IdentManager::new();
        idmgr.add_consts(&builtin::consts()).unwrap();
        idmgr.add_funcs(&builtin::functions()).unwrap();

        let mut inputs = HashMap::new();
        inputs.insert((OnAssemble, SEGMENT_IN_MIN), HashMap::from([("in.a".into(), 0)]));
        idmgr.add_inputs(&inputs).unwrap();
        idmgr.sync_block_path().unwrap();

        assert!(idmgr.add_arguments(&["a".into(), "b".into()]).is_ok());
        assert!(idmgr.add_arguments(&["a".into()]).is_err());
        assert!(idmgr.get("a").is_none());
        assert!(idmgr.get("A.a").is_some());

        assert!(idmgr.add_closure("c", 1.0).is_ok());
        assert!(idmgr.add_closure("in", 1.0).is_err());
        assert!(idmgr.add_closure("a", 2.0).is_ok());
        assert_eq!(idmgr.closures, vec![("c".into(), 1.0), ("a".into(), 2.0)]);
        assert!(idmgr.get("a").is_some());

        assert!(idmgr.add_local("d").is_ok());
        assert!(idmgr.add_local("a").is_err());
        idmgr.push_scope(Some(OnFinish));
        assert!(idmgr.add_local("e").is_ok());
        assert_eq!(idmgr.locals, vec![("d".into(), 0), ("e".into(), 1)]);
        idmgr.pop_scope();
        assert_eq!(idmgr.locals, vec![("d".into(), 0)]);

        assert!(idmgr.add_number("N", 1.0).is_ok());
        assert!(idmgr.add_string("S", "aaa".into()).is_ok());
        assert!(idmgr.get("N").is_some());
        assert!(idmgr.get("S").is_some());
        idmgr.push_scope(Some(OnFinish));
        assert!(idmgr.add_number("N2", 1.0).is_ok());
        assert_eq!(
            idmgr.temp_consts,
            vec![("N".into(), 0), ("S".into(), 0), ("N2".into(), 1)]
        );

        idmgr.reset();
        assert_eq!(idmgr.arguments.len(), 0);
        assert_eq!(idmgr.closures.len(), 0);
        assert_eq!(idmgr.temp_consts.len(), 0);
        assert_eq!(idmgr.locals.len(), 0);

        idmgr.reset();
        assert!(idmgr.add_arguments(&["a".into(), "b".into()]).is_ok());
        for idx in 2..MAX_CLOSURE {
            assert!(idmgr.add_closure(&format!("closure_{}", idx), 1.0).is_ok());
        }
        assert!(idmgr.add_closure("closure", 1.0).is_err());

        idmgr.reset();
        for idx in 0..MAX_LOCAL {
            assert!(idmgr.add_local(&format!("local{}", idx)).is_ok());
        }
        assert!(idmgr.add_local("local").is_err());
    }

    #[test]
    fn test_parser_empty() {
        let mut parser = new_parser();
        let res = parser.run("", &[]).unwrap();
        assert_eq!(res.blocks, vec![]);
    }

    #[test]
    fn test_parser_block() {
        let mut parser = new_parser();
        let code = r"
            on_assemble {}
            on_timeout(1s) {}
            on_timeout(10) {}
        ";
        let res = parser.run(code, &[]).unwrap();
        assert_eq!(
            res.blocks,
            vec![
                AstBlock::new_hook(OnAssemble, vec![]),
                AstBlock::new_timer(OnTimeout, 20.0, vec![]),
                AstBlock::new_timer(OnTimeout, 10.0, vec![]),
            ]
        );
        let code = r"
            on_assemble {}
            on_assemble {}
        ";
        assert!(parser.run(code, &[]).is_err());
    }

    #[test]
    fn test_parser_declare() {
        let mut parser = new_parser();
        let code = r"
            const C = 123
            var val = 4.5
            on_assemble { out.xx = C + val }
        ";
        let res = parser.run(code, &[]).unwrap();
        assert_eq!(
            res.blocks,
            vec![AstBlock::new_hook(
                OnAssemble,
                vec![AstStat::new_assign(
                    AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                    AstExpr::new_call(
                        CmdOpt::Add,
                        vec![
                            AstExpr::new_num(123.0),
                            AstExpr::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 0)),
                        ]
                    ),
                )]
            )]
        );
        assert_eq!(res.closure_inits, vec![4.5]);
    }

    #[test]
    fn test_parser_local_assign() {
        let mut parser = new_parser();
        let code = r"
            const C = -123
            after_assemble {
                var local = C
                out.yy += local
                local = in.aa
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        assert_eq!(
            res.blocks,
            vec![AstBlock::new_hook(
                AfterAssemble,
                vec![
                    AstStat::new_assign(AstVar::new_local(1), AstExpr::new_num(-123.0)),
                    AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 1)),
                        AstExpr::new_call(
                            CmdOpt::Add,
                            vec![
                                AstExpr::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 1)),
                                AstExpr::new_local(1)
                            ]
                        ),
                    ),
                    AstStat::new_assign(
                        AstVar::new_local(1),
                        AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN, 0)),
                    ),
                ]
            )]
        );
        let code = " after_assemble { out.zz = 1.0 }";
        assert!(parser.run(code, &[]).is_err());
    }

    #[test]
    fn test_parser_call_stat() {
        let mut parser = new_parser();
        let code = r"
            const KEY = 'key'
            on_interval(0.1m) {
                G.init(KEY, 22)
                G.del(KEY)
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        assert_eq!(
            res.blocks,
            vec![AstBlock::new_timer(
                OnInterval,
                120.0,
                vec![
                    AstStat::new_call(CmdOpt::XInit, vec![AstExpr::new_str("key"), AstExpr::new_num(22.0)]),
                    AstStat::new_call(CmdOpt::XDel, vec![AstExpr::new_str("key")]),
                ]
            )]
        );
    }

    #[test]
    fn test_parser_if_stat() {
        let mut parser = new_parser();
        let code = r"
            on_start {
                if in.bb {
                    math.floor(in.cc)
                } elsif A.a1 {
                    math.ceil(in.cc)
                } else {
                    var t = 1.0
                    out.xx = t
                }
            }
        ";
        let res = parser.run(code, &["a1", "a2"]).unwrap();
        assert_eq!(
            res.blocks,
            vec![AstBlock::new_hook(
                OnStart,
                vec![AstStat::new_branch(
                    Some(AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN, 1))),
                    vec![AstStat::new_call(
                        CmdOpt::Floor,
                        vec![AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN, 2))]
                    )],
                    Some(AstStatBranch::new(
                        Some(AstExpr::new_argument(CmdAddr::new(SEGMENT_CLOSURE, 0))),
                        vec![AstStat::new_call(
                            CmdOpt::Ceil,
                            vec![AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN, 2))]
                        )],
                        Some(AstStatBranch::new(
                            None,
                            vec![
                                AstStat::new_assign(AstVar::new_local(1), AstExpr::new_num(1.0),),
                                AstStat::new_assign(
                                    AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                                    AstExpr::new_local(1),
                                ),
                            ],
                            None
                        ))
                    ))
                )]
            )]
        );
    }

    #[test]
    fn test_parser_simple_expr() {
        let mut parser = new_parser();
        let code = r"
            on_finish {
                out.xx = -in.aa
                out.xx = 1 + -2
                out.xx = 4% %% (1 - A.arg)
                out.xx = (-1.0 != 0) ** 2
            }
        ";
        let res = parser.run(code, &["arg"]).unwrap();
        assert_eq!(
            res.blocks,
            vec![AstBlock::new_hook(
                OnFinish,
                vec![
                    AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                        AstExpr::new_call(CmdOpt::Neg, vec![AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN, 0))]),
                    ),
                    AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                        AstExpr::new_call(CmdOpt::Add, vec![AstExpr::new_num(1.0), AstExpr::new_num(-2.0)]),
                    ),
                    AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                        AstExpr::new_call(
                            CmdOpt::Mod,
                            vec![
                                AstExpr::new_num(0.04),
                                AstExpr::new_call(
                                    CmdOpt::Sub,
                                    vec![
                                        AstExpr::new_num(1.0),
                                        AstExpr::new_argument(CmdAddr::new(SEGMENT_CLOSURE, 0)),
                                    ]
                                ),
                            ]
                        ),
                    ),
                    AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                        AstExpr::new_call(
                            CmdOpt::Pow,
                            vec![
                                AstExpr::new_call(CmdOpt::Ne, vec![AstExpr::new_num(-1.0), AstExpr::new_num(0.0)]),
                                AstExpr::new_num(2.0),
                            ]
                        ),
                    ),
                ]
            )]
        )
    }

    #[test]
    fn test_parser_call_expr() {
        let mut parser = new_parser();
        let code = r"
            var res = 0
            on_finish {
                res = math.abs(in.aa)
                out.xx = math.max(G.get('key'), 1.0)
                res = (math.max(((math.round(A.arg))), 5.0))
            }
        ";
        let res = parser.run(code, &["arg"]).unwrap();
        assert_eq!(
            res.blocks,
            vec![AstBlock::new_hook(
                OnFinish,
                vec![
                    AstStat::new_assign(
                        AstVar::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 1)),
                        AstExpr::new_call(CmdOpt::Abs, vec![AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN, 0))]),
                    ),
                    AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                        AstExpr::new_call(
                            CmdOpt::Max,
                            vec![
                                AstExpr::new_call(CmdOpt::XGet, vec![AstExpr::new_str("key")]),
                                AstExpr::new_num(1.0),
                            ]
                        ),
                    ),
                    AstStat::new_assign(
                        AstVar::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 1)),
                        AstExpr::new_call(
                            CmdOpt::Max,
                            vec![
                                AstExpr::new_call(
                                    CmdOpt::Round,
                                    vec![AstExpr::new_argument(CmdAddr::new(SEGMENT_CLOSURE, 0))]
                                ),
                                AstExpr::new_num(5.0),
                            ]
                        ),
                    ),
                ]
            )]
        )
    }

    #[test]
    fn test_parser_if_expr() {
        let mut parser = new_parser();
        let code = r"
            var res = 0
            after_hit {
                res = if res { 1 } else { 0 }
                out.xx = if A.arg { 1 } elsif res { 2 } else { 3 }
                res = if res { 1 } else { if A.arg { -10 } else { -20 } }
            }
        ";
        let res = parser.run(code, &["arg", "_"]).unwrap();
        assert_eq!(
            res.blocks,
            vec![AstBlock::new_hook(
                AfterHit,
                vec![
                    AstStat::new_assign(
                        AstVar::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 2)),
                        AstExpr::new_branch(
                            AstExpr::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 2)),
                            AstExpr::new_num(1.0),
                            AstExpr::new_num(0.0)
                        ),
                    ),
                    AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                        AstExpr::new_branch(
                            AstExpr::new_argument(CmdAddr::new(SEGMENT_CLOSURE, 0)),
                            AstExpr::new_num(1.0),
                            AstExpr::new_branch(
                                AstExpr::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 2)),
                                AstExpr::new_num(2.0),
                                AstExpr::new_num(3.0)
                            )
                        ),
                    ),
                    AstStat::new_assign(
                        AstVar::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 2)),
                        AstExpr::new_branch(
                            AstExpr::new_closure(CmdAddr::new(SEGMENT_CLOSURE, 2)),
                            AstExpr::new_num(1.0),
                            AstExpr::new_branch(
                                AstExpr::new_argument(CmdAddr::new(SEGMENT_CLOSURE, 0)),
                                AstExpr::new_num(-10.0),
                                AstExpr::new_num(-20.0)
                            )
                        ),
                    ),
                ]
            )]
        );
    }

    #[test]
    fn test_parser_multi_block() {
        let mut parser = new_parser();
        let code = r"
            after_hit {
                if 1 {
                    out.xx = in.dd
                }
            }
            on_timeout(10) {
                out.xx = in.ii
            }
        ";
        let res = parser.run(code, &[]).unwrap();
        assert_eq!(
            res.blocks[0],
            AstBlock::new_hook(
                AfterHit,
                vec![AstStat::new_branch(
                    Some(AstExpr::new_num(1.0)),
                    vec![AstStat::new_assign(
                        AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                        AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN + 1, 0)),
                    )],
                    None
                )]
            ),
        );
        assert_eq!(
            res.blocks[1],
            AstBlock::new_timer(
                OnTimeout,
                10.0,
                vec![AstStat::new_assign(
                    AstVar::new_output(CmdAddr::new(SEGMENT_OUT_MIN, 0)),
                    AstExpr::new_input(CmdAddr::new(SEGMENT_IN_MIN + 1, 1)),
                )]
            ),
        );
    }
}
