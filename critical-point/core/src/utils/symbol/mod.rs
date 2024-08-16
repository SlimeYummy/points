mod base;
mod client;
mod server;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::str::FromStr;

use crate::utils::{IdentityState, XError, XResult};

#[cfg(not(feature = "server-side"))]
pub use client::*;
#[cfg(feature = "server-side")]
pub use server::*;

pub type SymbolMap<V> = HashMap<Symbol, V, IdentityState>;
pub type SymbolSet = HashSet<Symbol, IdentityState>;

#[macro_export]
macro_rules! s {
    ($string:expr) => {
        $crate::utils::Symbol::new($string).unwrap()
    };
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        $crate::utils::Symbol::new(&res).unwrap()
    }}
}
pub use s;

impl Symbol {
    pub fn len(&self) -> usize {
        return self.as_str().len();
    }

    pub fn is_empty(&self) -> bool {
        return self.as_str().is_empty();
    }

    pub fn to_owned(&self) -> String {
        return self.as_str().to_owned();
    }
}

impl Eq for Symbol {}

impl PartialEq<&str> for Symbol {
    fn eq(&self, other: &&str) -> bool {
        return self.as_str() == *other;
    }
}

impl PartialEq<String> for Symbol {
    fn eq(&self, other: &String) -> bool {
        return self.as_str() == other;
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Symbol) -> Option<Ordering> {
        return self.as_str().partial_cmp(other.as_str());
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Symbol) -> Ordering {
        return self.as_str().cmp(other.as_str());
    }
}

impl FromStr for Symbol {
    type Err = XError;

    fn from_str(s: &str) -> XResult<Symbol> {
        return Symbol::new(s);
    }
}

impl TryFrom<&str> for Symbol {
    type Error = XError;

    fn try_from(s: &str) -> XResult<Symbol> {
        return Symbol::new(s);
    }
}

impl TryFrom<String> for Symbol {
    type Error = XError;

    fn try_from(s: String) -> XResult<Symbol> {
        return Symbol::new(&s);
    }
}

impl TryFrom<&String> for Symbol {
    type Error = XError;

    fn try_from(s: &String) -> XResult<Symbol> {
        return Symbol::new(&s);
    }
}

impl From<Symbol> for String {
    fn from(s: Symbol) -> String {
        return s.to_owned();
    }
}

impl Deref for Symbol {
    type Target = str;

    fn deref(&self) -> &str {
        return self.as_str();
    }
}

const _: () = {
    use serde::de::{Deserialize, Deserializer, Error, Visitor};
    use serde::ser::{Serialize, Serializer};
    use std::fmt;

    impl<'de> Deserialize<'de> for Symbol {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Symbol, D::Error> {
            deserializer.deserialize_str(SymbolVisitor::new())
        }
    }

    impl Serialize for Symbol {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_str())
        }
    }

    pub struct SymbolVisitor {}

    impl SymbolVisitor {
        pub fn new() -> Self {
            return SymbolVisitor {};
        }
    }

    impl<'de> Visitor<'de> for SymbolVisitor {
        type Value = Symbol;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            return formatter.write_str("string");
        }

        fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
            return match Symbol::new(s) {
                Ok(symbol) => Ok(symbol),
                Err(XError::SymbolTooLong) => Err(E::custom("symbol is too long")),
                Err(_) => Err(E::custom("invalid symbol")),
            };
        }
    }
};

const _: () = {
    use rkyv::ser::Serializer;
    use rkyv::string::{ArchivedString, StringResolver};
    use rkyv::{Archive, Deserialize, Serialize};

    impl Archive for Symbol {
        type Archived = ArchivedString;
        type Resolver = StringResolver;

        unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
            ArchivedString::resolve_from_str(self.as_str(), pos, resolver, out);
        }
    }

    impl<S: Serializer + ?Sized> Serialize<S> for Symbol {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            return Ok(ArchivedString::serialize_from_str(self.as_str(), serializer)?);
        }
    }

    impl<D: rkyv::Fallible + ?Sized> Deserialize<Symbol, D> for ArchivedString {
        fn deserialize(&self, _: &mut D) -> Result<Symbol, D::Error> {
            // TODO: error handling
            let symbol = Symbol::new(self.as_str()).expect("invalid symbol");
            return Ok(symbol);
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_serde() {
        use serde_json;

        let s1 = Symbol::new("hello").unwrap();
        let json = serde_json::to_string(&s1).unwrap();
        assert_eq!(json, "\"hello\"");
        let s2: Symbol = serde_json::from_str(&json).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_symbol_rkyv() {
        use rkyv::Deserialize;

        let s1 = Symbol::new("hello").unwrap();
        let bytes = rkyv::to_bytes::<_, 256>(&s1).unwrap();
        let archived = rkyv::check_archived_root::<Symbol>(&bytes[..]).unwrap();
        assert_eq!(s1.as_str(), archived.as_str());

        let mut deserializer = rkyv::Infallible::default();
        let s2: Symbol = archived.deserialize(&mut deserializer).unwrap();
        assert_eq!(s1, s2);
    }
}
