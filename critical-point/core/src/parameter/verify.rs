use crate::consts::{ACCESSORY_MAX_COUNT, EQUIPMENT_MAX_COUNT, MAX_ENTRY_PLUS};
use crate::parameter::ParamPlayer;
use crate::template::{
    TmplAccessory, TmplAccessoryPattern, TmplAccessoryPool, TmplCharacter, TmplDatabase, TmplEquipment, TmplJewel,
    TmplJewelSlot, TmplPerk, TmplStyle,
};
use crate::utils::{xresf, JewelSlots, TmplIDLevel, TmplIDPlus, XResult};

pub struct ContextVerify<'t> {
    pub tmpl_db: &'t TmplDatabase,
}

impl<'t> ContextVerify<'t> {
    pub fn new(tmpl_db: &'t TmplDatabase) -> ContextVerify<'t> {
        ContextVerify { tmpl_db }
    }
}

pub fn verify_player(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<()> {
    let mut slots = JewelSlots::default();
    slots.append(&verify_style(ctx, param)?);
    slots.append(&verify_equipments(ctx, param)?);
    slots.append(&verify_perks(ctx, param)?);
    verify_accessories(ctx, param)?;
    verify_jewels(ctx, param, slots)?;

    Ok(())
}

fn verify_style(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<JewelSlots> {
    let character = ctx.tmpl_db.find_as::<TmplCharacter>(param.character)?;
    let style = ctx.tmpl_db.find_as::<TmplStyle>(param.style)?;

    if !character.styles.contains(&style.id) {
        return xresf!(BadParameter; "character.id={}, style.id={}", character.id, style.id);
    }
    if param.level < character.level.min || param.level > character.level.max {
        return xresf!(BadParameter; "style.id={}, param.level={}", style.id, param.level);
    }

    let mut slots = JewelSlots::default();
    if !style.slots.is_empty() {
        slots.append(&style.slots[(param.level - character.level.min) as usize]);
    }
    Ok(slots)
}

fn verify_equipments(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<JewelSlots> {
    let character = ctx.tmpl_db.find_as::<TmplCharacter>(param.character)?;

    let mut slots = JewelSlots::default();
    let mut equipments = [None; EQUIPMENT_MAX_COUNT];

    for (idx, TmplIDLevel { id, level }) in param.equipments.iter().enumerate() {
        let equipment = ctx.tmpl_db.find_as::<TmplEquipment>(*id)?;
        if !character.equipments.contains(&equipment.id) {
            return xresf!(BadParameter; "character.id={}, equipment.id={}", character.id, equipment.id);
        }

        if equipments.contains(&Some(equipment.slot)) {
            return xresf!(BadParameter; "equipment.id={}, slot={:?}", equipment.id, equipment.slot);
        }
        else {
            equipments[idx] = Some(equipment.slot);
        }

        if *level < equipment.level.min || *level > equipment.level.max {
            return xresf!(BadParameter; "equipment.id={}, level={}", equipment.id, level);
        }

        if !equipment.slots.is_empty() {
            slots.append(&equipment.slots[(level - equipment.level.min) as usize]);
        }
    }
    Ok(slots)
}

fn verify_perks(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<JewelSlots> {
    let style = ctx.tmpl_db.find_as::<TmplStyle>(param.style)?;

    let mut slots = JewelSlots::default();

    for TmplIDLevel { id, level } in param.perks.iter() {
        let perk = ctx.tmpl_db.find_as::<TmplPerk>(*id)?;
        if perk.style != style.id && !perk.usable_styles.contains(&style.id) {
            return xresf!(BadParameter; "style.id={}, perk.id={}", style.id, perk.id);
        }

        if *level < 1 || *level > perk.max_level.into() {
            return xresf!(BadParameter; "perk.id={}, level={}", perk.id, level);
        }

        if let Some(slot) = perk.slots.get(*level as usize - 1) {
            slots.append(&slot);
        }
    }
    Ok(slots)
}

fn verify_accessories(ctx: &mut ContextVerify<'_>, param: &ParamPlayer) -> XResult<()> {
    if param.accessories.len() > ACCESSORY_MAX_COUNT {
        return xresf!(BadParameter; "character.id={}, accessories.len={}", param.character, param.accessories.len());
    }

    for pa in param.accessories.iter() {
        let accessory = ctx.tmpl_db.find_as::<TmplAccessory>(pa.id)?;
        let pool = ctx.tmpl_db.find_as::<TmplAccessoryPool>(accessory.pool)?;

        if pa.level > pool.max_level.into() {
            return xresf!(BadParameter; "accessory.id={}, level={}", pa.id, pa.level);
        }
        if pa.entries.len() > pool.patterns.len() {
            return xresf!(BadParameter; "accessory.id={}, entries.len={}", pa.id, pa.entries.len());
        }

        for (idx, entry_id) in pa.entries.iter().enumerate() {
            let contain = match pool.patterns[idx] {
                TmplAccessoryPattern::A => pool.a_entries.contains_key(entry_id),
                TmplAccessoryPattern::B => pool.b_entries.contains_key(entry_id),
                TmplAccessoryPattern::AB => {
                    pool.a_entries.contains_key(entry_id) || pool.b_entries.contains_key(entry_id)
                }
            };
            if !contain {
                return xresf!(BadParameter; "accessory.id={}, entry_id={}", pa.id, entry_id);
            }
        }
    }
    Ok(())
}

fn verify_jewels(ctx: &mut ContextVerify<'_>, param: &ParamPlayer, slots: JewelSlots) -> XResult<()> {
    let mut slots = slots;
    for (idx, TmplIDPlus { id, plus }) in param.jewels.iter().enumerate() {
        let jewel = ctx.tmpl_db.find_as::<TmplJewel>(*id)?;
        if *plus > MAX_ENTRY_PLUS {
            return xresf!(BadParameter; "jewel.id={}, plus={}", jewel.id, plus);
        }

        let count = match jewel.slot {
            TmplJewelSlot::Attack => &mut slots.attack,
            TmplJewelSlot::Defense => &mut slots.defense,
            TmplJewelSlot::Special => &mut slots.special,
        };
        if *count == 0 {
            return xresf!(BadParameter; "idx={}, jewel.id={}, slot={:?}", idx, jewel.id, jewel.slot);
        }
        *count -= 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameter::ParamAccessory;
    use crate::utils::id;

    #[test]
    fn test_verify_style() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.character = id!("Character.Verify^1");
        param.style = id!("Style.Verify^1A");
        let err = verify_style(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "style.id=Style.Verify^1A, param.level=0");

        param.style = id!("Style.Verify^1A");
        param.level = 10;
        let err = verify_style(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "style.id=Style.Verify^1A, param.level=10");

        param.style = id!("Style.Verify^1A");
        param.level = 1;
        assert_eq!(verify_style(&mut ctx, &param).unwrap(), JewelSlots::new(0, 2, 2));

        param.style = id!("Style.Verify^1A");
        param.level = 3;
        assert_eq!(verify_style(&mut ctx, &param).unwrap(), JewelSlots::new(1, 2, 2));
    }

    #[test]
    fn test_verify_equipments() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.character = id!("Character.Verify^1");
        param.equipments = vec![TmplIDLevel::new(id!("Equipment.Verify^2A"), 0)];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(
            err.msg(),
            "character.id=Character.Verify^1, equipment.id=Equipment.Verify^2A"
        );

        param.equipments = vec![
            TmplIDLevel::new(id!("Equipment.Verify^1A"), 1),
            TmplIDLevel::new(id!("Equipment.Verify^1B"), 0),
        ];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "equipment.id=Equipment.Verify^1B, slot=Slot1");

        param.equipments = vec![TmplIDLevel::new(id!("Equipment.Verify^1A"), 0)];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "equipment.id=Equipment.Verify^1A, level=0");

        param.equipments = vec![TmplIDLevel::new(id!("Equipment.Verify^1A"), 5)];
        let err = verify_equipments(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "equipment.id=Equipment.Verify^1A, level=5");

        param.equipments = vec![
            TmplIDLevel::new(id!("Equipment.Verify^1A"), 4),
            TmplIDLevel::new(id!("Equipment.Verify^1C"), 2),
        ];
        assert_eq!(verify_equipments(&mut ctx, &param).unwrap(), JewelSlots::new(1, 1, 2));
    }

    #[test]
    fn test_verify_perks() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.style = id!("Style.Verify^1A");
        param.perks = vec![TmplIDLevel::new(id!("Perk.Verify^2A"), 1)];
        let err = verify_perks(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "style.id=Style.Verify^1A, perk.id=Perk.Verify^2A");

        param.style = id!("Style.Verify^1B");
        param.perks = vec![TmplIDLevel::new(id!("Perk.Verify^1B"), 1)];
        let err = verify_perks(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "style.id=Style.Verify^1B, perk.id=Perk.Verify^1B");

        param.style = id!("Style.Verify^1A");
        param.perks = vec![TmplIDLevel::new(id!("Perk.Verify^1A"), 0)];
        let err = verify_perks(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "perk.id=Perk.Verify^1A, level=0");

        param.style = id!("Style.Verify^1A");
        param.perks = vec![TmplIDLevel::new(id!("Perk.Verify^1A"), 2)];
        assert_eq!(verify_perks(&mut ctx, &param).unwrap(), JewelSlots::new(0, 1, 0));
    }

    #[test]
    fn test_verify_accessories() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.character = id!("Character.Verify^1");
        param.accessories = vec![
            ParamAccessory {
                id: id!("Accessory.AttackUp^1"),
                level: 1,
                entries: vec![id!("Entry.DefenseUp")]
            };
            5
        ];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "character.id=Character.Verify^1, accessories.len=5");

