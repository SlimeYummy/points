use crate::consts::{MAX_ACCESSORY_COUNT, MAX_ENTRY_PLUS, MAX_EQUIPMENT_COUNT};
use crate::parameter::ParamPlayer;
use crate::template::{
    TmplAccessory, TmplAccessoryPattern, TmplAccessoryPool, TmplCharacter, TmplDatabase, TmplEquipment, TmplJewel,
    TmplPerk, TmplSlotType, TmplSlotValue, TmplStyle,
};
use crate::utils::{IDLevel, IDPlus, XError, XResult};

pub struct ContextVerify<'t> {
    pub tmpl_db: &'t TmplDatabase,
}

impl<'t> ContextVerify<'t> {
    pub fn new(tmpl_db: &'t TmplDatabase) -> ContextVerify<'t> {
        ContextVerify { tmpl_db }
    }
}

pub fn verify_player(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<()> {
    let mut slots = TmplSlotValue::default();
    slots.append(&verify_style(ctx, param)?);
    slots.append(&verify_equipments(ctx, param)?);
    slots.append(&verify_perks(ctx, param)?);
    verify_accessories(ctx, param)?;
    verify_jewels(ctx, param, slots)?;

    Ok(())
}

fn verify_style(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<TmplSlotValue> {
    let character = ctx.tmpl_db.find_as::<TmplCharacter>(&param.character)?;
    let style = ctx.tmpl_db.find_as::<TmplStyle>(&param.style)?;

    if !character.styles.contains(&style.id) {
        return Err(XError::bad_parameter("Character and Style mismatch"));
    }
    if param.level < character.level.min || param.level > character.level.max {
        return Err(XError::bad_parameter("Invalid style level"));
    }

    let mut slots = TmplSlotValue::default();
    if !style.slots.is_empty() {
        slots.append(&style.slots[(param.level - character.level.min) as usize]);
    }
    Ok(slots)
}

fn verify_equipments(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<TmplSlotValue> {
    let character = ctx.tmpl_db.find_as::<TmplCharacter>(&param.character)?;

    let mut slots = TmplSlotValue::default();
    let mut positions = [None; MAX_EQUIPMENT_COUNT];

    for (idx, IDLevel { id, level }) in param.equipments.iter().enumerate() {
        let equipment = ctx.tmpl_db.find_as::<TmplEquipment>(id)?;
        if !character.equipments.contains(&equipment.id) {
            return Err(XError::bad_parameter("Character and Equipment mismatch"));
        }

        if positions.contains(&Some(equipment.position)) {
            return Err(XError::bad_parameter("Equipment type conflict"));
        } else {
            positions[idx] = Some(equipment.position);
        }

        if *level < equipment.level.min || *level > equipment.level.max {
            return Err(XError::bad_parameter("Invalid equipment level"));
        }

        if !equipment.slots.is_empty() {
            slots.append(&equipment.slots[(level - equipment.level.min) as usize]);
        }
    }

    Ok(slots)
}

fn verify_perks(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<TmplSlotValue> {
    let style = ctx.tmpl_db.find_as::<TmplStyle>(&param.style)?;

    let mut slots = TmplSlotValue::default();

    for id in param.perks.iter() {
        let perk = ctx.tmpl_db.find_as::<TmplPerk>(id)?;
        if perk.style != style.id && !perk.usable_styles.contains(&style.id) {
            return Err(XError::bad_parameter("Style and Perk mismatch"));
        }

        if let Some(slot) = perk.slot {
            slots.append(&slot);
        }
    }

    Ok(slots)
}

fn verify_accessories(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<()> {
    if param.accessories.len() > MAX_ACCESSORY_COUNT {
        return Err(XError::bad_parameter("Too many accessories"));
    }

    for pa in param.accessories.iter() {
        let accessory = ctx.tmpl_db.find_as::<TmplAccessory>(&pa.id)?;
        let pattern = ctx.tmpl_db.find_as::<TmplAccessoryPattern>(&accessory.pattern)?;

        if pa.level > pattern.max_level {
            return Err(XError::bad_parameter("Invalid accessory level"));
        }
        if pa.entries.len() > pattern.pattern.len() {
            return Err(XError::bad_parameter("Invalid entry count"));
        }

        for (idx, entry_id) in pa.entries.iter().enumerate() {
            let contain = match pattern.pattern[idx] {
                TmplAccessoryPool::A => pattern.a_pool.contains_key(entry_id),
                TmplAccessoryPool::B => pattern.b_pool.contains_key(entry_id),
                TmplAccessoryPool::AB => pattern.a_pool.contains_key(entry_id) || pattern.b_pool.contains_key(entry_id),
            };
            if !contain {
                return Err(XError::bad_parameter("Invalid entry type"));
            }
        }
    }

    Ok(())
}

fn verify_jewels(ctx: &mut ContextVerify<'_>, param: &ParamPlayer, slots: TmplSlotValue) -> XResult<()> {
    let mut slots = slots;
    for IDPlus { id, plus } in param.jewels.iter() {
        let jewel = ctx.tmpl_db.find_as::<TmplJewel>(id)?;
        if *plus > MAX_ENTRY_PLUS {
            return Err(XError::bad_parameter("Invalid jewel plus"));
        }

        let count = match jewel.slot_type {
            TmplSlotType::Attack => &mut slots.attack,
            TmplSlotType::Defense => &mut slots.defense,
            TmplSlotType::Special => &mut slots.special,
        };
        if *count == 0 {
            return Err(XError::bad_parameter("Jewels and slots mismatch"));
        }
        *count -= 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameter::ParamAccessory;
    use crate::utils::s;

    #[test]
    fn test_verify_style() {
        let db = TmplDatabase::new("../test-res").unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.character = s!("Character.No1");
        param.style = s!("Style.No2-1");
        let err = verify_style(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Character and Style mismatch)");

        param.style = s!("Style.No1-1");
        param.level = 10;
        let err = verify_style(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Invalid style level)");

        param.style = s!("Style.No1-1");
        param.level = 1;
        assert_eq!(verify_style(&mut ctx, &param).unwrap(), TmplSlotValue::new(0, 2, 2));

        param.style = s!("Style.No1-1");
        param.level = 6;
        assert_eq!(verify_style(&mut ctx, &param).unwrap(), TmplSlotValue::new(3, 5, 4));
    }

    #[test]
    fn test_verify_equipments() {
        let db = TmplDatabase::new("../test-res").unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.character = s!("Character.No1");
        param.equipments = vec![IDLevel::new(&s!("Equipment.No4"), 0)];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Character and Equipment mismatch)");

        param.equipments = vec![
            IDLevel::new(&s!("Equipment.No1"), 1),
            IDLevel::new(&s!("Equipment.No3"), 0),
        ];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Equipment type conflict)");

        param.equipments = vec![IDLevel::new(&s!("Equipment.No1"), 0)];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Invalid equipment level)");

        param.equipments = vec![IDLevel::new(&s!("Equipment.No1"), 5)];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Invalid equipment level)");

        param.equipments = vec![
            IDLevel::new(&s!("Equipment.No1"), 4),
            IDLevel::new(&s!("Equipment.No2"), 3),
        ];
        assert_eq!(
            verify_equipments(&mut ctx, &param).unwrap(),
            TmplSlotValue::new(1, 3, 0)
        );
    }

    #[test]
    fn test_verify_perks() {
        let db = TmplDatabase::new("../test-res").unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.style = s!("Style.No2-1");
        param.perks = vec![s!("Perk.No1.Empty")];
        let err = verify_perks(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Style and Perk mismatch)");

        param.style = s!("Style.No1-1");
        param.perks = vec![s!("Perk.No1.AttackUp"), s!("Perk.No1.Slot")];
        assert_eq!(verify_perks(&mut ctx, &param).unwrap(), TmplSlotValue::new(0, 2, 2));
    }

    #[test]
    fn test_verify_accessories() {
        let db = TmplDatabase::new("../test-res").unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.accessories = vec![
            ParamAccessory {
                id: s!("Accessory.No1"),
                level: 1,
                entries: vec![s!("Accessory.No1.Entry1")],
            };
            5
        ];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Too many accessories)");

        param.accessories = vec![ParamAccessory {
            id: s!("Accessory.AttackUp.Variant1"),
            level: 0,
            entries: vec![s!("Entry.DefenseUp")],
        }];
        assert!(verify_accessories(&mut ctx, &param).is_ok());

        param.accessories = vec![ParamAccessory {
            id: s!("Accessory.AttackUp.Variant1"),
            level: 10,
            entries: vec![s!("Entry.DefenseUp")],
        }];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Invalid accessory level)");

        param.accessories = vec![ParamAccessory {
            id: s!("Accessory.AttackUp.Variant1"),
            level: 1,
            entries: vec![s!("Entry.AttackUp"); 3],
        }];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Invalid entry count)");

        param.accessories = vec![ParamAccessory {
            id: s!("Accessory.AttackUp.Variant1"),
            level: 1,
            entries: vec![s!("Entry.DefenseUp"), s!("Entry.AttackUp")],
        }];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Invalid entry type)");
    }

    #[test]
    fn test_verify_jewels() {
        let db = TmplDatabase::new("../test-res").unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.jewels = vec![IDPlus::new(&s!("Jewel.AttackUp.Variant1"), 0)];
        assert!(verify_jewels(&mut ctx, &param, TmplSlotValue::new(1, 1, 1)).is_ok());

        param.jewels = vec![IDPlus::new(&s!("Jewel.AttackUp.Variant1"), 4)];
        let err = verify_jewels(&mut ctx, &param, TmplSlotValue::new(1, 1, 1)).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Invalid jewel plus)");

        param.jewels = vec![
            IDPlus::new(&s!("Jewel.AttackUp.Variant1"), 1),
            IDPlus::new(&s!("Jewel.AttackUp.Variant1"), 1),
            IDPlus::new(&s!("Jewel.AttackUp.VariantX"), 1),
        ];
        let err = verify_jewels(&mut ctx, &param, TmplSlotValue::new(1, 1, 1)).unwrap_err();
        assert_eq!(format!("{}", err), "Bad parameter (Jewels and slots mismatch)");
    }
}
