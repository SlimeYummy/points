use glam::Quat;
use glam_ext::Vec2xz;
use std::collections::hash_map::Entry;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::instance::action::{assemble_action, collect_action_keys, ContextActionAssemble, InstActionAny};
use crate::instance::base::ContextAssemble;
use crate::instance::values::{PanelValues, PrimaryValues, SecondaryValues};
use crate::instance::InstDeriveRule;
use crate::parameter::{ParamNpc, ParamPlayer};
use crate::template::{
    TmplAccessory, TmplAccessoryPool, TmplCharacter, TmplEntry, TmplEquipment, TmplJewel, TmplNpcCharacter, TmplPerk,
    TmplStyle,
};
use crate::utils::{
    force_mut, quat_from_dir_xz, sb, Castable, DtHashIndex, DtHashMap, JewelSlots, PiecePlus, Symbol, TmplID,
    VirtualKey, XResult,
};

#[inline]
pub fn assemble_player(ctx: &mut ContextAssemble, param: &ParamPlayer) -> XResult<InstCharacter> {
    InstCharacter::new_player(ctx, param)
}

#[inline]
pub fn assemble_npc(ctx: &mut ContextAssemble, param: &ParamNpc) -> XResult<InstCharacter> {
    InstCharacter::new_npc(ctx, param)
}

#[derive(Debug, Default)]
pub struct InstCharacter {
    pub is_player: bool,
    pub tmpl_character: TmplID, // TmplCharacter for
    pub tmpl_style: TmplID,     // player only
    pub level: u32,

    pub tags: Vec<Symbol>,
    pub skeleton_files: Symbol,
    pub skeleton_toward: Vec2xz,
    pub skeleton_rotation: Quat,

    pub values: Box<InstValues>,
    pub slots: JewelSlots,
    pub entries: DtHashMap<TmplID, PiecePlus>,
    pub var_indexes: DtHashMap<TmplID, u32>,

    // pub global: SymbolMap<Num>,
    // pub scripts: Vec<InstScript>,
    pub actions: DtHashMap<TmplID, Rc<dyn InstActionAny>>,
    pub primary_keys: DtHashIndex<VirtualKey, TmplID>,
    pub derive_keys: DtHashIndex<(TmplID, VirtualKey), InstDeriveRule>,
}

#[derive(Debug, Default)]
pub struct InstValues {
    pub primary: PrimaryValues,
    pub secondary: SecondaryValues,
    pub panel: PanelValues,
}

