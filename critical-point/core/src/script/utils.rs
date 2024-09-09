use std::collections::HashMap;

use crate::script::command::CmdOpt;
use crate::script::config::MAX_INOUT_OFFSET;
use crate::script::parser::{ScriptInputMap, ScriptOutputMap};

macro_rules! sin {
    ($path:path, $field:tt) => {
        (stringify!($field), std::mem::offset_of!($path, $field))
    };
}
pub(crate) use sin;

pub fn script_in(prefix: &str, fields: Vec<(&str, usize)>) -> ScriptInputMap {
    let mut ins = HashMap::new();
    for (field_name, field_offset) in fields {
        if field_offset > MAX_INOUT_OFFSET {
            panic!("offset overflow");
        }
        if field_offset % 8 != 0 {
            panic!("offset not aligned");
        }
        ins.insert(format!("{}.{}", prefix, field_name), (field_offset / 8) as u16);
    }
    ins
}

// pub fn script_ins(children: &[(usize, &ScriptInputMap)]) -> ScriptInputMap {
//     let mut ins = HashMap::new();
//     for (offset, sub_ins) in children {
//         if *offset % 8 != 0 {
//             panic!("offset not aligned");
//         }
//         for (name, field) in sub_ins.iter() {
//             let real_offset = offset + (*field as usize * 8);
//             if real_offset > MAX_INOUT_OFFSET {
//                 panic!("offset overflow");
//             }
//             ins.insert(name.clone(), (real_offset / 8) as u16);
//         }
//     }
//     return ins;
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptOutType {
    Mov,
    Add,
    Mul,
    All,
}

impl ScriptOutType {
    pub fn check_opt(&self, opt: CmdOpt) -> bool {
        use ScriptOutType::*;
        match opt {
            CmdOpt::Mov => *self == Mov || *self == All,
            CmdOpt::Add => *self == Add || *self == All,
            CmdOpt::Sub => *self == Add || *self == All,
            CmdOpt::Mul => *self == Mul || *self == All,
            CmdOpt::Div => *self == Mul || *self == All,
            _ => false,
        }
    }
}

macro_rules! sout {
    (=, $path:path, $field:tt) => {
        (
            stringify!($field),
            std::mem::offset_of!($path, $field),
            ScriptOutType::Mov,
        )
    };
    (+, $path:path, $field:tt) => {
        (
            stringify!($field),
            std::mem::offset_of!($path, $field),
            ScriptOutType::Add,
        )
    };
    (*, $path:path, $field:tt) => {
        (
            stringify!($field),
            std::mem::offset_of!($path, $field),
            ScriptOutType::Mul,
        )
    };
    (!, $path:path, $field:tt) => {
        (
            stringify!($field),
            std::mem::offset_of!($path, $field),
            ScriptOutType::All,
        )
    };
}
pub(crate) use sout;

pub fn script_out(prefix: &str, fields: Vec<(&str, usize, ScriptOutType)>) -> ScriptOutputMap {
    let mut ins = HashMap::new();
    for (field_name, field_offset, field_opts) in fields {
        if field_offset > MAX_INOUT_OFFSET {
            panic!("offset overflow");
        }
        if field_offset % 8 != 0 {
            panic!("offset not aligned");
        }
        ins.insert(
            format!("{}.{}", prefix, field_name),
            ((field_offset / 8) as u16, field_opts),
        );
    }
    ins
}
