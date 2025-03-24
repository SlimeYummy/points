use std::collections::HashMap;

use crate::consts::MAX_ENTRY_PLUS;
use crate::template::base::{TmplAny, TmplRare, TmplType};
use crate::utils::StrID;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplAccessoryPool {
    A,
    B,
    AB,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplAccessoryPattern {
    pub id: StrID,
    pub pattern: Vec<TmplAccessoryPool>,
    pub max_level: u32,
    pub a_pool: HashMap<StrID, u32>,
    pub b_pool: HashMap<StrID, u32>,
}

#[typetag::deserialize(name = "AccessoryPattern")]
impl TmplAny for TmplAccessoryPattern {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::AccessoryPattern
    }
}

impl TmplAccessoryPattern {
    pub fn main_plus(&self, level: u32, piece: u32) -> u32 {
        let count = (self.pattern.len() + 1) as u32;
        u32::min(level / count, MAX_ENTRY_PLUS) * piece
    }

    pub fn pool_plus(&self, level: u32, pos: u32) -> u32 {
        let count = (self.pattern.len() + 1) as u32;
        if level % count > pos {
            u32::min(level / count + 1, MAX_ENTRY_PLUS)
        } else {
            u32::min(level / count, MAX_ENTRY_PLUS)
        }
    }
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplAccessory {
    pub id: StrID,
    pub pattern: StrID,
    pub rare: TmplRare,
    pub entry: StrID,
    pub piece: u32,
}

#[typetag::deserialize(name = "Accessory")]
impl TmplAny for TmplAccessory {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Accessory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_TEMPLATE_PATH;
    use crate::template::database::TmplDatabase;
    use crate::utils::sb;

    #[test]
    fn test_load_accessory() {
        let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();

        let p1 = db
            .find_as::<TmplAccessoryPattern>(&sb!("AccessoryPattern.Rare1"))
            .unwrap();
        assert_eq!(p1.id, "AccessoryPattern.Rare1");
        assert_eq!(p1.pattern, &[TmplAccessoryPool::B, TmplAccessoryPool::B]);
        assert!(p1.a_pool.is_empty());
        assert_eq!(p1.b_pool.len(), 2);
        assert_eq!(*p1.b_pool.get(&sb!("Entry.DefenseUp")).unwrap(), 10);
        assert_eq!(*p1.b_pool.get(&sb!("Entry.CutDefenseUp")).unwrap(), 10);

        let p2 = db
            .find_as::<TmplAccessoryPattern>(&sb!("AccessoryPattern.Rare3"))
            .unwrap();
        assert_eq!(p2.id, "AccessoryPattern.Rare3");
        assert_eq!(
            p2.pattern,
            &[
                TmplAccessoryPool::A,
                TmplAccessoryPool::AB,
                TmplAccessoryPool::AB,
                TmplAccessoryPool::B
            ]
        );
        assert_eq!(p2.a_pool.len(), 2);
        assert_eq!(*p2.a_pool.get(&sb!("Entry.AttackUp")).unwrap(), 10);
        assert_eq!(*p2.a_pool.get(&sb!("Entry.CriticalChance")).unwrap(), 10);
        assert_eq!(p2.b_pool.len(), 3);

        let a1 = db
            .find_as::<TmplAccessory>(&sb!("Accessory.CriticalChance.Variant2"))
            .unwrap();
        assert_eq!(a1.pattern, "AccessoryPattern.Rare2");
        assert_eq!(a1.rare, TmplRare::Rare2);
        assert_eq!(a1.entry, "Entry.CriticalChance");
        assert_eq!(a1.piece, 2);

        let a1 = db
            .find_as::<TmplAccessory>(&sb!("Accessory.AttackUp.Variant3"))
            .unwrap();
        assert_eq!(a1.pattern, "AccessoryPattern.Rare3");
        assert_eq!(a1.rare, TmplRare::Rare3);
        assert_eq!(a1.entry, "Entry.AttackUp");
        assert_eq!(a1.piece, 3);
    }
}
