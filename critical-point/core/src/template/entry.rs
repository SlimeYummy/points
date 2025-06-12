use crate::consts::MAX_ENTRY_PLUS;
use crate::template::attribute::TmplAttribute;
use crate::template::base::impl_tmpl;
// use crate::template2::script::TmplScript;
use crate::utils::{impl_for, PiecePlus, Table, TmplID};

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
    pub plus_var_indexes: Table<TmplID, Vec<u32>>,
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
    pub fn piece_to_index(&self, piece: u32) -> usize {
        (piece.clamp(1, self.max_piece.into()) - 1) as usize
    }

    #[inline]
    pub fn plus_to_index(&self, plus: u32) -> usize {
        (plus / MAX_ENTRY_PLUS).min(self.max_piece.into()) as usize
    }
});

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::consts::TEST_TEMPLATE_PATH;
    // use crate::template2::database::TmplDatabase;
    // use crate::utils::sb;

    // #[test]
    // fn test_load_entry() {
    //     let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();

    //     let e1 = db.find_as::<TmplEntry>(&sb!("Entry.Empty")).unwrap();
    //     assert_eq!(e1.id, "Entry.Empty");
    //     assert_eq!(e1.name, "");
    //     assert_eq!(e1.icon, "");
    //     assert_eq!(e1.color, "#ffffff");
    //     assert_eq!(e1.max_piece, 1);
    //     assert_eq!(e1.max_plus(), 3);
    //     assert!(e1.attributes.is_empty());
    //     assert!(e1.script.is_none());
    //     assert!(e1.script_args.is_empty());

    //     let e2 = db.find_as::<TmplEntry>(&sb!("Entry.MaxHealthUp")).unwrap();
    //     assert_eq!(e2.id, "Entry.MaxHealthUp");
    //     assert_eq!(e2.name, "MaxHealthUp");
    //     assert_eq!(e2.max_piece, 4);
    //     assert_eq!(e2.max_plus(), 12);
    //     assert_eq!(
    //         e2.attributes.find(&(TmplAttribute::MaxHealthUp, false)).unwrap(),
    //         &[0.0, 0.1, 0.2, 0.3, 0.4]
    //     );
    //     assert_eq!(
    //         e2.attributes.find(&(TmplAttribute::DefenseUp, true)).unwrap(),
    //         &[0.0, 0.03, 0.06, 0.09, 0.12]
    //     );
    // }
}