        param.accessories = vec![ParamAccessory {
            id: id!("Accessory.AttackUp^1"),
            level: 1,
            entries: vec![id!("Entry.DefenseUp")],
        }];
        verify_accessories(&mut ctx, &param).unwrap();

        param.accessories = vec![ParamAccessory {
            id: id!("Accessory.AttackUp^1"),
            level: 10,
            entries: vec![id!("Entry.DefenseUp")],
        }];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "accessory.id=Accessory.AttackUp^1, level=10");

        param.accessories = vec![ParamAccessory {
            id: id!("Accessory.AttackUp^1"),
            level: 1,
            entries: vec![id!("Entry.AttackUp"); 3],
        }];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "accessory.id=Accessory.AttackUp^1, entries.len=3");

        param.accessories = vec![ParamAccessory {
            id: id!("Accessory.AttackUp^1"),
            level: 1,
            entries: vec![id!("Entry.DefenseUp"), id!("Entry.AttackUp")],
        }];
        let err = verify_accessories(&mut ctx, &param).unwrap_err();
        assert_eq!(err.msg(), "accessory.id=Accessory.AttackUp^1, entry_id=Entry.AttackUp");
    }

    #[test]
    fn test_verify_jewels() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut ctx = ContextVerify::new(&db);

        let mut param = ParamPlayer::default();
        param.jewels = vec![TmplIDPlus::new(id!("Jewel.DefenseUp^1"), 0)];
        verify_jewels(&mut ctx, &param, JewelSlots::new(1, 1, 1)).unwrap();

        param.jewels = vec![TmplIDPlus::new(id!("Jewel.DefenseUp^1"), 4)];
        let err = verify_jewels(&mut ctx, &param, JewelSlots::new(1, 1, 1)).unwrap_err();
        assert_eq!(err.msg(), "jewel.id=Jewel.DefenseUp^1, plus=4");

        param.jewels = vec![
            TmplIDPlus::new(id!("Jewel.DefenseUp^1"), 1),
            TmplIDPlus::new(id!("Jewel.DefenseUp^1"), 1),
        ];
        let err = verify_jewels(&mut ctx, &param, JewelSlots::new(1, 1, 1)).unwrap_err();
        assert_eq!(err.msg(), "idx=1, jewel.id=Jewel.DefenseUp^1, slot=Defense");
    }
}
