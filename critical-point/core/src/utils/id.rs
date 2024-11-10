use cirtical_point_csgen::CsIn;
use std::fmt;

use crate::utils::Symbol;

//
// StrID
//

pub type StrID = Symbol;

pub fn is_invalid_str_id(id: StrID) -> bool {
    id.is_empty()
}

//
// NumID
//

pub type NumID = u64;

pub fn is_invalid_num_id(id: NumID) -> bool {
    id == u64::MAX
}

//
// IDLevel
//

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    CsIn,
)]
pub struct IDLevel {
    pub id: StrID,
    pub level: u32,
}

impl IDLevel {
    pub fn new(id: &StrID, level: u32) -> IDLevel {
        IDLevel { id: id.clone(), level }
    }
}

impl From<(StrID, u32)> for IDLevel {
    fn from((id, level): (StrID, u32)) -> Self {
        IDLevel { id, level }
    }
}

impl From<IDLevel> for (StrID, u32) {
    fn from(val: IDLevel) -> Self {
        (val.id, val.level)
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;

    impl<'de> Deserialize<'de> for IDLevel {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IDLevel, D::Error> {
            return deserializer.deserialize_any(TmplIDLevelVisitor::new());
        }
    }

    pub struct TmplIDLevelVisitor {}

    impl TmplIDLevelVisitor {
        pub fn new() -> Self {
            TmplIDLevelVisitor {}
        }
    }

    impl<'de> Visitor<'de> for TmplIDLevelVisitor {
        type Value = IDLevel;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"[id, level] or {"id": id, "level": level}"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, u32);
            let Helper(id, level) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            Ok(IDLevel { id, level })
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                level: u32,
            }
            let Helper { id, level } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            Ok(IDLevel { id, level })
        }
    }
};

//
// IDPlus
//

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    CsIn,
)]
pub struct IDPlus {
    pub id: StrID,
    pub plus: u32,
}

impl IDPlus {
    pub fn new(id: &StrID, plus: u32) -> IDPlus {
        IDPlus { id: id.clone(), plus }
    }
}

impl From<(StrID, u32)> for IDPlus {
    fn from((id, plus): (StrID, u32)) -> Self {
        IDPlus { id, plus }
    }
}

impl From<IDPlus> for (StrID, u32) {
    fn from(val: IDPlus) -> Self {
        (val.id, val.plus)
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;

    impl<'de> Deserialize<'de> for IDPlus {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IDPlus, D::Error> {
            return deserializer.deserialize_any(TmplIDCountVisitor::new());
        }
    }

    pub struct TmplIDCountVisitor {}

    impl TmplIDCountVisitor {
        pub fn new() -> Self {
            TmplIDCountVisitor {}
        }
    }

    impl<'de> Visitor<'de> for TmplIDCountVisitor {
        type Value = IDPlus;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"[id, plus] or {"id": id, "plus": plus}"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, u32);
            let Helper(id, plus) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            Ok(IDPlus { id, plus })
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                plus: u32,
            }
            let Helper { id, plus } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            Ok(IDPlus { id, plus })
        }
    }
};

//
// IDSymbol
//

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    CsIn,
)]
pub struct IDSymbol {
    pub id: StrID,
    pub symbol: Symbol,
}

impl IDSymbol {
    pub fn new(id: &StrID, symbol: &Symbol) -> IDSymbol {
        IDSymbol {
            id: id.clone(),
            symbol: symbol.clone(),
        }
    }
}

impl From<(StrID, Symbol)> for IDSymbol {
    fn from((id, symbol): (StrID, Symbol)) -> Self {
        IDSymbol { id, symbol }
    }
}

impl From<IDSymbol> for (StrID, Symbol) {
    fn from(val: IDSymbol) -> Self {
        (val.id, val.symbol)
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;

    impl<'de> Deserialize<'de> for IDSymbol {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IDSymbol, D::Error> {
            return deserializer.deserialize_any(TmplIDLevelVisitor::new());
        }
    }

    pub struct TmplIDLevelVisitor {}

    impl TmplIDLevelVisitor {
        pub fn new() -> Self {
            TmplIDLevelVisitor {}
        }
    }

    impl<'de> Visitor<'de> for TmplIDLevelVisitor {
        type Value = IDSymbol;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"[id, symbol] or {"id": id, "symbol": symbol}"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, Symbol);
            let Helper(id, symbol) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            Ok(IDSymbol { id, symbol })
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                symbol: Symbol,
            }
            let Helper { id, symbol } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            Ok(IDSymbol { id, symbol })
        }
    }
};
