use std::any::{Any, TypeId};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;

use crate::utils::error::xres;
use crate::utils::XResult;

#[inline]
pub fn const_ptr<T, U>(val: &T) -> *const U {
    (val as *const T) as *const U
}

#[inline]
pub fn mut_ptr<T, U>(val: &mut T) -> *mut U {
    (val as *mut T) as *mut U
}

#[inline]
#[allow(clippy::all)]
pub unsafe fn force_mut<T: ?Sized>(val: &T) -> &mut T {
    let ptr = mem::transmute::<&T, *const T>(val);
    return mem::transmute::<*const T, &mut T>(ptr);
}

pub trait Castable {
    type Ptr<T: 'static>;

    fn cast<T: 'static>(self) -> XResult<Self::Ptr<T>>;
    unsafe fn cast_unchecked<T: 'static>(self) -> Self::Ptr<T>;
}

impl<'t, TO: ?Sized + Any> Castable for &'t TO {
    type Ptr<T: 'static> = &'t T;

    #[inline]
    fn cast<T: 'static>(self) -> XResult<&'t T> {
        if (*self).type_id() == TypeId::of::<T>() {
            Ok(unsafe { self.cast_unchecked() })
        }
        else {
            xres!(BadType; "invalid cast")
        }
    }

    #[inline]
    unsafe fn cast_unchecked<T: 'static>(self) -> &'t T {
        let (src_data, _) = (self as *const TO).to_raw_parts();
        &*(src_data as *const T)
    }
}

impl<'t, TO: ?Sized + Any> Castable for &'t mut TO {
    type Ptr<T: 'static> = &'t mut T;

    #[inline]
    fn cast<T: 'static>(self) -> XResult<&'t mut T> {
        if (*self).type_id() == TypeId::of::<T>() {
            Ok(unsafe { self.cast_unchecked() })
        }
        else {
            xres!(BadType; "invalid cast")
        }
    }

    #[inline]
    unsafe fn cast_unchecked<T: 'static>(self) -> &'t mut T {
        let (src_data, _) = (self as *mut TO).to_raw_parts();
        &mut *(src_data as *mut T)
    }
}

impl<TO: ?Sized + Any> Castable for Box<TO> {
    type Ptr<T: 'static> = Box<T>;

    #[inline]
    fn cast<T: 'static>(self) -> XResult<Box<T>> {
        if (*self).type_id() == TypeId::of::<T>() {
            Ok(unsafe { self.cast_unchecked() })
        }
        else {
            xres!(BadType; "invalid cast")
        }
    }

    #[inline]
    unsafe fn cast_unchecked<T: 'static>(self) -> Box<T> {
        let (src_data, _) = Box::into_raw(self).to_raw_parts();
        unsafe { Box::from_raw(src_data as *mut T) }
    }
}

impl<TO: ?Sized + Any> Castable for Rc<TO> {
    type Ptr<T: 'static> = Rc<T>;

    #[inline]
    fn cast<T: 'static>(self) -> XResult<Rc<T>> {
        if (*self).type_id() == TypeId::of::<T>() {
            Ok(unsafe { self.cast_unchecked() })
        }
        else {
            xres!(BadType; "invalid cast")
        }
    }

    #[inline]
    unsafe fn cast_unchecked<T: 'static>(self) -> Rc<T> {
        let (src_data, _) = Rc::into_raw(self).to_raw_parts();
        unsafe { Rc::from_raw(src_data as *const T) }
    }
}

impl<TO: ?Sized + Any> Castable for Arc<TO> {
    type Ptr<T: 'static> = Arc<T>;

