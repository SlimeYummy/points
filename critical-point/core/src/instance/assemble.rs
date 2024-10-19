use std::collections::hash_map::Entry;

use crate::instance::action::{try_assemble_action, ContextActionAssemble};
use crate::instance::base::InstEntryPair;
use crate::instance::player::InstPlayer;
use crate::instance::script::{AfterAssembleEnv, InstScript, OnAssembleEnv};
use crate::instance::stage::InstStage;
use crate::instance::values::{ExtraValues, PanelValues};
use crate::parameter::{ParamPlayer, ParamStage};
use crate::script::{ScriptBlockType, ScriptExecutor};
use crate::template::{
    TmplAccessory, TmplAccessoryPattern, TmplCharacter, TmplDatabase, TmplEntry, TmplEquipment, TmplJewel, TmplPerk,
    TmplStage, TmplStyle, MAX_ENTRY_PLUS,
};
use crate::utils::{IDLevel, IDPlus, XError, XResult};

pub struct ContextAssemble<'t> {
    pub tmpl_db: &'t TmplDatabase,
    pub executor: &'t mut ScriptExecutor,
}

impl<'t> ContextAssemble<'t> {
    pub fn new(tmpl_db: &'t TmplDatabase, executor: &'t mut ScriptExecutor) -> ContextAssemble<'t> {
        ContextAssemble { tmpl_db, executor }
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
    trigger_on_assemble(ctx, &mut inst)?;
    trigger_after_assemble(ctx, &mut inst)?;

    Ok(inst)
}

fn collect_style(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    let chara = ctx.tmpl_db.find_as::<TmplCharacter>(&param.character)?;
    let style = ctx.tmpl_db.find_as::<TmplStyle>(&param.style)?;
    let norm_level = chara.norm_level(param.level);
    for (attr, values) in style.attributes.iter() {
        inst.primary.append_attribute(*attr, values[norm_level as usize]);
        inst.secondary.append_attribute(*attr, values[norm_level as usize]);
    }
    if !style.slots.is_empty() {
        inst.slots = inst.slots.merge(&style.slots[norm_level as usize]);
    }
    Ok(())
}

fn collect_equipments(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    for IDLevel { id, level } in param.equipments.iter() {
        let equipment = ctx.tmpl_db.find_as::<TmplEquipment>(id)?;
        let norm_level = equipment.norm_level(*level);

        for (attr, values) in equipment.attributes.iter() {
            inst.primary.append_attribute(*attr, values[norm_level as usize]);
            inst.secondary.append_attribute(*attr, values[norm_level as usize]);
        }

        if !equipment.slots.is_empty() {
            inst.slots = inst.slots.merge(&equipment.slots[norm_level as usize]);
        }

        for (entry, values) in equipment.entries.iter() {
            inst.entries.append(entry, values[norm_level as usize]);
        }

        if let Some(script) = &equipment.script {
            inst.scripts
                .push(InstScript::new_table(script, norm_level, &equipment.script_args)?);
        }
    }
    Ok(())
}

fn collect_perks(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    for id in param.perks.iter() {
        let perk = ctx.tmpl_db.find_as::<TmplPerk>(id)?;

        for (attr, value) in perk.attributes.iter() {
            inst.primary.append_attribute(*attr, *value);
            inst.secondary.append_attribute(*attr, *value);
        }

        if let Some(slot) = perk.slot {
            inst.slots = inst.slots.merge(&slot);
        }

        for (entry, value) in perk.entries.iter() {
            inst.entries.append(entry, *value);
        }

        for (arg, value) in perk.action_args.iter() {
            match inst.action_args.entry(arg.clone()) {
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = u32::max(*entry.get(), *value);
                }
                Entry::Vacant(entry) => {
                    entry.insert(*value);
                }
            }
        }

        if let Some(script) = &perk.script {
            inst.scripts.push(InstScript::new_kvlist(script, &perk.script_args)?);
        }
    }
    Ok(())
}

