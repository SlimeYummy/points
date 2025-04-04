use crate::consts::MAX_ENTRY_PLUS;
use crate::template2::base::{ArchivedTmplAny, TmplAny, TmplRare, TmplType};
use crate::template2::entry::TmplEntryPair;
use crate::template2::id::TmplID;
use crate::utils::{rkyv_self, serde_by};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplJewelSlot {
    Special = 3,
    Attack = 1,
    Defense = 2,
}

rkyv_self!(TmplJewelSlot);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TmplJewelSlots {
    pub special: u8,
    pub attack: u8,
    pub defense: u8,
}

rkyv_self!(TmplJewelSlots);
serde_by!(TmplJewelSlots, [u8; 3], TmplJewelSlots::from, TmplJewelSlots::to_array);

impl TmplJewelSlots {
    #[inline]
    pub fn new(special: u8, attack: u8, defense: u8) -> TmplJewelSlots {
        TmplJewelSlots {
            special,
            attack,
            defense,
        }
    }

    #[inline]
    pub fn merge(&self, other: &TmplJewelSlots) -> TmplJewelSlots {
        TmplJewelSlots {
            special: self.special + other.special,
            attack: self.attack + other.attack,
            defense: self.defense + other.defense,
        }
    }

    #[inline]
    pub fn append(&mut self, other: &TmplJewelSlots) {
        *self = self.merge(other);
    }

    #[inline]
    pub fn to_array(&self) -> [u8; 3] {
        [self.special, self.attack, self.defense]
    }

    #[inline]
    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (self.special, self.attack, self.defense)
    }
}

impl From<(u8, u8, u8)> for TmplJewelSlots {
    #[inline]
    fn from((special, attack, defense): (u8, u8, u8)) -> Self {
        TmplJewelSlots {
            special,
            attack,
            defense,
        }
    }
}

impl From<TmplJewelSlots> for (u8, u8, u8) {
    #[inline]
    fn from(val: TmplJewelSlots) -> Self {
        (val.special, val.attack, val.defense)
    }
}

impl From<[u8; 3]> for TmplJewelSlots {
    #[inline]
    fn from([special, attack, defense]: [u8; 3]) -> Self {
        TmplJewelSlots {
            special,
            attack,
            defense,
        }
    }
}

impl From<TmplJewelSlots> for [u8; 3] {
    #[inline]
    fn from(val: TmplJewelSlots) -> Self {
        [val.special, val.attack, val.defense]
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TmplJewel {
    pub id: TmplID,
    pub slot: TmplJewelSlot,
    pub rare: TmplRare,
    pub entry: TmplID,
    pub piece: u32,
    pub sub_entry: Option<TmplID>,
    pub sub_piece: Option<u32>,
}

#[typetag::deserialize(name = "Jewel")]
impl TmplAny for TmplJewel {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Jewel
    }
}

impl TmplJewel {
    pub fn plus(&self, level: u32) -> u32 {
        self.piece * u32::min(level, MAX_ENTRY_PLUS)
    }

    pub fn sub_plus(&self, level: u32) -> Option<u32> {
        self.sub_piece.map(|piece| piece * u32::min(level, MAX_ENTRY_PLUS))
    }

    pub fn sub(&self) -> Option<(TmplID, TmplEntryPair)> {
        if let (Some(entry), Some(piece)) = (self.sub_entry.clone(), self.sub_piece) {
            let pp = TmplEntryPair::new(piece, self.sub_plus(1).unwrap());
            Some((entry, pp))
        } else {
            None
        }
    }
}

impl ArchivedTmplAny for ArchivedTmplJewel {
    fn id(&self) -> TmplID {
        TmplID::from(self.id).clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template2::database::TmplDatabase;
    use crate::template2::id::id;

    #[test]
    fn test_load_jewel() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let j1 = db.find_as::<TmplJewel>(id!("Jewel.DefenseUp/1")).unwrap();
        assert_eq!(j1.id, id!("Jewel.DefenseUp/1"));
        assert_eq!(j1.slot, TmplJewelSlot::Defense);
        assert_eq!(j1.rare, TmplRare::Rare1);
        assert_eq!(j1.entry, id!("Entry.DefenseUp"));
        assert_eq!(j1.piece, 1);
        assert!(j1.sub_entry.is_none());
        assert!(j1.sub_piece.is_none());

        let j1 = db.find_as::<TmplJewel>(id!("Jewel.SuperCritical")).unwrap();
        assert_eq!(j1.id, id!("Jewel.SuperCritical"));
        assert_eq!(j1.slot, TmplJewelSlot::Special);
        assert_eq!(j1.rare, TmplRare::Rare3);
        assert_eq!(j1.entry, id!("Entry.CriticalChance"));
        assert_eq!(j1.piece, 2);
        assert_eq!(j1.sub_entry.clone().unwrap(), id!("Entry.CriticalDamage"));
        assert_eq!(j1.sub_piece.unwrap(), 1);
    }
}
