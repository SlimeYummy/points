use crate::template::attribute::TmplAttribute;
use crate::template::base::impl_tmpl;
use crate::utils::{impl_for, JewelSlots, PiecePlus, Table, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplPerk {
    pub id: TmplID,
    pub name: String,
    pub character: TmplID,
    pub style: TmplID,
    pub usable_styles: Vec<TmplID>,
    pub max_level: u32,
    #[serde(default)]
    pub parents: Table<TmplID, u32>,
    #[serde(default)]
    pub attributes: Table<TmplAttribute, Vec<f32>>,
    #[serde(default)]
    pub slots: Vec<JewelSlots>,
    #[serde(default)]
    pub entries: Table<TmplID, Vec<PiecePlus>>,
    #[serde(default)]
    pub var_indexes: Table<TmplID, Vec<u32>>,
}

impl_tmpl!(TmplPerk, Perk, "Perk");

impl_for!(TmplPerk, ArchivedTmplPerk, {
    #[inline]
    pub fn level_to_index(&self, level: u32) -> usize {
        (level.clamp(1, self.max_level.into()) - 1) as usize
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_perk() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let perk1 = db.find_as::<TmplPerk>(id!("Perk.One.NormalAttack.Branch")).unwrap();
        assert_eq!(perk1.id, id!("Perk.One.NormalAttack.Branch"));
        assert_eq!(perk1.name, "Normal Attack Branch");
        assert_eq!(perk1.character, id!("Character.One"));
        assert_eq!(perk1.style, id!("Style.One^1"));
        assert_eq!(perk1.usable_styles.as_slice(), &[id!("Style.One^1")]);
        assert_eq!(perk1.max_level, 2);
        assert!(perk1.parents.is_empty());
        assert!(perk1.attributes.is_empty());
        assert!(perk1.slots.is_empty());
        assert!(perk1.entries.is_empty());
        assert_eq!(perk1.var_indexes.len(), 1);
        assert_eq!(perk1.var_indexes[0].k, id!("#.One.NormalAttack.Branch"));
        assert_eq!(perk1.var_indexes[0].v.as_slice(), &[1, 2]);

        let perk2 = db.find_as::<TmplPerk>(id!("Perk.One.AttackUp")).unwrap();
        assert_eq!(perk2.max_level, 3);
        assert_eq!(perk2.attributes.len(), 1);
        assert_eq!(perk2.attributes[0].k, TmplAttribute::AttackUp);
        assert_eq!(perk2.attributes[0].v.as_slice(), &[0.1, 0.15, 0.2]);
        assert!(perk2.var_indexes.is_empty());

        let perk3 = db.find_as::<TmplPerk>(id!("Perk.One.FinalPerk")).unwrap();
        assert_eq!(perk3.parents.len(), 1);
        assert_eq!(perk3.parents[0].k, id!("Perk.One.AttackUp"));
        assert_eq!(perk3.parents[0].v, 3);
        assert_eq!(perk3.max_level, 1);
        assert_eq!(perk3.slots.as_slice(), &[JewelSlots::new(1, 0, 0),]);
        assert_eq!(perk3.entries.len(), 2);
        assert_eq!(perk3.entries[0].k, id!("Entry.AttackUp"));
        assert_eq!(perk3.entries[0].v.as_slice(), &[PiecePlus::new(1, 3)]);
        assert_eq!(perk3.entries[1].k, id!("Entry.DefenseUp"));
        assert_eq!(perk3.entries[1].v.as_slice(), &[PiecePlus::new(1, 3)]);
    }
}
