use std::fmt;

use crate::utils::Symbol;

//
// StrID
//

pub type StrID = Symbol;

pub fn is_invalid_str_id(id: StrID) -> bool {
    return id.is_empty();
}

//
// NumID
//

pub type NumID = u64;

pub fn is_invalid_num_id(id: NumID) -> bool {
    return id == u64::MAX;
}

pub struct NumIDGenerator {
    counter: NumID,
}

impl NumIDGenerator {
    pub fn new(init: NumID) -> NumIDGenerator {
        return NumIDGenerator { counter: init };
    }

    pub fn generate(&mut self) -> NumID {
        let id = self.counter;
        self.counter += 1;
        return id;
    }
}

//
// IDLevel
//

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct IDLevel {
    pub id: StrID,
    pub level: u32,
}

impl IDLevel {
    pub fn new(id: &StrID, level: u32) -> IDLevel {
        return IDLevel {
            id: id.clone(),
            level,
        };
    }
}

impl From<(StrID, u32)> for IDLevel {
    fn from((id, level): (StrID, u32)) -> Self {
        return IDLevel { id, level };
    }
}

impl Into<(StrID, u32)> for IDLevel {
    fn into(self) -> (StrID, u32) {
        return (self.id, self.level);
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
            return TmplIDLevelVisitor {};
        }
    }

    impl<'de> Visitor<'de> for TmplIDLevelVisitor {
        type Value = IDLevel;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            return formatter.write_str(r#"[id, level] or {"id": id, "level": level}"#);
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, u32);
            let Helper(id, level) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            return Ok(IDLevel { id, level });
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                level: u32,
            }
            let Helper { id, level } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            return Ok(IDLevel { id, level });
        }
    }
};

//
// IDCount
//

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct IDCount {
    pub id: StrID,
    pub count: u32,
}

impl IDCount {
    pub fn new(id: &StrID, count: u32) -> IDCount {
        return IDCount {
            id: id.clone(),
            count,
        };
    }
}

impl From<(StrID, u32)> for IDCount {
    fn from((id, count): (StrID, u32)) -> Self {
        return IDCount { id, count };
    }
}

impl Into<(StrID, u32)> for IDCount {
    fn into(self) -> (StrID, u32) {
        return (self.id, self.count);
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;

    impl<'de> Deserialize<'de> for IDCount {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<IDCount, D::Error> {
            return deserializer.deserialize_any(TmplIDCountVisitor::new());
        }
    }

    pub struct TmplIDCountVisitor {}

    impl TmplIDCountVisitor {
        pub fn new() -> Self {
            return TmplIDCountVisitor {};
        }
    }

    impl<'de> Visitor<'de> for TmplIDCountVisitor {
        type Value = IDCount;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            return formatter.write_str(r#"[id, count] or {"id": id, "count": count}"#);
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, u32);
            let Helper(id, count) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            return Ok(IDCount { id, count });
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                count: u32,
            }
            let Helper { id, count } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            return Ok(IDCount { id, count });
        }
    }
};

//
// IDSymbol
//

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct IDSymbol {
    pub id: StrID,
    pub symbol: Symbol,
}

impl IDSymbol {
    pub fn new(id: &StrID, symbol: &Symbol) -> IDSymbol {
        return IDSymbol {
            id: id.clone(),
            symbol: symbol.clone(),
        };
    }
}

impl From<(StrID, Symbol)> for IDSymbol {
    fn from((id, symbol): (StrID, Symbol)) -> Self {
        return IDSymbol { id, symbol };
    }
}

impl Into<(StrID, Symbol)> for IDSymbol {
    fn into(self) -> (StrID, Symbol) {
        return (self.id, self.symbol);
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
            return TmplIDLevelVisitor {};
        }
    }

    impl<'de> Visitor<'de> for TmplIDLevelVisitor {
        type Value = IDSymbol;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            return formatter.write_str(r#"[id, symbol] or {"id": id, "symbol": symbol}"#);
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper(StrID, Symbol);
            let Helper(id, symbol) = Helper::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            return Ok(IDSymbol { id, symbol });
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper {
                id: StrID,
                symbol: Symbol,
            }
            let Helper { id, symbol } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            return Ok(IDSymbol { id, symbol });
        }
    }
};
