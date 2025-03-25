use crate::consts::MAX_ENTRY_PLUS;
use crate::template2::attribute::TmplAttribute;
use crate::template2::base::{ArchivedTmplAny, TmplAny, TmplType};
use crate::template2::id::TmplID;
// use crate::template2::script::TmplScript;
use crate::utils::{rkyv_self, serde_by, Num, Table};

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TmplEntry {
    pub id: TmplID,
    pub name: String,
    pub max_piece: u32,
    #[serde(default)]
    pub attributes: Table<TmplAttribute, Vec<Num>>,
    #[serde(default)]
    pub plus_attributes: Table<TmplAttribute, Vec<Num>>,
    // #[serde(default)]
    // pub script: Option<TmplScript>,
    // #[serde(default)]
    // pub script_args: Table2<(Symbol, TmplIsPlus), Num>,
}

#[typetag::deserialize(name = "Entry")]
impl TmplAny for TmplEntry {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Entry
    }
}

impl TmplEntry {
    #[inline]
    pub fn max_plus(&self) -> u32 {
        self.max_piece * MAX_ENTRY_PLUS
    }
}

impl ArchivedTmplAny for ArchivedTmplEntry {
    #[inline]
    fn id(&self) -> TmplID {
        TmplID::from(self.id).clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Entry
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TmplEntryPair {
    pub piece: u32,
    pub plus: u32,
}

rkyv_self!(TmplEntryPair);
serde_by!(TmplEntryPair, (u32, u32), TmplEntryPair::from, TmplEntryPair::to_tuple);

impl TmplEntryPair {
    #[inline]
    pub fn new(piece: u32, plus: u32) -> Self {
        Self { piece, plus }
    }

    #[inline]
    pub fn to_tuple(&self) -> (u32, u32) {
        (self.piece, self.plus)
    }
}

impl From<(u32, u32)> for TmplEntryPair {
    #[inline]
    fn from((piece, plus): (u32, u32)) -> Self {
        Self { piece, plus }
    }
}

impl From<TmplEntryPair> for (u32, u32) {
    #[inline]
    fn from(val: TmplEntryPair) -> Self {
        (val.piece, val.plus)
    }
}

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