fn collect_accessories(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    for ba in param.accessories.iter() {
        let accessory = ctx.tmpl_db.find_as::<TmplAccessory>(&ba.id)?;
        let pattern = ctx.tmpl_db.find_as::<TmplAccessoryPattern>(&accessory.pattern)?;

        inst.entries.append(
            &accessory.entry,
            InstEntryPair::new(accessory.piece, pattern.main_plus(ba.level, accessory.piece)),
        );
        for (pos, entry) in ba.entries.iter().enumerate() {
            inst.entries
                .append(entry, InstEntryPair::new(1, pattern.pool_plus(ba.level, pos as u32)));
        }
    }
    Ok(())
}

fn collect_jewels(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    for IDPlus { id, plus } in param.jewels.iter() {
        let jewel = ctx.tmpl_db.find_as::<TmplJewel>(id)?;

        inst.entries
            .append(&jewel.entry, InstEntryPair::new(jewel.piece, jewel.plus(*plus)));
        if let Some((entry, val)) = jewel.sub() {
            inst.entries.append(&entry, val);
        }
    }
    Ok(())
}

fn handle_entries(ctx: &mut ContextAssemble<'_>, inst: &mut InstPlayer) -> XResult<()> {
    for (entry, InstEntryPair { piece, plus }) in inst.entries.iter_mut() {
        let entry = ctx.tmpl_db.find_as::<TmplEntry>(entry)?;
        *piece = u32::clamp(*piece, 0, entry.max_piece);
        *plus = u32::clamp(*plus, 0, entry.max_plus());

        if *piece > 0 {
            inst.secondary
                .append_table_plus(*piece, *plus / MAX_ENTRY_PLUS, &entry.attributes)?;

            if let Some(script) = &entry.script {
                inst.scripts.push(InstScript::new_table_plus(
                    script,
                    *piece,
                    *plus / MAX_ENTRY_PLUS,
                    &entry.script_args,
                )?);
            }
        }
    }
    Ok(())
}

fn collect_actions(ctx: &mut ContextAssemble<'_>, param: &ParamPlayer, inst: &mut InstPlayer) -> XResult<()> {
    let style = ctx.tmpl_db.find_as::<TmplStyle>(&param.style)?;
    inst.skeleton = style.skeleton.clone();

    let mut action_ctx = ContextActionAssemble {
        args: &inst.action_args,
        primary_keys: &mut inst.primary_keys,
        derive_keys: &mut inst.derive_keys,
    };

    for id in style.actions.iter() {
        let action = ctx.tmpl_db.find(id)?;
        if let Some(action) = try_assemble_action(&mut action_ctx, action)? {
            inst.actions.insert(id.clone(), action);
        }
    }

    // check_builtin_actions(&mut action_ctx)?;
    Ok(())
}

fn trigger_on_assemble(ctx: &mut ContextAssemble<'_>, inst: &mut InstPlayer) -> XResult<()> {
    let mut env = OnAssembleEnv {
        closure: &mut [],
        primary: &mut inst.primary,
        secondary: &mut inst.secondary,
        global: &mut inst.global,
    };

    for script in &mut inst.scripts {
        env.closure = &mut script.closure;
        match ctx
            .executor
            .run_hook(&script.script, ScriptBlockType::OnAssemble, &mut env)
        {
            Ok(_) | Err(XError::ScriptNoHook) => continue,
            Err(err) => return Err(err),
        }
    }

    inst.panel = PanelValues::new(&inst.primary, &inst.secondary);
    Ok(())
}

