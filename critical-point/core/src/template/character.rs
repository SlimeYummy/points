use crate::template::attribute::TmplAttributeType;
use crate::template::base::{TmplAny, TmplLevelRange, TmplType};
use crate::template::slot::TmplSlotValue;
use crate::utils::{List, Num, ShapeCapsule, StrID, Symbol, Table2};
use cirtical_point_csgen::CsEnum;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsEnum,
)]
#[repr(u8)]
pub enum CharacterType {
    Melee,
    Magic,
    Shot,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplCharacter {
    pub id: StrID,
    pub name: String,
    pub level: TmplLevelRange,
    pub styles: List<StrID>,
    pub equipments: List<StrID>,
    pub bounding_capsule: ShapeCapsule,
    pub skeleton: Symbol,
    pub target_box: Symbol,
}

#[typetag::deserialize(name = "Character")]
impl TmplAny for TmplCharacter {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Character
    }
}

impl TmplCharacter {
    #[inline]
    pub fn norm_level(&self, level: u32) -> u32 {
        level - self.level.min
    }
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplStyle {
    pub id: StrID,
    pub name: String,
    pub character: String,
    pub attributes: Table2<TmplAttributeType, Num>,
    pub slots: List<TmplSlotValue>,
    pub fixed_attributes: TmplFixedAttributes,
    pub perks: List<StrID>,
    #[serde(default)]
    pub usable_perks: List<StrID>,
    pub actions: List<StrID>,
    pub icon: String,
    pub view_model: String,
}

#[typetag::deserialize(name = "Style")]
impl TmplAny for TmplStyle {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Style
    }
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplFixedAttributes {
    pub damage_reduce_param_1: f32,
    pub damage_reduce_param_2: f32,
    pub guard_damage_ratio_1: f32,
    pub deposture_reduce_param_1: f32,
    pub deposture_reduce_param_2: f32,
    pub guard_deposture_ratio_1: f32,
    pub weak_damage_up: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_TEMPLATE_PATH;
    use crate::template::database::TmplDatabase;
    use crate::utils::sb;

    #[test]
    fn test_load_character() {
        let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();

        let character = db.find_as::<TmplCharacter>(&sb!("Character.No1")).unwrap();
        assert_eq!(character.id(), "Character.No1");
        assert_eq!(character.name, "No1");
        assert_eq!(character.level, [1, 6].into());
        assert_eq!(&character.styles.as_slice(), &[sb!("Style.No1-1"), sb!("Style.No1-2")]);
        assert_eq!(
            &character.equipments.as_slice(),
            &[sb!("Equipment.No1"), sb!("Equipment.No2"), sb!("Equipment.No3")]
        );
        assert_eq!(character.bounding_capsule, ShapeCapsule::new(0.5 * 1.35, 0.3));
        assert_eq!(character.skeleton, "skel.ozz");

        let style = db.find_as::<TmplStyle>(&sb!("Style.No1-1")).unwrap();
        assert_eq!(style.id(), "Style.No1-1");
        assert_eq!(style.name, "No1-1");
        assert_eq!(style.character, "Character.No1");

        assert_eq!(
            style.attributes.key_iter().copied().collect::<Vec<TmplAttributeType>>(),
            &[
                TmplAttributeType::MaxHealth,
                TmplAttributeType::MaxPosture,
                TmplAttributeType::PostureRecovery,
                TmplAttributeType::PhysicalAttack,
                TmplAttributeType::PhysicalDefense,
                TmplAttributeType::ElementalAttack,
                TmplAttributeType::ElementalDefense,
                TmplAttributeType::ArcaneAttack,
                TmplAttributeType::ArcaneDefense,
                TmplAttributeType::CriticalChance,
                TmplAttributeType::CriticalDamage,
            ]
        );
        assert_eq!(
            style
                .attributes
                .values_iter()
                .map(|x| x.to_vec())
                .collect::<Vec<Vec<f64>>>(),
            &[
                vec![400.0, 550.0, 700.0, 850.0, 1000.0, 1200.0],
                vec![100.0, 115.0, 130.0, 145.0, 160.0, 180.0],
                vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0],
                vec![10.0, 15.0, 20.0, 25.0, 30.0, 35.0],
                vec![15.0, 20.0, 25.0, 30.0, 35.0, 40.0],
                vec![8.0, 12.0, 16.0, 20.0, 24.0, 28.0],
                vec![10.0, 15.0, 20.0, 25.0, 30.0, 35.0],
                vec![9.0, 13.0, 17.0, 21.0, 25.0, 30.0],
                vec![5.0, 8.0, 11.0, 14.0, 17.0, 20.0],
                vec![0.1; 6],
                vec![0.3; 6]
            ]
        );

        assert_eq!(
            style.slots.as_slice(),
            &[
                TmplSlotValue::new(0, 2, 2),
                TmplSlotValue::new(0, 2, 2),
                TmplSlotValue::new(0, 3, 3),
                TmplSlotValue::new(2, 3, 3),
                TmplSlotValue::new(2, 5, 4),
                TmplSlotValue::new(3, 5, 4),
            ]
        );

        assert_eq!(style.fixed_attributes.damage_reduce_param_1, 0.05);
        assert_eq!(style.fixed_attributes.damage_reduce_param_2, 100.0);
        assert_eq!(style.fixed_attributes.guard_damage_ratio_1, 0.8);
        assert_eq!(style.fixed_attributes.deposture_reduce_param_1, 0.05);
        assert_eq!(style.fixed_attributes.deposture_reduce_param_2, 200.0);
        assert_eq!(style.fixed_attributes.guard_deposture_ratio_1, 0.8);
        assert_eq!(style.fixed_attributes.weak_damage_up, 0.25);

        assert_eq!(
            style.perks.as_slice(),
            &[sb!("Perk.No1.AttackUp"), sb!("Perk.No1.CriticalChance"),]
        );
        assert_eq!(
            style.usable_perks.as_slice(),
            &[sb!("Perk.No1.Slot"), sb!("Perk.No1.Empty"),]
        );
        // assert_eq!(style.skeleton, "*.ozz");
        // assert_eq!(style.actions.as_slice(), &[]);

        assert_eq!(style.icon, "icon");
        assert_eq!(style.view_model, "No1-1.vrm");
    }
}
