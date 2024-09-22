use std::rc::Rc;

use crate::instance::action::InstAction;
use crate::instance::base::{InstEntryPair, InstSlotValue};
use crate::instance::script::InstScript;
use crate::instance::values::{PanelValues, PrimaryValues, SecondaryValues};
use crate::utils::{CastPtr, DtHashIndex, DtHashMap, IDSymbol, KeyCode, Num, StrID, Symbol, SymbolMap};

#[derive(Debug, Default)]
pub struct InstPlayer {
    pub tmpl_character: StrID,
    pub tmpl_style: StrID,
    pub level: u32,

    pub primary: PrimaryValues,
    pub secondary: SecondaryValues,
    pub panel: PanelValues,
    pub slots: InstSlotValue,
    pub entries: InstEntreis,

    pub global: SymbolMap<Num>,
    pub scripts: Vec<InstScript>,

    pub skeleton: Symbol,
    pub action_args: DtHashMap<IDSymbol, u32>,
    pub actions: DtHashMap<StrID, Rc<dyn InstAction>>,
    pub primary_keys: DtHashIndex<KeyCode, StrID>,
    pub derive_keys: DtHashIndex<(StrID, KeyCode), StrID>,
}

impl InstPlayer {
    pub fn find_action_by_id<T: 'static>(&self, id: &Symbol) -> Option<Rc<T>> {
        let inst_act = self.actions.get(id)?;
        inst_act.cast_to::<T>().ok()
    }

    pub fn find_iter_primary_actions<'a, 'b, 'c>(
        &'a self,
        key: &'b KeyCode,
    ) -> impl Iterator<Item = Rc<dyn InstAction + 'static>> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        return self
            .primary_keys
            .find_iter(key)
            .filter_map(|id| self.actions.get(id)).cloned();
    }

    pub fn find_first_primary_action<T: 'static>(&self, key: &KeyCode) -> Option<Rc<T>> {
        let act_id = self.primary_keys.find_first(key)?;
        let inst_act = self.actions.get(act_id)?;
        inst_act.cast_to::<T>().ok()
    }

    pub fn find_iter_derive_actions<'a, 'b, 'c>(
        &'a self,
        key: &'b (Symbol, KeyCode),
    ) -> impl Iterator<Item = Rc<dyn InstAction + 'static>> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        return self
            .derive_keys
            .find_iter(key)
            .filter_map(|id| self.actions.get(id)).cloned();
    }

    pub fn find_first_derive_action<T: 'static>(&self, key: &(Symbol, KeyCode)) -> Option<Rc<T>> {
        let act_id = self.derive_keys.find_first(key)?;
        let inst_act = self.actions.get(act_id)?;
        inst_act.cast_to::<T>().ok()
    }

    pub fn search_next_action(
        &self,
        current_action_id: &Symbol,
        current_derive_level: u16,
        key: KeyCode,
    ) -> Option<Rc<dyn InstAction>> {
        // TODO: important!!! Consider derive priority (guard/dodge) in preinput senarios.

        let current_action_id = current_action_id.clone();
        for action in self.find_iter_derive_actions(&(current_action_id, key)) {
            if action.enter_level > current_derive_level {
                return Some(action.clone());
            }
        }
        for action in self.find_iter_primary_actions(&key) {
            if action.enter_level > current_derive_level {
                return Some(action.clone());
            }
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct InstEntreis(SymbolMap<InstEntryPair>);

impl InstEntreis {
    pub fn append(&mut self, id: &StrID, pair: InstEntryPair) {
        if pair.piece == 0 {
            return;
        }
        if let Some(val) = self.0.get_mut(id) {
            val.piece += pair.piece;
            val.plus += pair.plus;
        } else {
            self.0.insert(id.clone(), pair);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&StrID, &InstEntryPair)> {
        return self.0.iter();
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&StrID, &mut InstEntryPair)> {
        return self.0.iter_mut();
    }

    pub fn keys(&self) -> impl Iterator<Item = &StrID> {
        return self.0.keys();
    }

    pub fn values(&self) -> impl Iterator<Item = &InstEntryPair> {
        return self.0.values();
    }

    pub fn get(&self, id: StrID) -> Option<&InstEntryPair> {
        return self.0.get(&id);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
