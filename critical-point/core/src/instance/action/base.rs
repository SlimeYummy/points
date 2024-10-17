use std::fmt::Debug;

use crate::template::{TmplAnimation, TmplClass, TmplSwitch};
use crate::utils::{interface, DtHashIndex, DtHashMap, IDSymbol, KeyCode, StrID, Symbol};

#[derive(Debug)]
pub struct InstActionBase {
    pub id: StrID,
    pub enter_key: Option<KeyCode>,
    pub enter_level: u16,
}

pub unsafe trait InstAction: Debug {
    fn class(&self) -> TmplClass;
    fn get_animations<'a>(&'a self, animations: &mut Vec<&'a TmplAnimation>);
}

interface!(InstAction, InstActionBase);

pub(crate) struct ContextActionAssemble<'t> {
    pub args: &'t DtHashMap<IDSymbol, u32>,
    pub primary_keys: &'t mut DtHashIndex<KeyCode, StrID>,
    pub derive_keys: &'t mut DtHashIndex<(StrID, KeyCode), StrID>,
}

pub(crate) fn query_switch(args: &DtHashMap<IDSymbol, u32>, id: &StrID, switch: &TmplSwitch) -> bool {
    match switch {
        TmplSwitch::Bool(b) => *b,
        TmplSwitch::Symbol(symbol) => {
            return args.get(&IDSymbol::new(id, symbol)).map(|v| *v > 0).unwrap_or(false);
        }
    }
}

pub(crate) fn query_index(args: &DtHashMap<IDSymbol, u32>, id: &StrID, arg: &Symbol) -> u32 {
    return *args.get(&IDSymbol::new(id, arg)).unwrap_or(&0);
}