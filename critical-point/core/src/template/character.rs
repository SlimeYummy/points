use crate::template2::attribute::TmplAttribute;
use crate::template2::base::{ArchivedTmplAny, TmplAny, TmplLevelRange, TmplType};
use crate::template2::id::TmplID;
use crate::template2::jewel::TmplJewelSlots;
use crate::utils::{rkyv_self, Num, ShapeCapsule, Symbol, Table};
use cirtical_point_csgen::CsEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, CsEnum)]
#[repr(u8)]
pub enum CharacterType {
    Melee,
    Magic,
    Shot,
}

rkyv_self!(CharacterType);

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TmplCharacter {
    pub id: TmplID,
    pub name: String,
    pub level: TmplLevelRange,
    pub styles: Vec<TmplID>,
    pub equipments: Vec<TmplID>,
    pub bounding_capsule: ShapeCapsule,
    pub skeleton: Symbol,
    pub target_box: Symbol,
}

#[typetag::deserialize(name = "Character")]
impl TmplAny for TmplCharacter {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
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

impl ArchivedTmplAny for ArchivedTmplCharacter {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Character
    }
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TmplStyle {
    pub id: TmplID,
    pub name: String,
    pub character: TmplID,
    pub attributes: Table<TmplAttribute, Vec<Num>>,
    pub slots: Vec<TmplJewelSlots>,
    pub fixed_attributes: TmplFixedAttributes,
    pub perks: Vec<TmplID>,
    #[serde(default)]
    pub usable_perks: Vec<TmplID>,
    pub actions: Vec<TmplID>,
    pub view_model: String,
}

#[typetag::deserialize(name = "Style")]
impl TmplAny for TmplStyle {
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Style
    }
}

impl ArchivedTmplAny for ArchivedTmplStyle {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Style
    }
}

#[derive(Debug, Default, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct TmplFixedAttributes {
    pub damage_reduce_param_1: f32,
    pub damage_reduce_param_2: f32,
    pub guard_damage_ratio_1: f32,
    pub deposture_reduce_param_1: f32,
    pub deposture_reduce_param_2: f32,
    pub guard_deposture_ratio_1: f32,
    pub weak_damage_up: f32,
}

rkyv_self!(TmplFixedAttributes);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template2::database::TmplDatabase;
    use crate::template2::id::id;

    #[test]
    fn test_load_character() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let character = db.find_as::<TmplCharacter>(id!("Character.One")).unwrap();
        assert_eq!(character.id, id!("Character.One"));
        assert_eq!(character.name, "Character One");
        assert_eq!(character.level, [1, 6].into());
        assert_eq!(&character.styles.as_slice(), &[id!("Style.One/1"), id!("Style.One/2")]);
        assert_eq!(
            &character.equipments.as_slice(),
            &[id!("Equipment.No1"), id!("Equipment.No2"), id!("Equipment.No3")]
        );
        assert_eq!(character.bounding_capsule, ShapeCapsule::new(0.5 * 1.35, 0.3));
        assert_eq!(character.skeleton, "skel.ozz");
        assert_eq!(character.target_box, "target_box.json");
    }

    #[test]
    fn test_load_style() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let style = db.find_as::<TmplStyle>(id!("Style.One/1")).unwrap();
        assert_eq!(style.id(), id!("Style.One/1"));
        assert_eq!(style.name, "Character One Type-1");
        assert_eq!(style.character, id!("Character.One"));

        let attrs = &style.attributes;
        assert_eq!(attrs.len(), 11);
        assert_eq!(attrs[0].k, TmplAttribute::MaxHealth);
        assert_eq!(attrs[0].v.as_slice(), &[400.0, 550.0, 700.0, 850.0, 1000.0, 1200.0]);
        assert_eq!(attrs[1].k, TmplAttribute::MaxPosture);
        assert_eq!(attrs[1].v.as_slice(), &[100.0, 115.0, 130.0, 145.0, 160.0, 180.0]);
        assert_eq!(attrs[2].k, TmplAttribute::PostureRecovery);
        assert_eq!(attrs[2].v.as_slice(), &[10.0, 11.0, 12.0, 13.0, 14.0, 15.0]);
        assert_eq!(attrs[3].k, TmplAttribute::PhysicalAttack);
        assert_eq!(attrs[3].v.as_slice(), &[10.0, 15.0, 20.0, 25.0, 30.0, 35.0]);
        assert_eq!(attrs[4].k, TmplAttribute::PhysicalDefense);
        assert_eq!(attrs[4].v.as_slice(), &[15.0, 20.0, 25.0, 30.0, 35.0, 40.0]);
        assert_eq!(attrs[5].k, TmplAttribute::ElementalAttack);
        assert_eq!(attrs[5].v.as_slice(), &[8.0, 12.0, 16.0, 20.0, 24.0, 28.0]);
        assert_eq!(attrs[6].k, TmplAttribute::ElementalDefense);
        assert_eq!(attrs[6].v.as_slice(), &[10.0, 15.0, 20.0, 25.0, 30.0, 35.0]);
        assert_eq!(attrs[7].k, TmplAttribute::ArcaneAttack);
        assert_eq!(attrs[7].v.as_slice(), &[9.0, 13.0, 17.0, 21.0, 25.0, 30.0]);
        assert_eq!(attrs[8].k, TmplAttribute::ArcaneDefense);
        assert_eq!(attrs[8].v.as_slice(), &[5.0, 8.0, 11.0, 14.0, 17.0, 20.0]);
        assert_eq!(attrs[9].k, TmplAttribute::CriticalChance);
        assert_eq!(attrs[9].v.as_slice(), &[0.1; 6]);
        assert_eq!(attrs[10].k, TmplAttribute::CriticalDamage);
        assert_eq!(attrs[10].v.as_slice(), &[0.3; 6]);

        assert_eq!(
            style.slots.as_slice(),
            &[
                TmplJewelSlots::new(0, 2, 2),
                TmplJewelSlots::new(0, 2, 2),
                TmplJewelSlots::new(0, 3, 3),
                TmplJewelSlots::new(2, 3, 3),
                TmplJewelSlots::new(2, 5, 4),
                TmplJewelSlots::new(3, 5, 4),
            ]
        );

        assert_eq!(style.fixed_attributes.damage_reduce_param_1, 0.05);
        assert_eq!(style.fixed_attributes.damage_reduce_param_2, 100.0);
        assert_eq!(style.fixed_attributes.guard_damage_ratio_1, 0.8);
        assert_eq!(style.fixed_attributes.deposture_reduce_param_1, 0.05);
        assert_eq!(style.fixed_attributes.deposture_reduce_param_2, 200.0);
        assert_eq!(style.fixed_attributes.guard_deposture_ratio_1, 0.8);
        assert_eq!(style.fixed_attributes.weak_damage_up, 0.25);

        // assert_eq!(
        //     style.perks.as_slice(),
        //     &[id!("Perk.No1.AttackUp"), id!("Perk.No1.CriticalChance"),]
        // );
        // assert_eq!(
        //     style.usable_perks.as_slice(),
        //     &[id!("Perk.No1.Slot"), id!("Perk.No1.Empty"),]
        // );
        // assert_eq!(style.actions.as_slice(), &[]);
        assert_eq!(style.view_model, "StyleOne-1.vrm");
    }
}
