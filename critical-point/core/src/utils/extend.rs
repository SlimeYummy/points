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

        impl crate::utils::Castable for dyn $itf {}
    };
}
pub(crate) use interface;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::CastRef;

    struct Base {
        b1: i32,
        b2: u8,
        b3: String,
    }

    unsafe trait Itf {
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
                b3: "aaa".to_string(),
            },
            d1: 40,
            d2: 50,
            d3: "bbb".to_string(),
        };

        assert_eq!(d.d1, 40);
        assert_eq!(d.d2, 50);
        assert_eq!(d.d3, "bbb");

        let itf: &dyn Itf = &d;
        assert_eq!(itf.sum(), 3);
        assert_eq!(itf.b1, 1);
        assert_eq!(itf.b2, 2);
        assert_eq!(itf.b3, "aaa");

        let dd = itf.cast_ref::<Derived>().unwrap();
        assert_eq!(dd.b1, 1);
        assert_eq!(dd.b2, 2);
        assert_eq!(dd.b3, "aaa");
        assert_eq!(dd.d1, 40);
        assert_eq!(dd.d2, 50);
        assert_eq!(dd.d3, "bbb");
    }
}
