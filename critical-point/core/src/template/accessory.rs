use crate::consts::MAX_ENTRY_PLUS;
use crate::template2::base::{ArchivedTmplAny, TmplAny, TmplRare, TmplType};
use crate::template2::id::{TmplID, TmplHashMap};
use crate::utils::rkyv_self;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplAccessoryPattern {
    A,
    B,
    AB,
}

rkyv_self!(TmplAccessoryPattern);

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TmplAccessoryPool {
    pub id: TmplID,
    pub patterns: Vec<TmplAccessoryPattern>,
    pub max_level: u32,
    pub a_entries: TmplHashMap<u32>,
    pub b_entries: TmplHashMap<u32>,
}

#[typetag::deserialize(name = "AccessoryPool")]
impl TmplAny for TmplAccessoryPool {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::AccessoryPool
    }
}

impl TmplAccessoryPool {
    #[inline]
    pub fn main_plus(&self, level: u32, piece: u32) -> u32 {
        let count = (self.patterns.len() + 1) as u32;
        u32::min(level / count, MAX_ENTRY_PLUS) * piece
    }

    #[inline]
    pub fn pool_plus(&self, level: u32, pos: u32) -> u32 {
        let count = (self.patterns.len() + 1) as u32;
        if level % count > pos {
            u32::min(level / count + 1, MAX_ENTRY_PLUS)
        } else {
            u32::min(level / count, MAX_ENTRY_PLUS)
        }
    }
}

impl ArchivedTmplAny for ArchivedTmplAccessoryPool {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Accessory
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TmplAccessory {
    pub id: TmplID,
    pub pool: TmplID,
    pub rare: TmplRare,
    pub entry: TmplID,
    pub piece: u32,
}

#[typetag::deserialize(name = "Accessory")]
impl TmplAny for TmplAccessory {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Accessory
    }
}

impl ArchivedTmplAny for ArchivedTmplAccessory {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Accessory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template2::database::TmplDatabase;
    use crate::template2::id::id;

    #[test]
    fn test_load_accessory_pool() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let p1 = db.find_as::<TmplAccessoryPool>(id!("AccessoryPool.Rare1")).unwrap();
        assert_eq!(p1.id, id!("AccessoryPool.Rare1"));
        assert_eq!(
            p1.patterns.as_slice(),
            &[TmplAccessoryPattern::B, TmplAccessoryPattern::B]
        );
        assert!(p1.a_entries.is_empty());
        assert_eq!(p1.b_entries.len(), 2);
        assert_eq!(*p1.b_entries.get(&id!("Entry.DefenseUp")).unwrap(), 10);
        assert_eq!(*p1.b_entries.get(&id!("Entry.ElementalDefenseUp")).unwrap(), 10);

        let p2 = db.find_as::<TmplAccessoryPool>(id!("AccessoryPool.Rare3")).unwrap();
        assert_eq!(p2.id, id!("AccessoryPool.Rare3"));
        assert_eq!(
            p2.patterns.as_slice(),
            &[
                TmplAccessoryPattern::A,
                TmplAccessoryPattern::AB,
                TmplAccessoryPattern::AB,
                TmplAccessoryPattern::B
            ]
        );
        assert_eq!(p2.a_entries.len(), 2);
        assert_eq!(*p2.a_entries.get(&id!("Entry.AttackUp")).unwrap(), 10);
        assert_eq!(*p2.a_entries.get(&id!("Entry.CriticalChance")).unwrap(), 10);
        assert_eq!(p2.b_entries.len(), 3);
    }

    #[test]
    fn test_load_accessory() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let a1 = db.find_as::<TmplAccessory>(id!("Accessory.CriticalChance")).unwrap();
        assert_eq!(a1.pool, id!("AccessoryPool.Rare2"));
        assert_eq!(a1.rare, TmplRare::Rare2);
        assert_eq!(a1.entry, id!("Entry.CriticalChance"));
        assert_eq!(a1.piece, 2);

        let a1 = db.find_as::<TmplAccessory>(id!("Accessory.AttackUp/3")).unwrap();
        assert_eq!(a1.pool, id!("AccessoryPool.Rare3"));
        assert_eq!(a1.rare, TmplRare::Rare3);
        assert_eq!(a1.entry, id!("Entry.AttackUp"));
        assert_eq!(a1.piece, 3);
    }
}
