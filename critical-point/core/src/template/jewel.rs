use rkyv::option::ArchivedOption;

use crate::consts::MAX_ENTRY_PLUS;
use crate::template::base::impl_tmpl;
use crate::utils::{rkyv_self, PiecePlus, RareLevel, TmplID};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplJewelSlot {
    Special = 3,
    Attack = 1,
    Defense = 2,
}

rkyv_self!(TmplJewelSlot);

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplJewel {
    pub id: TmplID,
    pub slot: TmplJewelSlot,
    pub rare: RareLevel,
    pub entry: TmplID,
    pub piece: u32,
    pub sub_entry: Option<TmplID>,
    pub sub_piece: Option<u32>,
}

impl_tmpl!(TmplJewel, Jewel, "Jewel");

impl TmplJewel {
    pub fn calc_plus(&self, plus: u32) -> u32 {
        self.piece * u32::min(plus, MAX_ENTRY_PLUS)
    }

    pub fn calc_sub_plus(&self, plus: u32) -> Option<u32> {
        self.sub_piece.map(|piece| piece * u32::min(plus, MAX_ENTRY_PLUS))
    }

    pub fn calc_sub(&self, plus: u32) -> Option<(TmplID, PiecePlus)> {
        if let (Some(entry), Some(piece)) = (self.sub_entry.clone(), self.sub_piece) {
            Some((entry, PiecePlus::new(piece, self.calc_sub_plus(plus).unwrap())))
        }
        else {
            None
        }
    }
}

impl ArchivedTmplJewel {
    pub fn calc_plus(&self, plus: u32) -> u32 {
        self.piece * u32::min(plus, MAX_ENTRY_PLUS)
    }

    pub fn calc_sub_plus(&self, plus: u32) -> Option<u32> {
        return match self.sub_piece {
            ArchivedOption::Some(piece) => Some(piece * u32::min(plus, MAX_ENTRY_PLUS)),
            ArchivedOption::None => None,
        };
    }

    pub fn calc_sub(&self, plus: u32) -> Option<(TmplID, PiecePlus)> {
        let entry = match self.sub_entry {
            ArchivedOption::Some(id) => TmplID::from(id),
            ArchivedOption::None => return None,
        };
        let piece = match self.sub_piece {
            ArchivedOption::Some(piece) => piece,
            ArchivedOption::None => return None,
        };
        Some((entry, PiecePlus::new(piece.into(), self.calc_sub_plus(plus).unwrap())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_jewel() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let j1 = db.find_as::<TmplJewel>(id!("Jewel.DefenseUp^1")).unwrap();
        assert_eq!(j1.id, id!("Jewel.DefenseUp^1"));
        assert_eq!(j1.slot, TmplJewelSlot::Defense);
        assert_eq!(j1.rare, RareLevel::Rare1);
        assert_eq!(j1.entry, id!("Entry.DefenseUp"));
        assert_eq!(j1.piece, 1);
        assert!(j1.sub_entry.is_none());
        assert!(j1.sub_piece.is_none());

        let j1 = db.find_as::<TmplJewel>(id!("Jewel.SuperCritical")).unwrap();
        assert_eq!(j1.id, id!("Jewel.SuperCritical"));
        assert_eq!(j1.slot, TmplJewelSlot::Special);
        assert_eq!(j1.rare, RareLevel::Rare3);
        assert_eq!(j1.entry, id!("Entry.CriticalChance"));
        assert_eq!(j1.piece, 2);
        assert_eq!(j1.sub_entry.clone().unwrap(), id!("Entry.CriticalDamage"));
        assert_eq!(j1.sub_piece.unwrap(), 1);
    }
}
