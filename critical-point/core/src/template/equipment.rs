use crate::template::attribute::TmplAttributeType;
use crate::template::base::{TmplAny, TmplLevelRange, TmplType};
use crate::template::entry::TmplEntryPair;
use crate::template::script::TmplScript;
use crate::template::slot::TmplSlotValue;
use crate::utils::{IDLevel2, List, Num, StrID, Symbol, Table2};

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplEquipmentPosition {
    Position1 = 1,
    Position2 = 2,
    Position3 = 3,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplEquipment {
    pub id: StrID,
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub sub_icon: String,
    pub character: StrID,
    pub position: TmplEquipmentPosition,
    #[serde(default)]
    pub parents: List<IDLevel2>,
    pub level: TmplLevelRange,
    #[serde(default)]
    pub materials: Table2<StrID, Num>,
    pub attributes: Table2<TmplAttributeType, Num>,
    #[serde(default)]
    pub slots: List<TmplSlotValue>,
    #[serde(default)]
    pub entries: Table2<StrID, TmplEntryPair>,
    #[serde(default)]
    pub script: Option<TmplScript>,
    #[serde(default)]
    pub script_args: Table2<Symbol, Num>,
}

#[typetag::deserialize(name = "Equipment")]
impl TmplAny for TmplEquipment {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Equipment
    }
}

impl TmplEquipment {
    #[inline]
    pub fn norm_level(&self, level: u32) -> u32 {
        level - self.level.min
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_TEMPLATE_PATH;
    use crate::template::database::TmplDatabase;
    use crate::utils::sb;

    #[test]
    fn test_load_equipment() {
        let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();

        let equipment = db.find_as::<TmplEquipment>(&sb!("Equipment.No1")).unwrap();
        assert_eq!(equipment.id(), "Equipment.No1");
        assert_eq!(equipment.name, "No1");
        assert_eq!(equipment.icon, "icon");
        assert_eq!(equipment.character, "Character.No1");
        assert_eq!(equipment.position, TmplEquipmentPosition::Position1);
        assert_eq!(equipment.level, [1, 4].into());

        assert_eq!(
            equipment
                .attributes
                .key_iter()
                .copied()
                .collect::<Vec<TmplAttributeType>>(),
            &[
                TmplAttributeType::PhysicalAttack,
                TmplAttributeType::ElementalAttack,
                TmplAttributeType::ArcaneAttack,
                TmplAttributeType::CriticalChance,
            ]
        );
        assert_eq!(
            equipment
                .attributes
                .values_iter()
                .map(|x| x.to_vec())
                .collect::<Vec<Vec<f64>>>(),
            &[
                vec![13.0, 19.0, 25.0, 31.0],
                vec![8.0, 12.0, 16.0, 20.0],
                vec![13.0, 18.0, 23.0, 28.0],
                vec![0.02, 0.03, 0.04, 0.05],
            ]
        );

        assert_eq!(
            equipment.slots.as_slice(),
            &[
                TmplSlotValue::new(0, 0, 0),
                TmplSlotValue::new(0, 0, 0),
                TmplSlotValue::new(0, 1, 0),
                TmplSlotValue::new(0, 1, 0),
            ]
        );

        assert_eq!(
            equipment.entries.key_iter().cloned().collect::<Vec<StrID>>(),
            &[sb!("Entry.AttackUp"),]
        );
        assert_eq!(
            equipment
                .entries
                .values_iter()
                .map(|x| x.to_vec())
                .collect::<Vec<Vec<TmplEntryPair>>>(),
            &[vec![(1, 0).into(), (1, 1).into(), (1, 2).into(), (1, 3).into()]]
        );

        assert_eq!(
            equipment.script_args.key_iter().cloned().collect::<Vec<Symbol>>(),
            &[sb!("extra_def"),]
        );
        assert_eq!(
            equipment
                .script_args
                .values_iter()
                .map(|x| x.to_vec())
                .collect::<Vec<Vec<f64>>>(),
            &[vec![5.0, 10.0, 15.0, 20.0]]
        );
    }
}