    #[inline]
    fn cast<T: 'static>(self) -> XResult<Arc<T>> {
        if (*self).type_id() == TypeId::of::<T>() {
            Ok(unsafe { self.cast_unchecked() })
        }
        else {
            xres!(BadType; "invalid cast")
        }
    }

    #[inline]
    unsafe fn cast_unchecked<T: 'static>(self) -> Arc<T> {
        let (src_data, _) = Arc::into_raw(self).to_raw_parts();
        unsafe { Arc::from_raw(src_data as *const T) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Trait: Any {
        fn foo(&self) -> String;
    }

    #[derive(Clone)]
    struct StructA {
        s1: String,
        s2: String,
    }

    impl Trait for StructA {
        fn foo(&self) -> String {
            format!("{} {}", self.s1, self.s2)
        }
    }

    #[derive(Clone)]
    struct StructB {
        i: i32,
        f: f32,
        s: String,
    }

    impl Trait for StructB {
        fn foo(&self) -> String {
            format!("{} {} {}", self.s, self.i, self.f)
        }
    }

    #[test]
    fn test_cast_ref() {
        let mut a = StructA {
            s1: "AA".into(),
            s2: "BB".into(),
        };
        let mut b = StructB {
            i: -99,
            f: 2.5,
            s: "CC".into(),
        };

        {
            let dyn_a = &a as &dyn Trait;
            assert!(dyn_a.cast::<StructB>().is_err());
            assert!(dyn_a.cast::<()>().is_err());
            let aa = dyn_a.cast::<StructA>().unwrap();
            assert_eq!(aa.s1, "AA");
            assert_eq!(aa.s2, "BB");
            assert_eq!(aa.foo(), "AA BB");

            let dyn_b = &b as &dyn Trait;
            assert!(dyn_b.cast::<StructA>().is_err());
            assert!(dyn_b.cast::<()>().is_err());
            let bb = dyn_b.cast::<StructB>().unwrap();
            assert_eq!(bb.i, -99);
            assert_eq!(bb.f, 2.5);
            assert_eq!(bb.s, "CC");
            assert_eq!(bb.foo(), "CC -99 2.5");
        }

        {
            let dyn_a = &mut a as &mut dyn Trait;
            assert!(dyn_a.cast::<StructB>().is_err());
            assert!(dyn_a.cast::<()>().is_err());
            let aa = dyn_a.cast::<StructA>().unwrap();
            aa.s1 = "XXX".into();
            aa.s2 = "YYY".into();
            assert_eq!(aa.s1, "XXX");
            assert_eq!(aa.s2, "YYY");
            assert_eq!(aa.foo(), "XXX YYY");

            let dyn_b = &mut b as &mut dyn Trait;
            assert!(dyn_b.cast::<StructA>().is_err());
            assert!(dyn_b.cast::<()>().is_err());
            let bb = dyn_b.cast::<StructB>().unwrap();
            bb.i = 123;
            bb.f = 3.5;
            bb.s = "Z!!!".into();
            assert_eq!(bb.i, 123);
            assert_eq!(bb.f, 3.5);
            assert_eq!(bb.s, "Z!!!");
            assert_eq!(bb.foo(), "Z!!! 123 3.5");
        }

        assert_eq!(a.s1, "XXX");
        assert_eq!(a.s2, "YYY");
        assert_eq!(a.foo(), "XXX YYY");

        assert_eq!(b.i, 123);
        assert_eq!(b.f, 3.5);
        assert_eq!(b.s, "Z!!!");
        assert_eq!(b.foo(), "Z!!! 123 3.5");
    }

    #[test]
    fn test_box_cast_ptr() {
        let a = Box::new(StructA {
            s1: "AA".into(),
            s2: "BB".into(),
        });
        assert!((a.clone() as Box<dyn Trait>).cast::<StructB>().is_err());
        assert!((a.clone() as Box<dyn Trait>).cast::<String>().is_err());
        let aa = (a as Box<dyn Trait>).cast::<StructA>().unwrap();
        assert_eq!(aa.s1, "AA");
        assert_eq!(aa.s2, "BB");
        assert_eq!(aa.foo(), "AA BB");

        let b = Box::new(StructB {
            i: -99,
            f: 2.5,
            s: "CC".into(),
        });
        assert!((b.clone() as Box<dyn Trait>).cast::<StructA>().is_err());
        assert!((b.clone() as Box<dyn Trait>).cast::<String>().is_err());
        let bb = (b as Box<dyn Trait>).cast::<StructB>().unwrap();
        assert_eq!(bb.i, -99);
        assert_eq!(bb.f, 2.5);
        assert_eq!(bb.s, "CC");
        assert_eq!(bb.foo(), "CC -99 2.5");
    }

    #[test]
    fn test_rc_cast_ptr() {
        let dyn_a = Rc::new(StructA {
            s1: "AA".into(),
            s2: "BB".into(),
        }) as Rc<dyn Trait>;
        assert!(dyn_a.clone().cast::<StructB>().is_err());
        assert!(dyn_a.clone().cast::<String>().is_err());

        let aa = dyn_a.clone().cast::<StructA>().unwrap();
        assert_eq!(aa.s1, "AA");
        assert_eq!(aa.s2, "BB");
        assert_eq!(aa.foo(), "AA BB");
        assert_eq!(Rc::strong_count(&dyn_a), 2);

        let aaa = dyn_a.cast::<StructA>().unwrap();
        assert_eq!(Rc::strong_count(&aaa), 2);
        assert_eq!(aaa.foo(), "AA BB");
        assert!(Rc::ptr_eq(&aa, &aaa));

        let dyn_b = Rc::new(StructB {
            i: -99,
            f: 2.5,
            s: "CC".into(),
        }) as Rc<dyn Trait>;
        assert!(dyn_b.clone().cast::<StructA>().is_err());
        assert!(dyn_b.clone().cast::<String>().is_err());

        let bb = dyn_b.clone().cast::<StructB>().unwrap();
        assert_eq!(bb.i, -99);
        assert_eq!(bb.f, 2.5);
        assert_eq!(bb.s, "CC");
        assert_eq!(bb.foo(), "CC -99 2.5");
        assert_eq!(Rc::strong_count(&dyn_b), 2);

        let bbb = dyn_b.cast::<StructB>().unwrap();
        assert_eq!(Rc::strong_count(&bbb), 2);
        assert_eq!(bbb.foo(), "CC -99 2.5");
        assert!(Rc::ptr_eq(&bb, &bbb));
    }

    #[test]
    fn test_arc_cast_ptr() {
        let dyn_a = Arc::new(StructA {
            s1: "PPP".into(),
            s2: "QQ".into(),
        }) as Arc<dyn Trait>;
        assert!(dyn_a.clone().cast::<StructB>().is_err());
        assert!(dyn_a.clone().cast::<String>().is_err());

        let aa = dyn_a.clone().cast::<StructA>().unwrap();
        assert_eq!(aa.s1, "PPP");
        assert_eq!(aa.s2, "QQ");
        assert_eq!(aa.foo(), "PPP QQ");
        assert_eq!(Arc::strong_count(&dyn_a), 2);

        let aaa = dyn_a.cast::<StructA>().unwrap();
        assert_eq!(Arc::strong_count(&aaa), 2);
        assert_eq!(aaa.foo(), "PPP QQ");
        assert!(Arc::ptr_eq(&aa, &aaa));

        let dyn_b = Arc::new(StructB {
            i: -99,
            f: 2.5,
            s: "$$".into(),
        }) as Arc<dyn Trait>;
        assert!(dyn_b.clone().cast::<StructA>().is_err());
        assert!(dyn_b.clone().cast::<String>().is_err());

        let bb = dyn_b.clone().cast::<StructB>().unwrap();
        assert_eq!(bb.i, -99);
        assert_eq!(bb.f, 2.5);
        assert_eq!(bb.s, "$$");
        assert_eq!(bb.foo(), "$$ -99 2.5");
        assert_eq!(Arc::strong_count(&dyn_b), 2);

        let bbb = dyn_b.cast::<StructB>().unwrap();
        assert_eq!(Arc::strong_count(&bbb), 2);
        assert_eq!(bbb.foo(), "$$ -99 2.5");
        assert!(Arc::ptr_eq(&bb, &bbb));
    }
}