fn trigger_after_assemble(ctx: &mut ContextAssemble<'_>, inst: &mut InstPlayer) -> XResult<()> {
    let mut extra = ExtraValues::default();
    let mut env = AfterAssembleEnv {
        closure: &mut [],
        panel: &mut inst.panel,
        extra: &mut extra,
        global: &mut inst.global,
    };

    for script in &mut inst.scripts {
        env.closure = &mut script.closure;
        match ctx
            .executor
            .run_hook(&script.script, ScriptBlockType::AfterAssemble, &mut env)
        {
            Ok(_) | Err(XError::ScriptNoHook) => continue,
            Err(err) => return Err(err),
        }
    }

    inst.panel.append_extra(&extra);
    Ok(())
}

pub fn assemble_stage(ctx: &mut ContextAssemble<'_>, param: &ParamStage) -> XResult<InstStage> {
    let tmpl = ctx.tmpl_db.find_as::<TmplStage>(&param.stage)?;
    Ok(InstStage {
        tmpl_stage: param.stage.clone(),
        asset_id: tmpl.asset_id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::player::InstEntreis;
    use crate::parameter::ParamAccessory;
    use crate::template::TmplSlotValue;
    use crate::utils::s;

    #[test]
    fn test_collect_style() {
        let db: TmplDatabase = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let mut param = ParamPlayer::default();
        param.character = s!("Character.No1");
        param.style = s!("Style.No1-1");
        param.level = 6;
        let mut inst = InstPlayer::default();
        collect_style(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.primary.max_health, 1200.0);
        assert_eq!(inst.primary.max_posture, 180.0);
        assert_eq!(inst.primary.posture_recovery, 15.0);
        assert_eq!(inst.primary.physical_attack, 35.0);
        assert_eq!(inst.primary.physical_defense, 40.0);
        assert_eq!(inst.secondary.critical_chance, 0.1);
        assert_eq!(inst.secondary.critical_damage, 0.3);
        assert_eq!(inst.secondary.max_health_up, 0.0);
        assert_eq!(inst.slots, TmplSlotValue::new(3, 5, 4));
    }

    #[test]
    fn test_collect_equipments() {
        let db = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let mut param = ParamPlayer::default();
        param.level = 1;
        param.equipments = vec![
            IDLevel::new(&s!("Equipment.No1"), 1),
            IDLevel::new(&s!("Equipment.No2"), 3),
        ];
        let mut inst = InstPlayer::default();
        collect_equipments(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.primary.physical_attack, 13.0 + 25.0);
        assert_eq!(inst.primary.elemental_attack, 8.0 + 16.0);
        assert_eq!(inst.primary.arcane_attack, 13.0 + 20.0);
        assert_eq!(inst.secondary.critical_chance, 0.02);
        assert_eq!(inst.secondary.critical_damage, 0.18);
        assert_eq!(inst.slots, TmplSlotValue::new(1, 2, 0));
        assert_eq!(inst.entries.len(), 2);
        assert_eq!(
            *inst.entries.get(s!("Entry.AttackUp")).unwrap(),
            InstEntryPair::new(1, 0)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.DefenseUp")).unwrap(),
            InstEntryPair::new(2, 3)
        );
        assert_eq!(inst.scripts.len(), 1)
    }

    #[test]
    fn test_collect_perks() {
        let db = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let mut param = ParamPlayer::default();
        param.perks = vec![
            s!("Perk.No1.AttackUp"),
            s!("Perk.No1.CriticalChance"),
            s!("Perk.No1.Slot"),
        ];
        let mut inst = InstPlayer::default();
        collect_perks(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.secondary.attack_up, 0.1);
        assert_eq!(inst.slots, TmplSlotValue::new(0, 2, 2));
        assert_eq!(inst.entries.len(), 1);
        assert_eq!(
            *inst.entries.get(s!("Entry.CriticalChance")).unwrap(),
            InstEntryPair::new(1, 3)
        );
    }

    #[test]
    fn test_collect_accessories() {
        let db = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let mut param = ParamPlayer::default();
        param.accessories = vec![
            ParamAccessory {
                id: s!("Accessory.AttackUp.Variant1"),
                level: 0,
                entries: vec![s!("Entry.DefenseUp"), s!("Entry.DefenseUp")],
            },
            ParamAccessory {
                id: s!("Accessory.AttackUp.Variant3"),
                level: 12,
                entries: vec![
                    s!("Entry.CriticalChance"),
                    s!("Entry.CriticalChance"),
                    s!("Entry.DefenseUp"),
                    s!("Entry.MaxHealthUp"),
                ],
            },
        ];
        let mut inst = InstPlayer::default();
        collect_accessories(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.entries.len(), 4);
        assert_eq!(
            *inst.entries.get(s!("Entry.AttackUp")).unwrap(),
            InstEntryPair::new(4, 6)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.DefenseUp")).unwrap(),
            InstEntryPair::new(3, 2)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.CriticalChance")).unwrap(),
            InstEntryPair::new(2, 6)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.MaxHealthUp")).unwrap(),
            InstEntryPair::new(1, 2)
        );
    }

    #[test]
    fn test_collect_jewels() {
        let db = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let mut param = ParamPlayer::default();
        param.jewels = vec![
            IDPlus::new(&s!("Jewel.DefenseUp.Variant1"), 3),
            IDPlus::new(&s!("Jewel.AttackUp.Variant1"), 2),
            IDPlus::new(&s!("Jewel.AttackUp.VariantX"), 0),
        ];
        let mut inst = InstPlayer::default();
        collect_jewels(&mut ctx, &param, &mut inst).unwrap();
        assert_eq!(inst.entries.len(), 3);
        assert_eq!(
            *inst.entries.get(s!("Entry.AttackUp")).unwrap(),
            InstEntryPair::new(3, 2)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.MaxHealthUp")).unwrap(),
            InstEntryPair::new(1, 1)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.DefenseUp")).unwrap(),
            InstEntryPair::new(1, 3)
        );
    }

    #[test]
    fn test_handle_entries() {
        let db = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let mut inst = InstPlayer::default();
        inst.entries = InstEntreis::default();
        inst.entries.append(&s!("Entry.AttackUp"), InstEntryPair::new(3, 6));
        inst.entries.append(&s!("Entry.MaxHealthUp"), InstEntryPair::new(2, 5));
        inst.entries.append(&s!("Entry.DefenseUp"), InstEntryPair::new(10, 0));
        handle_entries(&mut ctx, &mut inst).unwrap();
        assert_eq!(inst.secondary.attack_up, 0.3 + 0.04);
        assert_eq!(inst.secondary.max_health_up, 0.2);
        assert_eq!(inst.secondary.defense_up, 1.0 + 0.03);
    }

    #[test]
    fn test_trigger_script() {
        let db = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let mut param = ParamPlayer::default();
        param.equipments = vec![IDLevel::new(&s!("Equipment.No1"), 1)];
        param.perks = vec![s!("Perk.No1.AttackUp"), s!("Perk.No1.CriticalChance")];

        let mut inst = InstPlayer::default();
        collect_equipments(&mut ctx, &param, &mut inst).unwrap();
        collect_perks(&mut ctx, &param, &mut inst).unwrap();
        trigger_on_assemble(&mut ctx, &mut inst).unwrap();
        trigger_after_assemble(&mut ctx, &mut inst).unwrap();

        assert_eq!(inst.panel.cut_defense, 5.0);
        assert_eq!(inst.panel.blunt_defense, 5.0);
        assert_eq!(inst.panel.ammo_defense, 5.0);
        assert_eq!(inst.secondary.final_skill_damage_ratio, 1.1);
        assert_eq!(inst.secondary.critical_chance, 0.04);
        assert_eq!(inst.panel.physical_attack, 13.0 * 1.1 + 2.0);
        assert_eq!(inst.panel.elemental_attack, 8.0 * 1.1 + 2.0);
        assert_eq!(inst.panel.arcane_attack, 13.0 * 1.1 + 2.0);
    }

    #[test]
    fn test_assemble_player() {
        let db = TmplDatabase::new("../test_res").unwrap();
        let mut executor = ScriptExecutor::new();
        let mut ctx = ContextAssemble::new(&db, &mut executor);
        let param = ParamPlayer {
            character: s!("Character.No1"),
            style: s!("Style.No1-1"),
            level: 4,
            equipments: vec![
                IDLevel::new(&s!("Equipment.No1"), 2),
                IDLevel::new(&s!("Equipment.No2"), 2),
            ],
            accessories: vec![ParamAccessory {
                id: s!("Accessory.AttackUp.Variant1"),
                level: 0,
                entries: vec![s!("Entry.DefenseUp"), s!("Entry.DefenseUp")],
            }],
            jewels: vec![
                IDPlus::new(&s!("Jewel.DefenseUp.Variant1"), 3),
                IDPlus::new(&s!("Jewel.AttackUp.Variant1"), 2),
                IDPlus::new(&s!("Jewel.AttackUp.VariantX"), 0),
            ],
            perks: vec![s!("Perk.No1.AttackUp"), s!("Perk.No1.CriticalChance")],
        };
        let inst = assemble_player(&mut ctx, &param).unwrap();

        assert_eq!(inst.actions.len(), 1);
        assert_eq!(inst.primary.max_health, 850.0);
        assert_eq!(inst.primary.max_posture, 145.0);
        assert_eq!(inst.primary.posture_recovery, 13.0);
        assert_eq!(inst.primary.physical_attack, 25.0 + 19.0 + 20.0); // style + equip1 + equip2
        assert_eq!(inst.primary.physical_defense, 30.0);
        assert_eq!(inst.primary.elemental_attack, 20.0 + 12.0 + 13.0); // style + equip1 + equip2
        assert_eq!(inst.primary.arcane_attack, 21.0 + 18.0 + 16.0); // style + equip1 + equip2
        assert_eq!(inst.entries.len(), 4); // equip1 + equip2 + access
        assert_eq!(
            *inst.entries.get(s!("Entry.DefenseUp")).unwrap(),
            InstEntryPair::new(5, 5)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.CriticalChance")).unwrap(),
            InstEntryPair::new(1, 3)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.MaxHealthUp")).unwrap(),
            InstEntryPair::new(1, 1)
        );
        assert_eq!(
            *inst.entries.get(s!("Entry.AttackUp")).unwrap(),
            InstEntryPair::new(5, 3)
        );
        assert_eq!(inst.scripts.len(), 3); // equip1 + perk1 + perk2
        assert_eq!(inst.slots, TmplSlotValue::new(3, 4, 3)); // style + equip2
        assert_eq!(inst.secondary.defense_up, 1.0 + 0.1); // entry
        assert_eq!(inst.secondary.attack_up, 0.5 + 0.02 + 0.1); // entry + perk1
        assert_eq!(inst.secondary.max_health_up, 0.1); // entry
        assert_eq!(inst.secondary.critical_chance, 0.1 + 0.03 + 0.15 + 0.02); // style + equip1 + equip2 + perk2
        assert_eq!(inst.secondary.critical_damage, 0.3 + 0.15); // style + equip2 +
        assert_eq!(inst.secondary.max_health_up, 0.1); // jewel3

        // assert_eq!(inst.panel.cut_defense, 5.0);
        // assert_eq!(inst.panel.blunt_defense, 5.0);
        // assert_eq!(inst.panel.ammo_defense, 5.0);
        // assert_eq!(inst.secondary.final_skill_damage_ratio, 1.1);
        // assert_eq!(inst.secondary.critical_chance, 0.04);
        // assert_eq!(inst.panel.physical_attack, 13.0 * 1.1 + 2.0);
        // assert_eq!(inst.panel.elemental_attack, 8.0 * 1.1 + 2.0);
        // assert_eq!(inst.panel.arcane_attack, 13.0 * 1.1 + 2.0);
    }
}