impl Deref for InstCharacter {
    type Target = InstValues;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl DerefMut for InstCharacter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl InstCharacter {
    pub fn new_player(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer) -> XResult<InstCharacter> {
        let mut inst = InstCharacter {
            is_player: true,
            tmpl_character: param.character.clone(),
            tmpl_style: param.style.clone(),
            level: param.level,
            ..Default::default()
        };

        Self::collect_player_character_style(ctx, param, &mut inst)?;
        Self::collect_player_equipments(ctx, param, &mut inst)?;
        Self::collect_player_perks(ctx, param, &mut inst)?;
        Self::collect_player_accessories(ctx, param, &mut inst)?;
        Self::collect_player_jewels(ctx, param, &mut inst)?;
        Self::collect_player_actions(ctx, param, &mut inst)?;
        Self::handle_player_entries(ctx, &mut inst)?;

        Ok(inst)
    }

    fn collect_player_character_style(
        ctx: &mut ContextAssemble<'_>,
        param: &ParamPlayer,
        inst: &mut InstCharacter,
    ) -> XResult<()> {
        let chara = ctx.tmpl_db.find_as::<TmplCharacter>(param.character)?;
        let style = ctx.tmpl_db.find_as::<TmplStyle>(param.style)?;

        inst.tags = style.tags.iter().map(|t| sb!(t)).collect();
        inst.skeleton_files = sb!(&chara.skeleton_files);
        inst.skeleton_toward = chara.skeleton_toward;
        inst.skeleton_rotation = quat_from_dir_xz(chara.skeleton_toward);

        let idx = chara.level_to_index(param.level);
        for attr in style.attributes.iter() {
            inst.primary.append_attribute(attr.k, attr.v[idx as usize].into());
            inst.secondary.append_attribute(attr.k, attr.v[idx as usize].into());
        }
        if !style.slots.is_empty() {
            inst.slots = inst.slots.merge(&style.slots[idx as usize]);
        }
        Ok(())
    }

    fn collect_player_equipments(
        ctx: &mut ContextAssemble<'_>,
        param: &ParamPlayer,
        inst: &mut InstCharacter,
    ) -> XResult<()> {
        for pair in param.equipments.iter() {
            let equipment = ctx.tmpl_db.find_as::<TmplEquipment>(pair.id)?;
            let idx = equipment.level_to_index(pair.level);

            for attr in equipment.attributes.iter() {
                inst.primary.append_attribute(attr.k, attr.v[idx as usize].into());
                inst.secondary.append_attribute(attr.k, attr.v[idx as usize].into());
            }

            if !equipment.slots.is_empty() {
                inst.slots = inst.slots.merge(&equipment.slots[idx as usize]);
            }

            for entry in equipment.entries.iter() {
                inst.append_entry(entry.k, entry.v[idx as usize]);
            }
        }
        Ok(())
    }

    fn collect_player_perks(
        ctx: &mut ContextAssemble<'_>,
        param: &ParamPlayer,
        inst: &mut InstCharacter,
    ) -> XResult<()> {
        for pair in param.perks.iter() {
            let perk = ctx.tmpl_db.find_as::<TmplPerk>(pair.id)?;
            let idx = perk.level_to_index(pair.level);

            for attr in perk.attributes.iter() {
                inst.primary.append_attribute(attr.k, attr.v[idx].into());
                inst.secondary.append_attribute(attr.k, attr.v[idx].into());
            }

            if let Some(slot) = perk.slots.get(idx) {
                inst.slots = inst.slots.merge(&slot);
            }

            for entry in perk.entries.iter() {
                inst.append_entry(entry.k, entry.v[idx]);
            }

            for var in perk.var_indexes.iter() {
                inst.append_var_index(var.k, var.v[idx].into());
            }
        }
        Ok(())
    }

    fn collect_player_accessories(
        ctx: &mut ContextAssemble<'_>,
        param: &ParamPlayer,
        inst: &mut InstCharacter,
    ) -> XResult<()> {
        for pa in param.accessories.iter() {
            let accessory = ctx.tmpl_db.find_as::<TmplAccessory>(pa.id)?;
            let pool = ctx.tmpl_db.find_as::<TmplAccessoryPool>(accessory.pool)?;

            inst.append_entry(
                accessory.entry,
                PiecePlus::new(
                    accessory.piece.into(),
                    pool.calc_main_plus(pa.level, accessory.piece.into()),
                ),
            );
            for (pos, entry) in pa.entries.iter().enumerate() {
                inst.append_entry(*entry, PiecePlus::new(1, pool.calc_sub_plus(pa.level, pos as u32)));
            }
        }
        Ok(())
    }

    fn collect_player_jewels(
        ctx: &mut ContextAssemble<'_>,
        param: &ParamPlayer,
        inst: &mut InstCharacter,
    ) -> XResult<()> {
        for pair in param.jewels.iter() {
            let jewel = ctx.tmpl_db.find_as::<TmplJewel>(pair.id)?;

            inst.append_entry(
                jewel.entry,
                PiecePlus::new(jewel.piece.into(), jewel.calc_plus(pair.plus)),
            );
            if let Some((entry, val)) = jewel.calc_sub(pair.plus) {
                inst.append_entry(entry, val);
            }
        }
        Ok(())
    }

    fn collect_player_actions(
        ctx: &mut ContextAssemble<'_>,
        param: &ParamPlayer,
        inst: &mut InstCharacter,
    ) -> XResult<()> {
        let ctxa = ContextActionAssemble {
            var_indexes: &inst.var_indexes,
        };

        let style = ctx.tmpl_db.find_as::<TmplStyle>(param.style)?;
        for id in style.actions.iter() {
            let action = ctx.tmpl_db.find(*id)?;
            if let Some(action) = assemble_action(&ctxa, action)? {
                inst.actions.insert(id.clone(), action);
            }
        }

        let (primary_keys, derive_keys) = collect_action_keys(&inst.actions, true)?;
        inst.primary_keys = primary_keys;
        inst.derive_keys = derive_keys;
        Ok(())
    }

    fn handle_player_entries(ctx: &mut ContextAssemble<'_>, inst: &mut InstCharacter) -> XResult<()> {
        // TODO: remove force_mut
        let inst_mut = unsafe { force_mut(inst) };
        let inst_iter = unsafe { force_mut(inst) };

        for (id, pair) in inst_iter.entries.iter_mut() {
            let entry = ctx.tmpl_db.find_as::<TmplEntry>(*id)?;
            *pair = entry.normalize_pair(*pair);

            if pair.piece > 0 {
                let piece_idx = entry.piece_to_index(pair.piece);
                let plus_idx = entry.plus_to_index(pair.plus);

                inst_mut.secondary.append_table(piece_idx, &entry.attributes)?;
                inst_mut.secondary.append_table(plus_idx, &entry.plus_attributes)?;

                for var in entry.var_indexes.iter() {
                    inst_mut.append_var_index(var.k, var.v[piece_idx].into());
                }
                for var in entry.plus_var_indexes.iter() {
                    inst_mut.append_var_index(var.k, var.v[plus_idx].into());
                }
            }
        }
        Ok(())
    }
}

impl InstCharacter {
    pub fn new_npc(ctx: &mut ContextAssemble<'_>, param: &ParamNpc) -> XResult<InstCharacter> {
        let mut inst = InstCharacter {
            is_player: false,
            tmpl_character: param.character.clone(),
            level: param.level,
            ..Default::default()
        };

        Self::collect_npc_character(ctx, param, &mut inst)?;
        Self::collect_npc_actions(ctx, param, &mut inst)?;

        Ok(inst)
    }

