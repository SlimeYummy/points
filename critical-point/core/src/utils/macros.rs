//
// Rust derive
//

#[macro_export]
macro_rules! ifelse {
    ($c:expr, $a:expr, $b:expr) => {
        if $c {
            $a
        } else {
            $b
        }
    };
}
pub use ifelse;

macro_rules! impl_for {
    ($t1:ty, $t2:ty, { $($body:tt)* }) => {
        impl $t1 {
            $($body)*
        }
        impl $t2 {
            $($body)*
        }
    };
}
pub(crate) use impl_for;

macro_rules! extend {
    ($cls:ident, $base:ty) => {
        static_assertions::const_assert!(std::mem::offset_of!($cls, _base) == 0);

        impl std::ops::Deref for $cls {
            type Target = $base;

            #[inline(always)]
            fn deref(&self) -> &$base {
                return &self._base;
            }
        }

        impl std::ops::DerefMut for $cls {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut $base {
                return &mut self._base;
            }
        }
    };
}
pub(crate) use extend;

macro_rules! interface {
    ($itf:ident, $base:ty) => {
        impl std::ops::Deref for dyn $itf {
            type Target = $base;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                let (ptr, _) = (self as *const dyn $itf).to_raw_parts();
                return unsafe { &*(ptr as *const () as *const Self::Target) };
            }
        }

        impl std::ops::DerefMut for dyn $itf {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                let (ptr, _) = (self as *mut dyn $itf).to_raw_parts();
                return unsafe { &mut *(ptr as *const () as *mut Self::Target) };
            }
        }
    };
}
pub(crate) use interface;

//
// serde & rkyv
//

macro_rules! rkyv_self {
    ($type:ty) => {
        const _: () = {
            use rkyv::rancor::Fallible;
            use rkyv::traits::NoUndef;
            use rkyv::{Archive, Deserialize, Place, Portable, Serialize};

            unsafe impl NoUndef for $type {}
            unsafe impl Portable for $type {}

            impl Archive for $type {
                type Archived = $type;
                type Resolver = ();

                #[inline]
                fn resolve(&self, _: Self::Resolver, out: Place<Self::Archived>) {
                    out.write(*self);
                }
            }

            impl<S: Fallible + ?Sized> Serialize<S> for $type {
                #[inline]
                fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
                    Ok(())
                }
            }

            impl<D: Fallible + ?Sized> Deserialize<$type, D> for $type {
                #[inline]
                fn deserialize(&self, _: &mut D) -> Result<$type, D::Error> {
                    Ok(*self)
                }
            }
        };
    };
}
pub(crate) use rkyv_self;

macro_rules! serde_by {
    ($type:ty, $tuple:ty, $from:expr, $to:expr) => {
        const _: () = {
            use serde::de::{Deserialize, Deserializer};
            use serde::ser::{Serialize, Serializer};

            impl<'de> Deserialize<'de> for $type {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    <$tuple>::deserialize(deserializer).map(|by| $from(by))
                }
            }

            impl Serialize for $type {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let by: $tuple = $to(self);
                    by.serialize(serializer)
                }
            }
        };
    };
}
pub(crate) use serde_by;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Castable;
    use std::any::Any;

    struct Base {
        b1: i32,
        b2: u8,
        b3: String,
    }

    unsafe trait Itf: Any {
        fn sum(&self) -> u64;
    }

    interface!(Itf, Base);

    struct Derived {
        _base: Base,
        d1: i32,
        d2: u8,
        d3: String,
    }

    extend!(Derived, Base);

    unsafe impl Itf for Derived {
        fn sum(&self) -> u64 {
            self.b1 as u64 + self.b2 as u64
        }
    }

    #[test]
    fn test_extend_interface() {
        let d = Derived {
            _base: Base {
                b1: 1,
                b2: 2,
                b3: "aaa".into(),
            },
            d1: 40,
            d2: 50,
            d3: "bbb".into(),
        };

        assert_eq!(d.d1, 40);
        assert_eq!(d.d2, 50);
        assert_eq!(d.d3, "bbb");

        let itf: &dyn Itf = &d;
        assert_eq!(itf.sum(), 3);
        assert_eq!(itf.b1, 1);
        assert_eq!(itf.b2, 2);
        assert_eq!(itf.b3, "aaa");

        let dd = itf.cast::<Derived>().unwrap();
        assert_eq!(dd.b1, 1);
        assert_eq!(dd.b2, 2);
        assert_eq!(dd.b3, "aaa");
        assert_eq!(dd.d1, 40);
        assert_eq!(dd.d2, 50);
        assert_eq!(dd.d3, "bbb");
    }
}
