use glam::Quat;
use glam_ext::Vec2xz;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use crate::instance::action::InstActionAny;
use crate::instance::values::{PanelValues, PrimaryValues, SecondaryValues};
use crate::template::TmplHashMap;
use crate::utils::{
    Castable, DtHashIndex, DtHashMap, JewelSlots, PiecePlus, ShapeTaperedCapsule, Symbol, TmplID, VirtualKey,
};

#[derive(Debug, Default)]
pub struct InstPlayer {
    pub tmpl_character: TmplID,
    pub tmpl_style: TmplID,
    pub level: u32,

    pub tags: Vec<Symbol>,
    pub skeleton_files: Symbol,
    pub skeleton_toward: Vec2xz,
    pub skeleton_rotation: Quat,
    pub body_file: Symbol,
    pub bounding: ShapeTaperedCapsule,

    pub primary: PrimaryValues,
    pub secondary: SecondaryValues,
    pub panel: PanelValues,
    pub slots: JewelSlots,
    pub entries: TmplHashMap<PiecePlus>,

    pub var_indexes: TmplHashMap<u32>,
    // pub global: SymbolMap<Num>,
    // pub scripts: Vec<InstScript>,
    pub actions: DtHashMap<TmplID, Rc<dyn InstActionAny>>,
    pub primary_keys: DtHashIndex<VirtualKey, TmplID>,
    pub derive_keys: DtHashIndex<(TmplID, VirtualKey), TmplID>,
}

impl InstPlayer {
    pub(crate) fn append_entry(&mut self, id: TmplID, pair: PiecePlus) {
        if pair.piece == 0 {
            return;
        }
        if let Some(val) = self.entries.get_mut(&id) {
            val.piece += pair.piece;
            val.plus += pair.plus;
        }
        else {
            self.entries.insert(id.clone(), pair);
        }
    }

    pub(crate) fn append_var_index(&mut self, var: TmplID, index: u32) {
        match self.var_indexes.entry(var) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = u32::max(*entry.get(), entry.get() + index);
            }
            Entry::Vacant(entry) => {
                entry.insert(index);
            }
        }
    }

    pub fn find_action_by_id<T>(&self, id: TmplID) -> Option<Rc<T>>
    where
        T: 'static + InstActionAny,
    {
        let inst_act = self.actions.get(&id)?;
        inst_act.clone().cast::<T>().ok()
    }

    pub fn filter_primary_actions<'a, 'b, 'c>(
        &'a self,
        key: &'b VirtualKey,
    ) -> impl Iterator<Item = Rc<dyn InstActionAny + 'static>> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        self.primary_keys
            .find_iter(key)
            .filter_map(|id| self.actions.get(id))
            .cloned()
    }

    pub fn filter_derive_actions<'a, 'b, 'c>(
        &'a self,
        key: &'b (TmplID, VirtualKey),
    ) -> impl Iterator<Item = Rc<dyn InstActionAny + 'static>> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        self.derive_keys
            .find_iter(key)
            .filter_map(|id| self.actions.get(id))
            .cloned()
    }

    pub fn filter_actions<'a, 'b, 'c>(
        &'a self,
        key: &'b (TmplID, VirtualKey),
    ) -> impl Iterator<Item = Rc<dyn InstActionAny + 'static>> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        self.filter_derive_actions(key)
            .chain(self.filter_primary_actions(&key.1))
    }

    pub fn find_first_primary_action<T: 'static>(&self, key: &VirtualKey) -> Option<Rc<T>> {
        let act_id = self.primary_keys.find_first(key)?;
        let inst_act = self.actions.get(act_id)?;
        inst_act.clone().cast::<T>().ok()
    }

    pub fn find_first_derive_action<T: 'static>(&self, key: &(TmplID, VirtualKey)) -> Option<Rc<T>> {
        let act_id = self.derive_keys.find_first(key)?;
        let inst_act = self.actions.get(act_id)?;
        inst_act.clone().cast::<T>().ok()
    }
}
