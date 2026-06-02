mod internal;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::ops::Deref;
use std::str::FromStr;
use std::{fmt, str};

use crate::utils::error::{XError, XResult};
pub use crate::utils::symbol::internal::Symbol;

impl Symbol {
    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.len() == 0;
    }

    #[inline]
    pub fn to_owned(&self) -> String {
        return self.as_str().to_owned();
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.precomputed_hash().hash(state);
    }
}

impl PartialEq<&str> for Symbol {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        return self.as_str() == *other;
    }
}

impl PartialEq<Symbol> for &str {
    #[inline]
    fn eq(&self, other: &Symbol) -> bool {
        other.as_str() == *self
    }
}

impl PartialEq<String> for Symbol {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        return self.as_str() == other;
    }
}

impl PartialEq<Symbol> for String {
    #[inline]
    fn eq(&self, other: &Symbol) -> bool {
        return other.as_str() == self;
    }
}

impl PartialOrd<Symbol> for Symbol {
    fn partial_cmp(&self, other: &Symbol) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
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
        Symbol::new(s)
    }
}

impl From<Symbol> for String {
    #[inline]
    fn from(s: Symbol) -> String {
        s.to_owned()
    }
}

impl Deref for Symbol {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        return self.as_str();
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s{:?}", self.as_str())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
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
            match Symbol::new(s) {
                Ok(symbol) => Ok(symbol),
                Err(e) => Err(E::custom(e.to_string())),
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

impl TryFrom<&rkyv::string::ArchivedString> for Symbol {
    type Error = XError;

    #[inline]
    fn try_from(s: &rkyv::string::ArchivedString) -> Result<Symbol, XError> {
        Symbol::new(s.as_str())
    }
}

impl TryFrom<&ArchivedSymbol> for Symbol {
    type Error = XError;

    #[inline]
    fn try_from(s: &ArchivedSymbol) -> Result<Symbol, XError> {
        Symbol::new(s.inner.as_str())
    }
}

const _: () = {
    use rkyv::munge::munge;
    use rkyv::rancor::{Fallible, ResultExt, Source};
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
            Symbol::new(self.as_str()).into_error()
        }
    }
};

#[macro_export]
macro_rules! sb {
    ($string:expr) => {
        $crate::utils::Symbol::new($string).unwrap()
    };
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        $crate::utils::Symbol::new(&res).unwrap()
    }}
}
pub use sb;

/// A hasher that directly uses the precomputed hash value from `Symbol`.
#[derive(Default)]
pub struct SymbolHasher {
    hash: u64,
}

impl Hasher for SymbolHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        // Note: This is not the intended use case for SymbolHasher.
        // It's provided for completeness but only uses the first 8 bytes,
        // which may result in poor hash distribution for general use.
        let mut buf = [0u8; 8];
        let len = bytes.len().min(8);
        buf[..len].copy_from_slice(&bytes[..len]);
        self.hash = u64::from_ne_bytes(buf);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        // Primary use case: directly use the precomputed hash from Symbol.
        // This is called by Symbol::hash() via Hash trait implementation.
        self.hash = i;
    }
}

pub type SymbolMap<V> = HashMap<Symbol, V, BuildHasherDefault<SymbolHasher>>;
pub type SymbolSet = HashSet<Symbol, BuildHasherDefault<SymbolHasher>>;

#[cfg(test)]
mod tests {
    use super::*;
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
