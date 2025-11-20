// use crate::consts::MAX_ENTRY_PLUS;
use crate::instance::action::{collect_action_keys, try_assemble_action, ContextActionAssemble};
use crate::instance::player::InstPlayer;
// use crate::instance::script::{AfterAssembleEnv, InstScript, OnAssembleEnv};
// use crate::instance::values::{ExtraValues, PanelValues};
use crate::instance::zone::InstZone;
use crate::parameter::{ParamPlayer, ParamZone};
use crate::sb;
// use crate::script::{ScriptBlockType, ScriptExecutor};
use crate::template::{
    TmplAccessory, TmplAccessoryPool, TmplCharacter, TmplDatabase, TmplEntry, TmplEquipment, TmplJewel, TmplPerk,
    TmplStyle, TmplZone,
};
use crate::utils::{force_mut, quat_from_dir_xz, PiecePlus, XResult};

pub struct ContextAssemble<'t> {
    pub tmpl_db: &'t TmplDatabase,
    // pub executor: &'t mut ScriptExecutor,
}

impl<'t> ContextAssemble<'t> {
    pub fn new(tmpl_db: &'t TmplDatabase) -> ContextAssemble<'t> {
        ContextAssemble { tmpl_db }
    }
}

pub fn assemble_player(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer) -> XResult<InstPlayer> {
    let mut inst = InstPlayer {
        tmpl_character: param.character.clone(),
        tmpl_style: param.style.clone(),
        level: param.level,
        ..Default::default()
    };

    collect_style(ctx, param, &mut inst)?;
    collect_equipments(ctx, param, &mut inst)?;
    collect_perks(ctx, param, &mut inst)?;
    collect_accessories(ctx, param, &mut inst)?;
    collect_jewels(ctx, param, &mut inst)?;
    collect_actions(ctx, param, &mut inst)?;
    handle_entries(ctx, &mut inst)?;
    // trigger_on_assemble(ctx, &mut inst)?;
    // trigger_after_assemble(ctx, &mut inst)?;

    Ok(inst)
}

fn collect_style(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    let chara = ctx.tmpl_db.find_as::<TmplCharacter>(param.character)?;
    let style = ctx.tmpl_db.find_as::<TmplStyle>(param.style)?;

    inst.tags = style.tags.iter().map(|t| sb!(t)).collect();
    inst.skeleton_files = sb!(&chara.skeleton_files);
    inst.skeleton_toward = chara.skeleton_toward;
    inst.skeleton_rotation = quat_from_dir_xz(chara.skeleton_toward);
    inst.body_file = sb!(&chara.body_file);
    inst.bounding = chara.bounding.clone();

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

fn collect_equipments(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
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

        // if let Some(script) = &equipment.script {
        //     inst.scripts
        //         .push(InstScript::new_table(script, idx, &equipment.script_args)?);
        // }
    }
    Ok(())
}

fn collect_perks(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
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

        // if let Some(script) = &perk.script {
        //     inst.scripts.push(InstScript::new_kvlist(script, &perk.script_args)?);
        // }
    }
    Ok(())
}

fn collect_accessories(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
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

fn collect_jewels(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
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

fn collect_actions(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    let ctxa = ContextActionAssemble {
        var_indexes: &inst.var_indexes,
    };

    let style = ctx.tmpl_db.find_as::<TmplStyle>(param.style)?;
    for id in style.actions.iter() {
        let action = ctx.tmpl_db.find(*id)?;
        if let Some(action) = try_assemble_action(&ctxa, action)? {
            inst.actions.insert(id.clone(), action);
        }
    }

    let (primary_keys, derive_keys) = collect_action_keys(&inst.actions)?;
    inst.primary_keys = primary_keys;
    inst.derive_keys = derive_keys;
    Ok(())
}

fn handle_entries(ctx: &mut ContextAssemble<'_>, inst: &mut InstPlayer) -> XResult<()> {
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
            // if let Some(script) = &entry.script {
            //     inst_mut.scripts.push(InstScript::new_table_plus(
            //         script,
            //         pair.piece,
            //         pair.plus / MAX_ENTRY_PLUS,
            //         &entry.script_args,
            //     )?);
            // }
        }
    }
    Ok(())
}

// fn trigger_on_assemble(ctx: &mut ContextAssemble<'_>, inst: &mut InstPlayer) -> XResult<()> {
//     let mut env = OnAssembleEnv {
//         closure: &mut [],
//         primary: &mut inst.primary,
//         secondary: &mut inst.secondary,
//         global: &mut inst.global,
//     };

//     for script in &mut inst.scripts {
//         env.closure = &mut script.closure;
//         match ctx
//             .executor
//             .run_hook(&script.script, ScriptBlockType::OnAssemble, &mut env)
//         {
//             Ok(_) | Err(XError::ScriptNoHook(_)) => continue,
//             Err(err) => return Err(err),
//         }
//     }

