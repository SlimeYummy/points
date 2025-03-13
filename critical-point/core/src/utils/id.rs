use cirtical_point_csgen::CsIn;
use std::fmt;

use crate::consts::MAX_PLAYER;
use crate::utils::{ASymbol, Symbol};

use super::rkyv_self;

//
// StrID
//

pub type StrID = Symbol;
pub type AStrID = ASymbol;

#[inline]
pub fn is_invalid_str_id(id: StrID) -> bool {
    id.is_empty()
}

//
// NumID
//

pub type NumID = u64;

#[inline]
pub fn is_invalid_num_id(id: NumID) -> bool {
    id == u64::MAX
}

pub const GAME_ID: NumID = 1;
pub const STAGE_ID: NumID = 2;
pub const MIN_PLAYER_ID: NumID = 100;
pub const MAX_PLAYER_ID: NumID = MIN_PLAYER_ID + (MAX_PLAYER as u64);

#[inline]
pub fn is_valid_player_id(id: NumID) -> bool {
    id >= MIN_PLAYER_ID && id <= MAX_PLAYER_ID
}

#[inline]
pub fn is_invalid_player_id(id: NumID) -> bool {
    id < MIN_PLAYER_ID || id > MAX_PLAYER_ID
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
pub struct IDLevel2 {
    pub id: StrID,
    pub level: u32,
}

impl IDLevel2 {
    #[inline]
    pub fn new(id: &StrID, level: u32) -> IDLevel2 {
        IDLevel2 { id: id.clone(), level }
    }
}

impl From<(StrID, u32)> for IDLevel2 {
    #[inline]
    fn from((id, level): (StrID, u32)) -> Self {
        IDLevel2 { id, level }
    }
}

impl From<IDLevel2> for (StrID, u32) {
    #[inline]
    fn from(val: IDLevel2) -> Self {
        (val.id, val.level)
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;

    impl<'de> Deserialize<'de> for IDLevel2 {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IDLevel2, D::Error> {
            return deserializer.deserialize_any(IDLevelVisitor::new());
        }
    }

    pub struct IDLevelVisitor {}

    impl IDLevelVisitor {
        pub fn new() -> Self {
            IDLevelVisitor {}
        }
    }

    impl<'de> Visitor<'de> for IDLevelVisitor {
        type Value = IDLevel2;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"[id, level] or {"id": id, "level": level}"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, u32);
            let Helper(id, level) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            Ok(IDLevel2 { id, level })
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                level: u32,
            }
            let Helper { id, level } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            Ok(IDLevel2 { id, level })
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
pub struct IDPlus2 {
    pub id: StrID,
    pub plus: u32,
}

impl IDPlus2 {
    #[inline]
    pub fn new(id: &StrID, plus: u32) -> IDPlus2 {
        IDPlus2 { id: id.clone(), plus }
    }
}

impl From<(StrID, u32)> for IDPlus2 {
    #[inline]
    fn from((id, plus): (StrID, u32)) -> Self {
        IDPlus2 { id, plus }
    }
}

impl From<IDPlus2> for (StrID, u32) {
    #[inline]
    fn from(val: IDPlus2) -> Self {
        (val.id, val.plus)
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;

    impl<'de> Deserialize<'de> for IDPlus2 {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IDPlus2, D::Error> {
            return deserializer.deserialize_any(IDCountVisitor::new());
        }
    }

    pub struct IDCountVisitor {}

    impl IDCountVisitor {
        pub fn new() -> Self {
            IDCountVisitor {}
        }
    }

    impl<'de> Visitor<'de> for IDCountVisitor {
        type Value = IDPlus2;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"[id, plus] or {"id": id, "plus": plus}"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, u32);
            let Helper(id, plus) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            Ok(IDPlus2 { id, plus })
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                plus: u32,
            }
            let Helper { id, plus } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            Ok(IDPlus2 { id, plus })
        }
    }
};

//
// IDSymbol
//

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize,
)]
pub struct IDSymbol {
    pub id: StrID,
    pub symbol: Symbol,
}

impl IDSymbol {
    #[inline]
    pub fn new(id: &StrID, symbol: &Symbol) -> IDSymbol {
        IDSymbol {
            id: id.clone(),
            symbol: symbol.clone(),
        }
    }
}

impl From<(StrID, Symbol)> for IDSymbol {
    #[inline]
    fn from((id, symbol): (StrID, Symbol)) -> Self {
        IDSymbol { id, symbol }
    }
}

impl From<IDSymbol> for (StrID, Symbol) {
    #[inline]
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
            return deserializer.deserialize_any(IDSymbolVisitor::new());
        }
    }

    pub struct IDSymbolVisitor {}

    impl IDSymbolVisitor {
        pub fn new() -> Self {
            IDSymbolVisitor {}
        }
    }

    impl<'de> Visitor<'de> for IDSymbolVisitor {
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
