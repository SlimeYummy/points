use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use ustr_fxhash::Ustr;

use crate::utils::{XError, XResult};

#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(Ustr);

impl Symbol {
    #[inline]
    pub fn new(string: &str) -> Symbol {
        return Symbol(Ustr::from(string));
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        return self.0.as_str();
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.0.len();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.len() == 0;
    }

    #[inline]
    pub fn to_owned(&self) -> String {
        return self.as_str().to_owned();
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
        Ok(Symbol::new(s))
    }
}

impl From<&str> for Symbol {
    #[inline]
    fn from(s: &str) -> Symbol {
        Symbol::new(s)
    }
}

impl From<&String> for Symbol {
    #[inline]
    fn from(s: &String) -> Symbol {
        Symbol::new(s)
    }
}

impl From<String> for Symbol {
    #[inline]
    fn from(s: String) -> Symbol {
        Symbol::new(&s)
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
            Ok(Symbol::new(s))
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

impl From<&rkyv::string::ArchivedString> for Symbol {
    #[inline]
    fn from(s: &rkyv::string::ArchivedString) -> Symbol {
        Symbol::new(s.as_str())
    }
}

impl From<&ArchivedSymbol> for Symbol {
    #[inline]
    fn from(s: &ArchivedSymbol) -> Symbol {
        Symbol::new(s.inner.as_str())
    }
}

const _: () = {
    use rkyv::munge::munge;
    use rkyv::rancor::{Fallible, Source};
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
            Ok(Symbol::new(self.as_str()))
        }
    }
};

#[macro_export]
macro_rules! sb {
    ($string:expr) => {
        $crate::utils::Symbol::new($string)
    };
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        $crate::utils::Symbol::new(&res)
    }}
}
pub use sb;
