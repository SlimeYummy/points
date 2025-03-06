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

#[macro_export]
macro_rules! asb {
    ($string:expr) => {
        $crate::utils::ASymbol::new($string).unwrap()
    };
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        $crate::utils::ASymbol::new(&res).unwrap()
    }}
}
pub use asb;

macro_rules! symbol_methods {
    ($symbol:ty) => {
        impl $symbol {
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

        impl Eq for $symbol {}

        impl PartialEq<&str> for $symbol {
            #[inline]
            fn eq(&self, other: &&str) -> bool {
                return self.as_str() == *other;
            }
        }

        impl PartialEq<String> for $symbol {
            #[inline]
            fn eq(&self, other: &String) -> bool {
                return self.as_str() == other;
            }
        }

        impl PartialOrd for $symbol {
            #[inline]
            fn partial_cmp(&self, other: &$symbol) -> Option<Ordering> {
                return self.as_str().partial_cmp(other.as_str());
            }
        }

        impl Ord for $symbol {
            #[inline]
            fn cmp(&self, other: &$symbol) -> Ordering {
                return self.as_str().cmp(other.as_str());
            }
        }

        impl AsRef<str> for $symbol {
            #[inline]
            fn as_ref(&self) -> &str {
                return self.as_str();
            }
        }

        impl FromStr for $symbol {
            type Err = XError;

            #[inline]
            fn from_str(s: &str) -> XResult<$symbol> {
                <$symbol>::new(s)
            }
        }

        impl TryFrom<&str> for $symbol {
            type Error = XError;

            #[inline]
            fn try_from(s: &str) -> XResult<$symbol> {
                <$symbol>::new(s)
            }
        }

        impl TryFrom<String> for $symbol {
            type Error = XError;

            #[inline]
            fn try_from(s: String) -> XResult<$symbol> {
                <$symbol>::new(&s)
            }
        }

        impl TryFrom<&String> for $symbol {
            type Error = XError;

            #[inline]
            fn try_from(s: &String) -> XResult<$symbol> {
                <$symbol>::new(s)
            }
        }

        impl From<$symbol> for String {
            #[inline]
            fn from(s: $symbol) -> String {
                s.to_owned()
            }
        }

        impl Deref for $symbol {
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

            impl<'de> Deserialize<'de> for $symbol {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<$symbol, D::Error> {
                    deserializer.deserialize_str(SymbolVisitor::new())
                }
            }

            impl Serialize for $symbol {
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
                type Value = $symbol;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("string")
                }

                fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
                    match <$symbol>::new(s) {
                        Ok(symbol) => Ok(symbol),
                        Err(XError::SymbolTooLong(_)) => Err(E::custom("symbol is too long")),
                        Err(_) => Err(E::custom("invalid symbol")),
                    }
                }
            }
        };

        const _: () = {
            use rkyv::ser::Serializer;
            use rkyv::string::{ArchivedString, StringResolver};
            use rkyv::{Archive, Deserialize, Serialize};

            impl Archive for $symbol {
                type Archived = ArchivedString;
                type Resolver = StringResolver;

                unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
                    ArchivedString::resolve_from_str(self.as_str(), pos, resolver, out);
                }
            }

            impl<S: Serializer + ?Sized> Serialize<S> for $symbol {
                fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                    return ArchivedString::serialize_from_str(self.as_str(), serializer);
                }
            }

            impl<D: rkyv::Fallible + ?Sized> Deserialize<$symbol, D> for ArchivedString {
                fn deserialize(&self, _: &mut D) -> Result<$symbol, D::Error> {
                    // TODO: error handling
                    let symbol = <$symbol>::new(self.as_str()).expect("invalid symbol");
                    Ok(symbol)
                }
            }
        };
    };
}

symbol_methods!(Symbol);
#[cfg(not(feature = "server-side"))]
symbol_methods!(ASymbol);

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

        let mut deserializer = rkyv::Infallible;
        let s2: Symbol = archived.deserialize(&mut deserializer).unwrap();
        assert_eq!(s1, s2);
    }
}
