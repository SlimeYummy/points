mod base;
mod cache;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use crate::utils::{IdentityState, XError, XResult};

#[cfg(not(feature = "server-side"))]
pub use cache::*;

pub type SymbolHashMap<V> = HashMap<Symbol, V, IdentityState>;
pub type SymbolHashSet = HashSet<Symbol, IdentityState>;

#[macro_export]
macro_rules! sb {
    ($string:expr) => {
        $crate::utils::Symbol::try_from($string).unwrap()
    };
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        $crate::utils::Symbol::new(&res).unwrap()
    }}
}
pub use sb;

impl Symbol {
    #[inline]
    pub fn len(&self) -> usize {
        return self.as_str().len();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.as_str().is_empty();
    }

    #[inline]
    pub fn to_owned(&self) -> String {
        return self.as_str().to_owned();
    }
}

impl Eq for Symbol {}

impl PartialEq<&str> for Symbol {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        return self.as_str() == *other;
    }
}

impl PartialEq<String> for Symbol {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        return self.as_str() == other;
    }
}

impl PartialOrd for Symbol {
    #[inline]
    fn partial_cmp(&self, other: &Symbol) -> Option<Ordering> {
        return self.as_str().partial_cmp(other.as_str());
    }
}

impl Ord for Symbol {
    #[inline]
    fn cmp(&self, other: &Symbol) -> Ordering {
        return self.as_str().cmp(other.as_str());
    }
}

impl AsRef<str> for Symbol {
    #[inline]
    fn as_ref(&self) -> &str {
        return self.as_str();
    }
}

impl FromStr for Symbol {
    type Err = XError;

    #[inline]
    fn from_str(s: &str) -> XResult<Symbol> {
        <Symbol>::new(s)
    }
}

impl TryFrom<&str> for Symbol {
    type Error = XError;

    #[inline]
    fn try_from(s: &str) -> XResult<Symbol> {
        <Symbol>::new(s)
    }
}

impl TryFrom<&String> for Symbol {
    type Error = XError;

    #[inline]
    fn try_from(s: &String) -> XResult<Symbol> {
        <Symbol>::new(s)
    }
}

impl TryFrom<String> for Symbol {
    type Error = XError;

    #[inline]
    fn try_from(s: String) -> XResult<Symbol> {
        <Symbol>::new(&s)
    }
}

impl From<Symbol> for String {
    #[inline]
    fn from(s: Symbol) -> String {
        s.to_owned()
    }
}

impl TryFrom<&rkyv::string::ArchivedString> for Symbol {
    type Error = XError;

    #[inline]
    fn try_from(s: &rkyv::string::ArchivedString) -> XResult<Symbol> {
        <Symbol>::new(s.as_str())
    }
}

impl Deref for Symbol {
    type Target = str;

    #[inline]
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
            SymbolVisitor {}
        }
    }

    impl<'de> Visitor<'de> for SymbolVisitor {
        type Value = Symbol;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
            match <Symbol>::new(s) {
                Ok(symbol) => Ok(symbol),
                Err(XError::SymbolTooLong(_)) => Err(E::custom("symbol is too long")),
                Err(_) => Err(E::custom("invalid symbol")),
            }
        }
    }
};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, rkyv::Portable)]
pub struct ArchivedSymbol {
    inner: rkyv::string::ArchivedString,
}

impl ArchivedSymbol {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl AsRef<str> for ArchivedSymbol {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for ArchivedSymbol {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for ArchivedSymbol {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for ArchivedSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

const _: () = {
    use rkyv::munge::munge;
    use rkyv::rancor::{fail, Fallible, Source};
    use rkyv::ser::Writer;
    use rkyv::string::{ArchivedString, StringResolver};
    use rkyv::{Archive, Deserialize, Place, Serialize};

    impl Archive for Symbol {
        type Archived = ArchivedSymbol;
        type Resolver = StringResolver;

        #[inline]
        fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
            munge!(let ArchivedSymbol { inner } = out);
            ArchivedString::resolve_from_str(self.as_str(), resolver, inner);
        }
    }

    impl<S> Serialize<S> for Symbol
    where
        S: Fallible + Writer + ?Sized,
        S::Error: Source,
    {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            return ArchivedString::serialize_from_str(self.as_str(), serializer);
        }
    }

    impl<D> Deserialize<Symbol, D> for ArchivedSymbol
    where
        D: Fallible + ?Sized,
        D::Error: Source,
    {
        fn deserialize(&self, _: &mut D) -> Result<Symbol, D::Error> {
            match Symbol::new(self.as_str()) {
                Ok(symbol) => Ok(symbol),
                Err(err) => fail!(err),
            }
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use rkyv::string::ArchivedString;
    use rkyv::Archived;

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
        use rkyv::rancor::Error;

        let s1 = Symbol::new("hello").unwrap();
        let bytes = rkyv::to_bytes::<Error>(&s1).unwrap();
        let archived = unsafe { rkyv::access_unchecked::<Archived<Symbol>>(&bytes) };
        assert_eq!(s1.as_str(), archived.as_str());

        let s2 = rkyv::deserialize::<_, Error>(archived).unwrap();
        assert_eq!(s1, s2);
    }
}