//     inst.panel = PanelValues::new(&inst.primary, &inst.secondary);
//     Ok(())
// }

// fn trigger_after_assemble(ctx: &mut ContextAssemble<'_>, inst: &mut InstPlayer) -> XResult<()> {
//     let mut extra = ExtraValues::default();
//     let mut env = AfterAssembleEnv {
//         closure: &mut [],
//         panel: &mut inst.panel,
//         extra: &mut extra,
//         global: &mut inst.global,
//     };

//     for script in &mut inst.scripts {
//         env.closure = &mut script.closure;
//         match ctx
//             .executor
//             .run_hook(&script.script, ScriptBlockType::AfterAssemble, &mut env)
//         {
//             Ok(_) | Err(XError::ScriptNoHook(_)) => continue,
//             Err(err) => return Err(err),
//         }
//     }

//     inst.panel.append_extra(&extra);
//     Ok(())
// }

pub fn assemble_zone(ctx: &mut ContextAssemble<'_>, param: &ParamZone) -> XResult<InstZone> {
    let _ = ctx.tmpl_db.find_as::<TmplZone>(param.zone)?;
    Ok(InstZone {
        tmpl_zone: param.zone.clone(),
    })
}

#[cfg(test)]
mod tests {
    use glam::{Vec3, Vec3A};

    use super::*;
    use crate::parameter::ParamAccessory;
    use crate::utils::{id, JewelSlots, TmplIDLevel, TmplIDPlus, VirtualKey};