    fn collect_npc_character(ctx: &mut ContextAssemble<'_>, param: &ParamNpc, inst: &mut InstCharacter) -> XResult<()> {
        let chara = ctx.tmpl_db.find_as::<TmplNpcCharacter>(param.character)?;

        inst.tags = chara.tags.iter().map(|t| sb!(t)).collect();
        inst.skeleton_files = sb!(&chara.skeleton_files);
        inst.skeleton_toward = chara.skeleton_toward;
        inst.skeleton_rotation = quat_from_dir_xz(chara.skeleton_toward);

        let idx = chara.level_to_index(param.level);
        for attr in chara.attributes.iter() {
            inst.primary.append_attribute(attr.k, attr.v[idx as usize].into());
            inst.secondary.append_attribute(attr.k, attr.v[idx as usize].into());
        }
        Ok(())
    }

    fn collect_npc_actions(ctx: &mut ContextAssemble<'_>, param: &ParamNpc, inst: &mut InstCharacter) -> XResult<()> {
        let empty_var_indexes = DtHashMap::default();
        let ctxa = ContextActionAssemble {
            var_indexes: &empty_var_indexes,
        };

        let chara = ctx.tmpl_db.find_as::<TmplNpcCharacter>(param.character)?;
        for id in chara.actions.iter() {
            let action = ctx.tmpl_db.find(*id)?;
            if let Some(action) = assemble_action(&ctxa, action)? {
                inst.actions.insert(id.clone(), action);
            }
        }

        let (primary_keys, _) = collect_action_keys(&inst.actions, false)?;
        inst.primary_keys = primary_keys;
        Ok(())
    }
}

impl InstCharacter {
    fn append_entry(&mut self, id: TmplID, pair: PiecePlus) {
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

    fn append_var_index(&mut self, var: TmplID, index: u32) {
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

    pub fn find_first_primary_action<T: 'static>(&self, key: &VirtualKey) -> Option<Rc<T>> {
        let act_id = self.primary_keys.find_first(key)?;
        let inst_act = self.actions.get(act_id)?;
        inst_act.clone().cast::<T>().ok()
    }

    pub fn filter_derive_actions<'a, 'b, 'c>(
        &'a self,
        key: &'b (TmplID, VirtualKey),
    ) -> impl Iterator<Item = (InstDeriveRule, Rc<dyn InstActionAny + 'static>)> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        self.derive_keys
            .find_iter(key)
            .filter_map(|rule| self.actions.get(&rule.action).map(|act| (rule.clone(), act.clone())))
    }

