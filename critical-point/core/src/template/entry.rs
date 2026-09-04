use crate::consts::MAX_ENTRY_PLUS;
use crate::template::attribute::TmplAttribute;
use crate::template::base::impl_tmpl;
// use crate::template2::script::TmplScript;
use crate::utils::{PiecePlus, Table, TmplID, impl_for};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplEntry {
    pub id: TmplID,
    pub name: String,
    pub max_piece: u32,
    #[serde(default)]
    pub attributes: Table<TmplAttribute, Vec<f32>>,
    #[serde(default)]
    pub plus_attributes: Table<TmplAttribute, Vec<f32>>,
    #[serde(default)]
    pub var_indexes: Table<TmplID, Vec<u32>>,
    #[serde(default)]
    pub var_plus_indexes: Table<TmplID, Vec<u32>>,
    // #[serde(default)]
    // pub script: Option<TmplScript>,
    // #[serde(default)]
    // pub script_args: Table2<(Symbol, TmplIsPlus), f32>,
}

impl_tmpl!(TmplEntry, Entry, "Entry");

impl_for!(TmplEntry, ArchivedTmplEntry, {
    #[inline]
    pub fn max_plus(&self) -> u32 {
        self.max_piece * MAX_ENTRY_PLUS
    }

    #[inline]
    pub fn normalize_pair(&self, pair: PiecePlus) -> PiecePlus {
        PiecePlus {
            piece: u32::clamp(pair.piece, 0, self.max_piece.into()),
            plus: u32::clamp(pair.plus, 0, self.max_plus()),
        }
    }

    #[inline]
    pub fn piece_to_index(&self, piece: u32) -> Option<usize> {
        if piece == 0 {
            None
        }
        else {
            Some((piece.clamp(1, self.max_piece.into()) - 1) as usize)
        }
    }

    #[inline]
    pub fn plus_to_index(&self, plus: u32) -> Option<usize> {
        let plus_idx = (plus / MAX_ENTRY_PLUS) as usize;
        if plus_idx == 0 {
            None
        }
        else {
            let max_piece: u32 = self.max_piece.into();
            Some((plus_idx.clamp(1, max_piece as usize) - 1) as usize)
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::attribute::TmplAttribute;
    use crate::template::database::TmplDatabase;
    use crate::utils::{PiecePlus, id};

    #[test]
    fn test_load_entry() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let empty = db.find_as::<TmplEntry>(id!("Entry.Empty")).unwrap();
        assert_eq!(empty.id, id!("Entry.Empty"));
        assert_eq!(empty.name, "");
        assert_eq!(empty.max_piece, 1);
        assert_eq!(empty.max_plus(), 3);
        assert!(empty.attributes.is_empty());
        assert!(empty.plus_attributes.is_empty());
        assert!(empty.var_indexes.is_empty());
        assert!(empty.var_plus_indexes.is_empty());

        let max_health = db.find_as::<TmplEntry>(id!("Entry.MaxHealthUp")).unwrap();
        assert_eq!(max_health.id, id!("Entry.MaxHealthUp"));
        assert_eq!(max_health.name, "MaxHealthUp");
        assert_eq!(max_health.max_piece, 4);
        assert_eq!(max_health.max_plus(), 12);
        assert_eq!(
            max_health
                .attributes
                .find(&TmplAttribute::MaxHealthUp)
                .unwrap()
                .as_slice(),
            &[0.1, 0.2, 0.275, 0.35]
        );
        assert_eq!(
            max_health
                .plus_attributes
                .find(&TmplAttribute::MaxHealthUp)
                .unwrap()
                .as_slice(),
            &[0.035, 0.07, 0.11, 0.15]
        );

        let attack = db.find_as::<TmplEntry>(id!("Entry.AttackUp")).unwrap();
        assert_eq!(attack.name, "AttackUp");
        assert_eq!(attack.max_piece, 5);
        assert_eq!(attack.max_plus(), 15);
        assert_eq!(attack.attributes.find(&TmplAttribute::AttackUp).unwrap().as_slice(), &[
            0.04, 0.08, 0.12, 0.16, 0.2
        ]);
        assert_eq!(
            attack
                .plus_attributes
                .find(&TmplAttribute::AttackUp)
                .unwrap()
                .as_slice(),
            &[0.02, 0.04, 0.06, 0.08, 0.1]
        );

        let defense = db.find_as::<TmplEntry>(id!("Entry.DefenseUp")).unwrap();
        assert_eq!(defense.name, "DefenseUp");
        assert_eq!(defense.max_piece, 5);
        assert_eq!(defense.max_plus(), 15);
        assert_eq!(
            defense.attributes.find(&TmplAttribute::DefenseUp).unwrap().as_slice(),
            &[0.15, 0.3, 0.4, 0.5, 0.6]
        );
        assert_eq!(
            defense
                .plus_attributes
                .find(&TmplAttribute::DefenseUp)
                .unwrap()
                .as_slice(),
            &[0.05, 0.1, 0.2, 0.2, 0.2]
        );
        assert_eq!(
            defense
                .plus_attributes
                .find(&TmplAttribute::MaxHealthUp)
                .unwrap()
                .as_slice(),
            &[0.0, 0.0, 0.0, 0.05, 0.1]
        );

        let elemental_defense = db.find_as::<TmplEntry>(id!("Entry.ElementalDefenseUp")).unwrap();
        assert_eq!(elemental_defense.name, "ElementalDefenseUp");
        assert_eq!(elemental_defense.max_piece, 3);
        assert_eq!(elemental_defense.max_plus(), 9);
        assert_eq!(
            elemental_defense
                .attributes
                .find(&TmplAttribute::ElementalDefenseUp)
                .unwrap()
                .as_slice(),
            &[0.2, 0.4, 0.6]
        );
        assert_eq!(
            elemental_defense
                .plus_attributes
                .find(&TmplAttribute::ElementalDefenseUp)
                .unwrap()
                .as_slice(),
            &[0.05, 0.1, 0.1]
        );
        assert_eq!(
            elemental_defense
                .plus_attributes
                .find(&TmplAttribute::AttackUp)
                .unwrap()
                .as_slice(),
            &[0.01, 0.02, 0.04]
        );

        let critical_chance = db.find_as::<TmplEntry>(id!("Entry.CriticalChance")).unwrap();
        assert_eq!(critical_chance.name, "CriticalChance");
        assert_eq!(critical_chance.max_piece, 4);
        assert_eq!(critical_chance.max_plus(), 12);
        assert_eq!(
            critical_chance
                .attributes
                .find(&TmplAttribute::CriticalChance)
                .unwrap()
                .as_slice(),
            &[0.05, 0.1, 0.15, 0.2]
        );
        assert_eq!(
            critical_chance
                .plus_attributes
                .find(&TmplAttribute::CriticalChance)
                .unwrap()
                .as_slice(),
            &[0.025, 0.05, 0.075, 0.1]
        );

        let critical_damage = db.find_as::<TmplEntry>(id!("Entry.CriticalDamage")).unwrap();
        assert_eq!(critical_damage.name, "CriticalDamage");
        assert_eq!(critical_damage.max_piece, 3);
        assert_eq!(critical_damage.max_plus(), 9);
        assert_eq!(
            critical_damage
                .attributes
                .find(&TmplAttribute::CriticalChance)
                .unwrap()
                .as_slice(),
            &[0.08, 0.16, 0.25]
        );
        assert_eq!(
            critical_damage
                .plus_attributes
                .find(&TmplAttribute::CriticalChance)
                .unwrap()
                .as_slice(),
            &[0.05, 0.1, 0.15]
        );

        let variable = db.find_as::<TmplEntry>(id!("Entry.Variable")).unwrap();
        assert_eq!(variable.name, "Entry Variable");
        assert_eq!(variable.max_piece, 3);
        assert_eq!(variable.max_plus(), 9);
        assert!(variable.attributes.is_empty());
        assert!(variable.plus_attributes.is_empty());
        assert_eq!(variable.var_indexes.len(), 2);
        assert_eq!(
            variable
                .var_indexes
                .find(&id!("#.Entry.Variable^1"))
                .unwrap()
                .as_slice(),
            &[1, 2, 3]
        );
        assert_eq!(
            variable
                .var_indexes
                .find(&id!("#.Entry.Variable^2"))
                .unwrap()
                .as_slice(),
            &[0, 1, 2]
        );
        assert!(variable.var_plus_indexes.is_empty());
    }

    #[test]
    fn test_entry_index_helpers() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let entry = db.find_as::<TmplEntry>(id!("Entry.MaxHealthUp")).unwrap();
        assert_eq!(entry.max_plus(), 12);
        assert_eq!(entry.normalize_pair(PiecePlus::new(0, 20)), PiecePlus::new(0, 12));
        assert_eq!(entry.normalize_pair(PiecePlus::new(6, 2)), PiecePlus::new(4, 2));
        assert_eq!(entry.piece_to_index(0), None);
        assert_eq!(entry.piece_to_index(1), Some(0));
        assert_eq!(entry.piece_to_index(2), Some(1));
        assert_eq!(entry.piece_to_index(4), Some(3));
        assert_eq!(entry.piece_to_index(9), Some(3));
        assert_eq!(entry.plus_to_index(0), None);
        assert_eq!(entry.plus_to_index(1), None);
        assert_eq!(entry.plus_to_index(3), Some(0));
        assert_eq!(entry.plus_to_index(4), Some(0));
        assert_eq!(entry.plus_to_index(6), Some(1));
        assert_eq!(entry.plus_to_index(entry.max_plus()), Some(3));
    }
}