    #[test]
    fn test_collect_style() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.character = id!("Character.Instance/1");
        param.style = id!("Style.Instance/1A");
        param.level = 6;
        let mut inst = InstPlayer::default();
        collect_style(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.tags.as_slice(), &[sb!("Player")]);
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
    fn test_collect_equipments() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.level = 1;
        param.equipments = vec![
            TmplIDLevel::new(id!("Equipment.No1"), 1),
            TmplIDLevel::new(id!("Equipment.No2"), 3),
        ];
        let mut inst = InstPlayer::default();
        collect_equipments(&mut ctx, &param, &mut inst).unwrap();
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
    fn test_collect_perks() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.perks = vec![
            TmplIDLevel::new(id!("Perk.Instance/1A"), 1),
            TmplIDLevel::new(id!("Perk.Instance/1B"), 3),
        ];
        let mut inst = InstPlayer::default();
        collect_perks(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.secondary.attack_up, 0.1);
        assert_eq!(inst.slots, JewelSlots::new(1, 2, 2));
        assert_eq!(inst.entries.len(), 2);
        assert_eq!(*inst.entries.get(&id!("Entry.AttackUp")).unwrap(), PiecePlus::new(1, 0));
        assert_eq!(
            *inst.entries.get(&id!("Entry.DefenseUp")).unwrap(),
            PiecePlus::new(1, 3)
        );
        assert_eq!(inst.var_indexes.len(), 2);
        assert_eq!(*inst.var_indexes.get(&id!("#.Perk.Instance/1A")).unwrap(), 0);
        assert_eq!(*inst.var_indexes.get(&id!("#.Perk.Instance/1B")).unwrap(), 5);
    }

    #[test]
    fn test_collect_accessories() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.accessories = vec![
            ParamAccessory {
                id: id!("Accessory.AttackUp/1"),
                level: 0,
                entries: vec![id!("Entry.DefenseUp"), id!("Entry.DefenseUp")],
            },
            ParamAccessory {
                id: id!("Accessory.AttackUp/3"),
                level: 12,
                entries: vec![
                    id!("Entry.CriticalChance"),
                    id!("Entry.CriticalChance"),
                    id!("Entry.DefenseUp"),
                    id!("Entry.MaxHealthUp"),
                ],
            },
        ];
        let mut inst = InstPlayer::default();
        collect_accessories(&mut ctx, &param, &mut inst).unwrap();
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
    fn test_collect_jewels() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.jewels = vec![
            TmplIDPlus::new(id!("Jewel.SuperCritical"), 1),
            TmplIDPlus::new(id!("Jewel.AttackUp/1"), 3),
            TmplIDPlus::new(id!("Jewel.AttackUp/2"), 1),
        ];
        let mut inst = InstPlayer::default();
        collect_jewels(&mut ctx, &param, &mut inst).unwrap();
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
    fn test_collect_actions() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut param = ParamPlayer::default();
        param.character = id!("Character.Instance/1");
        param.style = id!("Style.Instance/1A");

        let mut inst = InstPlayer::default();
        inst.var_indexes.insert(id!("#.Action.Instance.AttackDerive/1A"), 2);
        collect_actions(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.actions.len(), 4);
        assert!(inst.actions.contains_key(&id!("Action.Instance.Idle/1A")));
        assert!(inst.actions.contains_key(&id!("Action.Instance.Run/1A")));
        assert!(inst.actions.contains_key(&id!("Action.Instance.Attack/1A")));
        assert!(inst.actions.contains_key(&id!("Action.Instance.AttackDerive/1A")));
        assert_eq!(inst.primary_keys.len(), 3);
        assert_eq!(
            inst.primary_keys.find_first(&VirtualKey::Idle).unwrap(),
            &id!("Action.Instance.Idle/1A")
        );
        assert_eq!(
            inst.primary_keys.find_first(&VirtualKey::Run).unwrap(),
            &id!("Action.Instance.Run/1A")
        );
        assert_eq!(
            inst.primary_keys.find_first(&VirtualKey::Attack1).unwrap(),
            &id!("Action.Instance.Attack/1A")
        );
        assert_eq!(inst.derive_keys.len(), 2);
        assert_eq!(
            inst.derive_keys
                .find_first(&(id!("Action.Instance.Attack/1A"), VirtualKey::Attack1))
                .unwrap(),
            &id!("Action.Instance.AttackDerive/1A")
        );
        assert_eq!(
            inst.derive_keys
                .find_first(&(id!("Action.Instance.Attack/1A"), VirtualKey::Attack2))
                .unwrap(),
            &id!("Action.Instance.AttackDerive/1A")
        );
    }

    #[test]
    fn test_handle_entries() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let mut inst = InstPlayer::default();
        inst.entries.insert(id!("Entry.AttackUp"), PiecePlus::new(3, 6));
        inst.entries.insert(id!("Entry.MaxHealthUp"), PiecePlus::new(2, 5));
        inst.entries.insert(id!("Entry.DefenseUp"), PiecePlus::new(10, 0));
        inst.entries.insert(id!("Entry.Variable"), PiecePlus::new(3, 0));
        handle_entries(&mut ctx, &mut inst).unwrap();
        assert_eq!(inst.secondary.attack_up, 0.12 + 0.04);
        assert_eq!(inst.secondary.max_health_up, 0.2 + 0.035);
        assert_eq!(inst.secondary.defense_up, 0.6);
        assert_eq!(inst.var_indexes.len(), 2);
        assert_eq!(*inst.var_indexes.get(&id!("#.Entry.Variable/1")).unwrap(), 3);
        assert_eq!(*inst.var_indexes.get(&id!("#.Entry.Variable/2")).unwrap(), 2);
    }

    // #[test]
    // fn test_trigger_script() {
    //     let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();
    //     let mut executor = ScriptExecutor::new();
    //     let mut ctx = ContextAssemble::new(&db, &mut executor);
    //     let mut param = ParamPlayer::default();
    //     param.equipments = vec![IDLevel2::new(&id!("Equipment.No1"), 1)];
    //     param.perks = vec![id!("Perk.No1.AttackUp"), id!("Perk.No1.CriticalChance")];

    //     let mut inst = InstPlayer::default();
    //     collect_equipments(&mut ctx, &param, &mut inst).unwrap();
    //     collect_perks(&mut ctx, &param, &mut inst).unwrap();
    //     trigger_on_assemble(&mut ctx, &mut inst).unwrap();
    //     trigger_after_assemble(&mut ctx, &mut inst).unwrap();

    //     assert_eq!(inst.panel.cut_defense, 5.0);
    //     assert_eq!(inst.panel.blunt_defense, 5.0);
    //     assert_eq!(inst.panel.ammo_defense, 5.0);
    //     assert_eq!(inst.secondary.final_skill_damage_ratio, 1.1);
    //     assert_eq!(inst.secondary.critical_chance, 0.04);
    //     assert_eq!(inst.panel.physical_attack, 13.0 * 1.1 + 2.0);
    //     assert_eq!(inst.panel.elemental_attack, 8.0 * 1.1 + 2.0);
    //     assert_eq!(inst.panel.arcane_attack, 13.0 * 1.1 + 2.0);
    // }

    #[test]
    fn test_assemble_player() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextAssemble::new(&db);

        let param = ParamPlayer {
            character: id!("Character.Instance/1"),
            style: id!("Style.Instance/1A"),
            level: 4,
            equipments: vec![
                TmplIDLevel::new(id!("Equipment.Instance/1A"), 2),
                TmplIDLevel::new(id!("Equipment.Instance/1B"), 3),
            ],
            accessories: vec![ParamAccessory {
                id: id!("Accessory.AttackUp/1"),
                level: 0,
                entries: vec![id!("Entry.DefenseUp"), id!("Entry.DefenseUp")],
            }],
            jewels: vec![
                TmplIDPlus::new(id!("Jewel.SuperCritical"), 1),
                TmplIDPlus::new(id!("Jewel.AttackUp/1"), 3),
                TmplIDPlus::new(id!("Jewel.AttackUp/2"), 1),
            ],
            perks: vec![
                TmplIDLevel::new(id!("Perk.Instance/1A"), 1),
                TmplIDLevel::new(id!("Perk.Instance/1B"), 3),
            ],
            position: Vec3A::ZERO,
        };
        let inst = assemble_player(&mut ctx, &param).unwrap();

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
}