    pub fn find_first_derive_action<T: 'static>(&self, key: &(TmplID, VirtualKey)) -> Option<(InstDeriveRule, Rc<T>)> {
        let rule = self.derive_keys.find_first(key)?;
        let inst_act = self.actions.get(&rule.action)?;
        let inst_act = inst_act.clone().cast::<T>().ok()?;
        Some((rule.clone(), inst_act))
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3A;

    use super::*;
    use crate::instance::InstDeriveRule;
    use crate::parameter::ParamAccessory;
    use crate::template::TmplDatabase;
    use crate::utils::{id, InputDir, JewelSlots, TmplIDLevel, TmplIDPlus, VirtualKey, LEVEL_ATTACK};

    #[test]
    fn test_collect_player_character_style() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.character = id!("Character.Instance^1");
        param.style = id!("Style.Instance^1A");
        param.level = 6;
        let mut inst = InstCharacter::default();
        InstCharacter::collect_player_character_style(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.tags.as_slice(), &[sb!("Player")]);
        assert_eq!(inst.skeleton_files, sb!("Girl.*"));
        assert_eq!(inst.skeleton_toward, Vec2xz::new(0.0, 1.0));
        assert_eq!(inst.skeleton_rotation, quat_from_dir_xz(Vec2xz::new(0.0, 1.0)));

        assert_eq!(inst.primary.max_health, 1200.0);
        assert_eq!(inst.primary.max_posture, 180.0);
        assert_eq!(inst.primary.posture_recovery, 15.0);
        assert_eq!(inst.primary.physical_attack, 35.0);
        assert_eq!(inst.primary.physical_defense, 40.0);
        assert_eq!(inst.secondary.critical_chance, 0.1);
        assert_eq!(inst.secondary.critical_damage, 0.3);
        assert_eq!(inst.secondary.max_health_up, 0.0);
        assert_eq!(inst.slots, JewelSlots::new(3, 5, 4));
    }

