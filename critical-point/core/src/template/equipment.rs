use crate::template::attribute::TmplAttribute;
use crate::template::base::impl_tmpl;
use crate::utils::{impl_for, rkyv_self, JewelSlots, LevelRange, PiecePlus, Table, TmplID};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplEquipmentSlot {
    Slot1 = 1,
    Slot2 = 2,
    Slot3 = 3,
}

rkyv_self!(TmplEquipmentSlot);

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplEquipment {
    pub id: TmplID,
    pub name: String,
    pub character: TmplID,
    pub slot: TmplEquipmentSlot,
    #[serde(default)]
    pub parents: Table<TmplID, u32>,
    pub level: LevelRange,
    #[serde(default)]
    pub materials: Table<TmplID, Vec<f32>>,
    pub attributes: Table<TmplAttribute, Vec<f32>>,
    #[serde(default)]
    pub slots: Vec<JewelSlots>,
    #[serde(default)]
    pub entries: Table<TmplID, Vec<PiecePlus>>,
    // #[serde(default)]
    // pub script: Option<TmplScript>,
    // #[serde(default)]
    // pub script_args: Table<Symbol, f32>,
}

impl_tmpl!(TmplEquipment, Equipment, "Equipment");

impl_for!(TmplEquipment, ArchivedTmplEquipment, {
    #[inline]
    pub fn level_to_index(&self, level: u32) -> usize {
        (level.clamp(self.level.min, self.level.max) - self.level.min) as usize
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_equipment() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let equipment = db.find_as::<TmplEquipment>(id!("Equipment.No1")).unwrap();
        assert_eq!(equipment.id, id!("Equipment.No1"));
        assert_eq!(equipment.name, "Weapon No1");
        assert_eq!(equipment.character, id!("Character.One"));
        assert_eq!(equipment.slot, TmplEquipmentSlot::Slot1);
        assert_eq!(equipment.level, [1, 4].into());

        assert_eq!(
            equipment.attributes.keys().copied().collect::<Vec<TmplAttribute>>(),
            &[
                TmplAttribute::PhysicalAttack,
                TmplAttribute::ElementalAttack,
                TmplAttribute::ArcaneAttack,
                TmplAttribute::CriticalChance,
            ]
        );
        assert_eq!(equipment.attributes.value_x(0).as_slice(), &[13.0, 19.0, 25.0, 31.0]);
        assert_eq!(equipment.attributes.value_x(1).as_slice(), &[8.0, 12.0, 16.0, 20.0]);
        assert_eq!(equipment.attributes.value_x(2).as_slice(), &[13.0, 18.0, 23.0, 28.0]);
        assert_eq!(equipment.attributes.value_x(3).as_slice(), &[0.02, 0.03, 0.04, 0.05]);

        assert_eq!(
            equipment.slots.as_slice(),
            &[
                JewelSlots::new(0, 0, 0),
                JewelSlots::new(0, 0, 0),
                JewelSlots::new(0, 1, 0),
                JewelSlots::new(0, 1, 0)
            ]
        );

        assert_eq!(equipment.entries.len(), 1);
        assert_eq!(equipment.entries[0].k, id!("Entry.AttackUp"));
        assert_eq!(
            equipment.entries[0].v.as_slice(),
            &[
                PiecePlus::new(1, 0),
                PiecePlus::new(1, 1),
                PiecePlus::new(1, 2),
                PiecePlus::new(1, 3)
            ]
        );

        // assert_eq!(
        //     equipment.script_args.key_iter().cloned().collect::<Vec<Symbol>>(),
        //     &[id!("extra_def"),]
        // );
        // assert_eq!(
        //     equipment
        //         .script_args
        //         .values_iter()
        //         .map(|x| x.to_vec())
        //         .collect::<Vec<Vec<f64>>>(),
        //     &[vec![5.0, 10.0, 15.0, 20.0]]
        // );
    }
}
