use crate::consts::MAX_ENTRY_PLUS;
use crate::template::base::{TmplAny, TmplRare, TmplType};
use crate::template::entry::TmplEntryPair;
use crate::template::slot::TmplSlotType;
use crate::utils::StrID;

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplJewel {
    pub id: StrID,
    pub slot_type: TmplSlotType,
    pub rare: TmplRare,
    pub entry: StrID,
    pub piece: u32,
    pub sub_entry: Option<StrID>,
    pub sub_piece: Option<u32>,
}

#[typetag::deserialize(name = "Jewel")]
impl TmplAny for TmplJewel {
    fn id(&self) -> StrID {
        self.id.clone()
    }

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

    pub fn sub(&self) -> Option<(StrID, TmplEntryPair)> {
        if let (Some(entry), Some(piece)) = (self.sub_entry.clone(), self.sub_piece) {
            let pp = TmplEntryPair::new(piece, self.sub_plus(1).unwrap());
            Some((entry, pp))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_TEMPLATE_PATH;
    use crate::template::database::TmplDatabase;
    use crate::utils::sb;

    #[test]
    fn test_load_jewel() {
        let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();

        let j1 = db.find_as::<TmplJewel>(&sb!("Jewel.DefenseUp.Variant1")).unwrap();
        assert_eq!(j1.id, "Jewel.DefenseUp.Variant1");
        assert_eq!(j1.slot_type, TmplSlotType::Defense);
        assert_eq!(j1.rare, TmplRare::Rare1);
        assert_eq!(j1.entry, "Entry.DefenseUp");
        assert_eq!(j1.piece, 1);
        assert_eq!(j1.sub_entry, None);
        assert_eq!(j1.sub_piece, None);

        let j1 = db.find_as::<TmplJewel>(&sb!("Jewel.AttackUp.VariantX")).unwrap();
        assert_eq!(j1.id, "Jewel.AttackUp.VariantX");
        assert_eq!(j1.slot_type, TmplSlotType::Special);
        assert_eq!(j1.rare, TmplRare::Rare3);
        assert_eq!(j1.entry, "Entry.AttackUp");
        assert_eq!(j1.piece, 2);
        assert_eq!(j1.sub_entry.clone().unwrap(), "Entry.MaxHealthUp");
        assert_eq!(j1.sub_piece.unwrap(), 1);
    }
}