    #[test]
    fn test_collect_player_equipments() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.level = 1;
        param.equipments = vec![
            TmplIDLevel::new(id!("Equipment.No1"), 1),
            TmplIDLevel::new(id!("Equipment.No2"), 3),
        ];
        let mut inst = InstCharacter::default();
        InstCharacter::collect_player_equipments(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.primary.physical_attack, 13.0 + 25.0);
        assert_eq!(inst.primary.elemental_attack, 8.0 + 16.0);
        assert_eq!(inst.primary.arcane_attack, 13.0 + 20.0);
        assert_eq!(inst.secondary.critical_chance, 0.02);
        assert_eq!(inst.secondary.critical_damage, 0.18);
        assert_eq!(inst.slots, JewelSlots::new(1, 2, 0));
        assert_eq!(inst.entries.len(), 2);
        assert_eq!(*inst.entries.get(&id!("Entry.AttackUp")).unwrap(), PiecePlus::new(1, 0));
        assert_eq!(
            *inst.entries.get(&id!("Entry.DefenseUp")).unwrap(),
            PiecePlus::new(2, 3)
        );
        // assert_eq!(inst.scripts.len(), 1)
    }

    #[test]
    fn test_collect_player_perks() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.perks = vec![
            TmplIDLevel::new(id!("Perk.Instance^1A"), 1),
            TmplIDLevel::new(id!("Perk.Instance^1B"), 3),
        ];
        let mut inst = InstCharacter::default();
        InstCharacter::collect_player_perks(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.secondary.attack_up, 0.1);
        assert_eq!(inst.slots, JewelSlots::new(1, 2, 2));
        assert_eq!(inst.entries.len(), 2);
        assert_eq!(*inst.entries.get(&id!("Entry.AttackUp")).unwrap(), PiecePlus::new(1, 0));
        assert_eq!(
            *inst.entries.get(&id!("Entry.DefenseUp")).unwrap(),
            PiecePlus::new(1, 3)
        );
        assert_eq!(inst.var_indexes.len(), 2);
        assert_eq!(*inst.var_indexes.get(&id!("#.Perk.Instance^1A")).unwrap(), 0);
        assert_eq!(*inst.var_indexes.get(&id!("#.Perk.Instance^1B")).unwrap(), 5);
    }

    #[test]
    fn test_collect_player_accessories() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.accessories = vec![
            ParamAccessory {
                id: id!("Accessory.AttackUp^1"),
                level: 0,
                entries: vec![id!("Entry.DefenseUp"), id!("Entry.DefenseUp")],
            },
            ParamAccessory {
                id: id!("Accessory.AttackUp^3"),
                level: 12,
                entries: vec![
                    id!("Entry.CriticalChance"),
                    id!("Entry.CriticalChance"),
                    id!("Entry.DefenseUp"),
                    id!("Entry.MaxHealthUp"),
                ],
            },
        ];
        let mut inst = InstCharacter::default();
        InstCharacter::collect_player_accessories(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.entries.len(), 4);
        assert_eq!(*inst.entries.get(&id!("Entry.AttackUp")).unwrap(), PiecePlus::new(4, 6));
        assert_eq!(
            *inst.entries.get(&id!("Entry.DefenseUp")).unwrap(),
            PiecePlus::new(3, 2)
        );
        assert_eq!(
            *inst.entries.get(&id!("Entry.CriticalChance")).unwrap(),
            PiecePlus::new(2, 6)
        );
        assert_eq!(
            *inst.entries.get(&id!("Entry.MaxHealthUp")).unwrap(),
            PiecePlus::new(1, 2)
        );
    }

    #[test]
    fn test_collect_player_jewels() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.jewels = vec![
            TmplIDPlus::new(id!("Jewel.SuperCritical"), 1),
            TmplIDPlus::new(id!("Jewel.AttackUp^1"), 3),
            TmplIDPlus::new(id!("Jewel.AttackUp^2"), 1),
        ];
        let mut inst = InstCharacter::default();
        InstCharacter::collect_player_jewels(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.entries.len(), 3);
        assert_eq!(*inst.entries.get(&id!("Entry.AttackUp")).unwrap(), PiecePlus::new(3, 5));
        assert_eq!(
            *inst.entries.get(&id!("Entry.CriticalChance")).unwrap(),
            PiecePlus::new(2, 2)
        );
        assert_eq!(
            *inst.entries.get(&id!("Entry.CriticalDamage")).unwrap(),
            PiecePlus::new(1, 1)
        );
    }

    #[test]
    fn test_collect_player_actions() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.character = id!("Character.Instance^1");
        param.style = id!("Style.Instance^1A");
        let mut inst = InstCharacter::default();
        inst.var_indexes.insert(id!("#.Action.Instance.AttackDerive^1A"), 2);
        InstCharacter::collect_player_actions(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.actions.len(), 4);
        assert!(inst.actions.contains_key(&id!("Action.Instance.Idle^1A")));
        assert!(inst.actions.contains_key(&id!("Action.Instance.Run^1A")));
        assert!(inst.actions.contains_key(&id!("Action.Instance.Attack^1A")));
        assert!(inst.actions.contains_key(&id!("Action.Instance.AttackDerive^1A")));
        assert_eq!(inst.primary_keys.len(), 3);
        assert_eq!(
            inst.primary_keys.find_first(&VirtualKey::Idle).unwrap(),
            &id!("Action.Instance.Idle^1A")
        );
        assert_eq!(
            inst.primary_keys.find_first(&VirtualKey::Run).unwrap(),
            &id!("Action.Instance.Run^1A")
        );
        assert_eq!(
            inst.primary_keys.find_first(&VirtualKey::Attack1).unwrap(),
            &id!("Action.Instance.Attack^1A")
        );
        assert_eq!(inst.derive_keys.len(), 2);
        assert_eq!(
            inst.derive_keys
                .find_first(&(id!("Action.Instance.Attack^1A"), VirtualKey::Attack1))
                .unwrap(),
            &InstDeriveRule {
                action: id!("Action.Instance.AttackDerive^1A"),
                key: VirtualKey::Attack1,
                level: LEVEL_ATTACK + 1,
                dir: None,
            }
        );
        assert_eq!(
            inst.derive_keys
                .find_first(&(id!("Action.Instance.Attack^1A"), VirtualKey::Attack2))
                .unwrap(),
            &InstDeriveRule {
                action: id!("Action.Instance.AttackDerive^1A"),
                key: VirtualKey::Attack2,
                level: LEVEL_ATTACK + 1,
                dir: Some(InputDir::Backward(0.5)),
            }
        );
    }

    #[test]
    fn test_handle_player_entries() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut inst = InstCharacter::default();
        inst.entries.insert(id!("Entry.AttackUp"), PiecePlus::new(3, 6));
        inst.entries.insert(id!("Entry.MaxHealthUp"), PiecePlus::new(2, 5));
        inst.entries.insert(id!("Entry.DefenseUp"), PiecePlus::new(10, 0));
        inst.entries.insert(id!("Entry.Variable"), PiecePlus::new(3, 0));
        InstCharacter::handle_player_entries(&mut ctx, &mut inst).unwrap();

        assert_eq!(inst.secondary.attack_up, 0.12 + 0.04);
        assert_eq!(inst.secondary.max_health_up, 0.2 + 0.035);
        assert_eq!(inst.secondary.defense_up, 0.6);
        assert_eq!(inst.var_indexes.len(), 2);
        assert_eq!(*inst.var_indexes.get(&id!("#.Entry.Variable^1")).unwrap(), 3);
        assert_eq!(*inst.var_indexes.get(&id!("#.Entry.Variable^2")).unwrap(), 2);
    }

    #[test]
    fn test_inst_player_new() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let param = ParamPlayer {
            character: id!("Character.Instance^1"),
            style: id!("Style.Instance^1A"),
            level: 4,
            equipments: vec![
                TmplIDLevel::new(id!("Equipment.Instance^1A"), 2),
                TmplIDLevel::new(id!("Equipment.Instance^1B"), 3),
            ],
            accessories: vec![ParamAccessory {
                id: id!("Accessory.AttackUp^1"),
                level: 0,
                entries: vec![id!("Entry.DefenseUp"), id!("Entry.DefenseUp")],
            }],
            jewels: vec![
                TmplIDPlus::new(id!("Jewel.SuperCritical"), 1),
                TmplIDPlus::new(id!("Jewel.AttackUp^1"), 3),
                TmplIDPlus::new(id!("Jewel.AttackUp^2"), 1),
            ],
            perks: vec![
                TmplIDLevel::new(id!("Perk.Instance^1A"), 1),
                TmplIDLevel::new(id!("Perk.Instance^1B"), 3),
            ],
            position: Vec3A::ZERO,
        };
        let inst = InstCharacter::new_player(&mut ctx, &param).unwrap();

        assert_eq!(inst.actions.len(), 3);
        assert_eq!(inst.primary_keys.len(), 3);
        assert_eq!(inst.derive_keys.len(), 2);

        assert_eq!(inst.primary.max_health, 850.0);
        assert_eq!(inst.primary.max_posture, 145.0);
        assert_eq!(inst.primary.posture_recovery, 13.0);
        //     assert_eq!(inst.primary.physical_attack, 25.0 + 19.0 + 20.0); // style + equip1 + equip2
        //     assert_eq!(inst.primary.physical_defense, 30.0);
        //     assert_eq!(inst.primary.elemental_attack, 20.0 + 12.0 + 13.0); // style + equip1 + equip2
        //     assert_eq!(inst.primary.arcane_attack, 21.0 + 18.0 + 16.0); // style + equip1 + equip2
        //     assert_eq!(inst.entries.len(), 4); // equip1 + equip2 + access
        //     assert_eq!(
        //         *inst.entries.get(id!("Entry.DefenseUp")).unwrap(),
        //         InstEntryPair::new(5, 5)
        //     );
        //     assert_eq!(
        //         *inst.entries.get(id!("Entry.CriticalChance")).unwrap(),
        //         InstEntryPair::new(1, 3)
        //     );
        //     assert_eq!(
        //         *inst.entries.get(id!("Entry.MaxHealthUp")).unwrap(),
        //         InstEntryPair::new(1, 1)
        //     );
        //     assert_eq!(
        //         *inst.entries.get(id!("Entry.AttackUp")).unwrap(),
        //         InstEntryPair::new(5, 3)
        //     );
        //     assert_eq!(inst.scripts.len(), 3); // equip1 + perk1 + perk2
        //     assert_eq!(inst.slots, TmplSlotValue::new(3, 4, 3)); // style + equip2
        //     assert_eq!(inst.secondary.defense_up, 1.0 + 0.1); // entry
        //     assert_eq!(inst.secondary.attack_up, 0.5 + 0.02 + 0.1); // entry + perk1
        //     assert_eq!(inst.secondary.max_health_up, 0.1); // entry
        //     assert_eq!(inst.secondary.critical_chance, 0.1 + 0.03 + 0.15 + 0.02); // style + equip1 + equip2 + perk2
        //     assert_eq!(inst.secondary.critical_damage, 0.3 + 0.15); // style + equip2 +
        //     assert_eq!(inst.secondary.max_health_up, 0.1); // jewel3

        //     // assert_eq!(inst.panel.cut_defense, 5.0);
        //     // assert_eq!(inst.panel.blunt_defense, 5.0);
        //     // assert_eq!(inst.panel.ammo_defense, 5.0);
        //     // assert_eq!(inst.secondary.final_skill_damage_ratio, 1.1);
        //     // assert_eq!(inst.secondary.critical_chance, 0.04);
        //     // assert_eq!(inst.panel.physical_attack, 13.0 * 1.1 + 2.0);
        //     // assert_eq!(inst.panel.elemental_attack, 8.0 * 1.1 + 2.0);
        //     // assert_eq!(inst.panel.arcane_attack, 13.0 * 1.1 + 2.0);
    }

    #[test]
    fn test_collect_npc_character() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamNpc::default();
        param.character = id!("NpcCharacter.Instance^1");
        param.level = 3;
        let mut inst = InstCharacter::default();
        InstCharacter::collect_npc_character(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.tags.as_slice(), &[sb!("Npc")]);
        assert_eq!(inst.skeleton_files, sb!("TrainingDummy.*"));
        assert_eq!(inst.skeleton_toward, Vec2xz::new(0.0, 1.0));
        assert_eq!(inst.skeleton_rotation, quat_from_dir_xz(Vec2xz::new(0.0, 1.0)));

        assert_eq!(inst.primary.max_health, 1000.0);
        assert_eq!(inst.primary.max_posture, 160.0);
        assert_eq!(inst.primary.physical_attack, 30.0);
        assert_eq!(inst.primary.physical_defense, 35.0);
    }

    #[test]
    fn test_collect_npc_actions() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamNpc::default();
        param.character = id!("NpcCharacter.Instance^1");
        let mut inst = InstCharacter::default();
        InstCharacter::collect_npc_actions(&mut ctx, &param, &mut inst).unwrap();

        assert_eq!(inst.actions.len(), 1);
        assert!(inst.actions.contains_key(&id!("NpcAction.Instance.Idle^1A")));

        assert_eq!(inst.primary_keys.len(), 1);
        assert_eq!(
            inst.primary_keys.find_first(&VirtualKey::Idle).unwrap(),
            &id!("NpcAction.Instance.Idle^1A")
        );
    }

    #[test]
    fn test_inst_npc_new() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let param = ParamNpc {
            character: id!("NpcCharacter.Instance^1"),
            level: 4,
            position: Vec3A::ZERO,
        };
        let inst = InstCharacter::new_npc(&mut ctx, &param).unwrap();

        assert_eq!(inst.actions.len(), 1);

        assert_eq!(inst.primary.max_health, 1000.0);
        assert_eq!(inst.primary.max_posture, 160.0);
        assert_eq!(inst.primary.physical_attack, 30.0);
        assert_eq!(inst.primary.physical_defense, 35.0);
    }
}
