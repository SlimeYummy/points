use crate::consts::MAX_ENTRY_PLUS;
use crate::template::attribute::TmplAttributeType;
use crate::template::base::{TmplAny, TmplType};
use crate::template::script::TmplScript;
use crate::utils::{Num, StrID, Symbol, Table};

pub type TmplIsPlus = bool;

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplEntry {
    pub id: StrID,
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub color: String,
    pub max_piece: u32,
    #[serde(default)]
    pub attributes: Table<(TmplAttributeType, TmplIsPlus), Num>,
    #[serde(default)]
    pub script: Option<TmplScript>,
    #[serde(default)]
    pub script_args: Table<(Symbol, TmplIsPlus), Num>,
}

#[typetag::deserialize(name = "Entry")]
impl TmplAny for TmplEntry {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Entry
    }
}

impl TmplEntry {
    pub fn max_plus(&self) -> u32 {
        self.max_piece * MAX_ENTRY_PLUS
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplEntryPair {
    pub piece: u32,
    pub plus: u32,
}

impl TmplEntryPair {
    pub fn new(piece: u32, plus: u32) -> Self {
        Self { piece, plus }
    }
}

impl From<(u32, u32)> for TmplEntryPair {
    fn from((piece, plus): (u32, u32)) -> Self {
        Self { piece, plus }
    }
}

impl From<TmplEntryPair> for (u32, u32) {
    fn from(val: TmplEntryPair) -> Self {
        (val.piece, val.plus)
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;
    use std::fmt;

    impl<'de> Deserialize<'de> for TmplEntryPair {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<TmplEntryPair, D::Error> {
            deserializer.deserialize_any(TmplIDLevelVisitor::new())
        }
    }

    pub struct TmplIDLevelVisitor {}

    impl TmplIDLevelVisitor {
        pub fn new() -> Self {
            TmplIDLevelVisitor {}
        }
    }

    impl<'de> Visitor<'de> for TmplIDLevelVisitor {
        type Value = TmplEntryPair;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"[piece, plus] or {"piece": piece, "plus": plus}"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(u32, u32);
            let Helper(piece, plus) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            Ok(TmplEntryPair { piece, plus })
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                piece: u32,
                plus: u32,
            }
            let Helper { piece, plus } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            Ok(TmplEntryPair { piece, plus })
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::s;

    #[test]
    fn test_load_entry() {
        let db = TmplDatabase::new("../test-res").unwrap();

        let e1 = db.find_as::<TmplEntry>(&s!("Entry.Empty")).unwrap();
        assert_eq!(e1.id, "Entry.Empty");
        assert_eq!(e1.name, "");
        assert_eq!(e1.icon, "");
        assert_eq!(e1.color, "#ffffff");
        assert_eq!(e1.max_piece, 1);
        assert_eq!(e1.max_plus(), 3);
        assert!(e1.attributes.is_empty());
        assert!(e1.script.is_none());
        assert!(e1.script_args.is_empty());

        let e2 = db.find_as::<TmplEntry>(&s!("Entry.MaxHealthUp")).unwrap();
        assert_eq!(e2.id, "Entry.MaxHealthUp");
        assert_eq!(e2.name, "MaxHealthUp");
        assert_eq!(e2.max_piece, 4);
        assert_eq!(e2.max_plus(), 12);
        assert_eq!(
            e2.attributes.find(&(TmplAttributeType::MaxHealthUp, false)).unwrap(),
            &[0.0, 0.1, 0.2, 0.3, 0.4]
        );
        assert_eq!(
            e2.attributes.find(&(TmplAttributeType::DefenseUp, true)).unwrap(),
            &[0.0, 0.03, 0.06, 0.09, 0.12]
        );
    }
}
