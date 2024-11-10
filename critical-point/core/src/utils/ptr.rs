use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;
use std::{mem, ptr};

use crate::utils::{XError, XResult};

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

pub trait Castable {}

pub trait CastRef {
    fn cast_ref<T: 'static>(&self) -> XResult<&T>;
    unsafe fn cast_ref_unchecked<T: 'static>(&self) -> &T;
    fn cast_mut<T: 'static>(&mut self) -> XResult<&mut T>;
    unsafe fn cast_mut_unchecked<T: 'static>(&mut self) -> &mut T;
}

impl<TO> CastRef for TO
where
    TO: ?Sized + Castable,
{
    #[inline]
    fn cast_ref<T: 'static>(&self) -> XResult<&T> {
        check_variant::<TO, T>(self)?;
        return Ok(unsafe { self.cast_ref_unchecked() });
    }

    #[inline]
    unsafe fn cast_ref_unchecked<T: 'static>(&self) -> &T {
        let (src_data, _) = (self as *const TO).to_raw_parts();
        &*(src_data as *const T)
    }

    #[inline]
    fn cast_mut<T: 'static>(&mut self) -> XResult<&mut T> {
        check_variant::<TO, T>(self)?;
        return Ok(unsafe { self.cast_mut_unchecked() });
    }

    #[inline]
    unsafe fn cast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        let (src_data, _) = (self as *mut TO).to_raw_parts();
        &mut *(src_data as *mut T)
    }
}

pub trait CastPtr {
    type Ptr<T>;

    fn cast_as<T: 'static>(self) -> XResult<Self::Ptr<T>>;
    unsafe fn cast_as_unchecked<T: 'static>(self) -> Self::Ptr<T>;

    fn cast_to<T>(&self) -> XResult<Self::Ptr<T>>
    where
        T: 'static,
        Self::Ptr<T>: Clone;
    unsafe fn cast_to_unchecked<T>(&self) -> Self::Ptr<T>
    where
        T: 'static,
        Self::Ptr<T>: Clone;
}

impl<TO> CastPtr for Box<TO>
where
    TO: ?Sized + Castable + CastRef,
{
    type Ptr<T> = Box<T>;

    #[inline]
    fn cast_as<T: 'static>(self) -> XResult<Box<T>> {
        check_variant::<TO, T>(self.as_ref())?;
        Ok(unsafe { self.cast_as_unchecked() })
    }

    #[inline]
    unsafe fn cast_as_unchecked<T: 'static>(self) -> Box<T> {
        let (src_data, _) = (Box::leak(self) as *mut TO).to_raw_parts();

        Box::from_raw(src_data as *mut T)
    }

    #[inline]
    fn cast_to<T>(&self) -> XResult<Box<T>>
    where
        T: 'static,
        Box<T>: Clone,
    {
        check_variant::<TO, T>(self.as_ref())?;
        Ok(unsafe { self.cast_to_unchecked() })
    }

    #[inline]
    unsafe fn cast_to_unchecked<T>(&self) -> Box<T>
    where
        T: 'static,
        Box<T>: Clone,
    {
        let (src_data, _) = (self.as_ref() as *const TO).to_raw_parts();
        let dst_box = Box::from_raw(src_data as *mut T);
        let new_dst_box = dst_box.clone();
        mem::forget(dst_box);
        new_dst_box
    }
}

impl<TO> CastPtr for Rc<TO>
where
    TO: ?Sized + Castable + CastRef,
{
    type Ptr<T> = Rc<T>;

    #[inline]
    fn cast_as<T: 'static>(self) -> XResult<Rc<T>> {
        check_variant::<TO, T>(self.as_ref())?;
        Ok(unsafe { self.cast_as_unchecked() })
    }

    #[inline]
    unsafe fn cast_as_unchecked<T: 'static>(self) -> Rc<T> {
        let (src_data, _) = Rc::into_raw(self).to_raw_parts();

        unsafe { Rc::from_raw(src_data as *const T) }
    }

    #[inline]
    fn cast_to<T: 'static>(&self) -> XResult<Rc<T>> {
        self.clone().cast_as()
    }

    #[inline]
    unsafe fn cast_to_unchecked<T: 'static>(&self) -> Rc<T> {
        self.clone().cast_as_unchecked()
    }
}

impl<TO> CastPtr for Arc<TO>
where
    TO: ?Sized + Castable + CastRef,
{
    type Ptr<T> = Arc<T>;

    #[inline]
    fn cast_as<T: 'static>(self) -> XResult<Arc<T>> {
        check_variant::<TO, T>(self.as_ref())?;
        Ok(unsafe { self.cast_as_unchecked() })
    }

    #[inline]
    unsafe fn cast_as_unchecked<T: 'static>(self) -> Arc<T> {
        let (src_data, _) = Arc::into_raw(self).to_raw_parts();
        unsafe { Arc::from_raw(src_data as *const T) }
    }

    #[inline]
    fn cast_to<T: 'static>(&self) -> XResult<Arc<T>> {
        self.clone().cast_as()
    }

    #[inline]
    unsafe fn cast_to_unchecked<T: 'static>(&self) -> Arc<T> {
        self.clone().cast_as_unchecked()
    }
}

#[inline]
fn check_variant<TO, T>(re: &TO) -> XResult<()>
where
    TO: ?Sized + Castable,
    T: 'static,
{
    let src_meta = ptr::metadata(re as *const TO);
    let src_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&src_meta) };

    let dst_ref: &dyn Any = unsafe { mem::transmute_copy::<usize, &T>(&0) };
    let dst_meta = ptr::metadata(dst_ref);
    let dst_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&dst_meta) };

    if src_drop != dst_drop {
        return Err(XError::BadType);
    }
    Ok(())
}

#[cfg(not(feature = "server-side"))]
mod x {
    pub type Xrc<T> = std::rc::Rc<T>;
    pub type Xweak<T> = std::rc::Weak<T>;
}

#[cfg(feature = "server-side")]
mod x {
    pub type Xrc<T> = std::sync::Arc<T>;
    pub type Xweak<T> = std::sync::Weak<T>;
}

pub use x::*;

#[cfg(test)]
mod tests {
    use super::*;

    trait Trait {
        fn foo(&self) -> String;
    }

    impl Castable for dyn Trait {}

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
            s1: "AA".to_string(),
            s2: "BB".to_string(),
        };
        let mut b = StructB {
            i: -99,
            f: 2.5,
            s: "CC".to_string(),
        };

        {
            let dyn_a = &a as &dyn Trait;
            assert!(dyn_a.cast_ref::<StructB>().is_err());
            assert!(dyn_a.cast_ref::<()>().is_err());
            let aa = dyn_a.cast_ref::<StructA>().unwrap();
            assert_eq!(aa.s1, "AA");
            assert_eq!(aa.s2, "BB");
            assert_eq!(aa.foo(), "AA BB");

            let dyn_b = &b as &dyn Trait;
            assert!(dyn_b.cast_ref::<StructA>().is_err());
            assert!(dyn_b.cast_ref::<()>().is_err());
            let bb = dyn_b.cast_ref::<StructB>().unwrap();
            assert_eq!(bb.i, -99);
            assert_eq!(bb.f, 2.5);
            assert_eq!(bb.s, "CC");
            assert_eq!(bb.foo(), "CC -99 2.5");
        }

        {
            let dyn_a = &mut a as &mut dyn Trait;
            assert!(dyn_a.cast_mut::<StructB>().is_err());
            assert!(dyn_a.cast_mut::<()>().is_err());
            let aa = dyn_a.cast_mut::<StructA>().unwrap();
            aa.s1 = "XXX".to_string();
            aa.s2 = "YYY".to_string();
            assert_eq!(aa.s1, "XXX");
            assert_eq!(aa.s2, "YYY");
            assert_eq!(aa.foo(), "XXX YYY");

            let dyn_b = &mut b as &mut dyn Trait;
            assert!(dyn_b.cast_mut::<StructA>().is_err());
            assert!(dyn_b.cast_mut::<()>().is_err());
            let bb = dyn_b.cast_mut::<StructB>().unwrap();
            bb.i = 123;
            bb.f = 3.5;
            bb.s = "Z!!!".to_string();
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
        {
            let a = Box::new(StructA {
                s1: "AA".to_string(),
                s2: "BB".to_string(),
            });
            assert!((a.clone() as Box<dyn Trait>).cast_as::<StructB>().is_err());
            assert!((a.clone() as Box<dyn Trait>).cast_as::<String>().is_err());
            let aa = (a as Box<dyn Trait>).cast_as::<StructA>().unwrap();
            assert_eq!(aa.s1, "AA");
            assert_eq!(aa.s2, "BB");
            assert_eq!(aa.foo(), "AA BB");

            let b = Box::new(StructB {
                i: -99,
                f: 2.5,
                s: "CC".to_string(),
            });
            assert!((b.clone() as Box<dyn Trait>).cast_as::<StructA>().is_err());
            assert!((b.clone() as Box<dyn Trait>).cast_as::<String>().is_err());
            let bb = (b as Box<dyn Trait>).cast_as::<StructB>().unwrap();
            assert_eq!(bb.i, -99);
            assert_eq!(bb.f, 2.5);
            assert_eq!(bb.s, "CC");
            assert_eq!(bb.foo(), "CC -99 2.5");
        }

        {
            let a = Box::new(StructA {
                s1: "AA".to_string(),
                s2: "BB".to_string(),
            });
            let dyn_a = a as Box<dyn Trait>;
            assert!(dyn_a.cast_to::<StructB>().is_err());
            assert!(dyn_a.cast_to::<String>().is_err());
            let mut aa = dyn_a.cast_to::<StructA>().unwrap();
            assert_eq!(aa.foo(), "AA BB");
            aa.s2 = "YYY".to_string();
            assert_eq!(aa.foo(), "AA YYY");
            assert_eq!(dyn_a.foo(), "AA BB");

            let b = Box::new(StructB {
                i: -99,
                f: 2.5,
                s: "CC".to_string(),
            });
            let dyn_b = b as Box<dyn Trait>;
            assert!(dyn_b.cast_to::<StructA>().is_err());
            assert!(dyn_b.cast_to::<String>().is_err());
            let mut bb = dyn_b.cast_to::<StructB>().unwrap();
            assert_eq!(bb.foo(), "CC -99 2.5");
            bb.f = 3.5;
            assert_eq!(bb.foo(), "CC -99 3.5");
            assert_eq!(dyn_b.foo(), "CC -99 2.5");
        }
    }

    #[test]
    fn test_rc_cast_ptr() {
        let dyn_a = Rc::new(StructA {
            s1: "AA".to_string(),
            s2: "BB".to_string(),
        }) as Rc<dyn Trait>;
        assert!(dyn_a.cast_to::<StructB>().is_err());
        assert!(dyn_a.cast_to::<String>().is_err());

        let aa = dyn_a.cast_to::<StructA>().unwrap();
        assert_eq!(aa.s1, "AA");
        assert_eq!(aa.s2, "BB");
        assert_eq!(aa.foo(), "AA BB");
        assert_eq!(Rc::strong_count(&dyn_a), 2);

        let aaa = dyn_a.cast_as::<StructA>().unwrap();
        assert_eq!(Rc::strong_count(&aaa), 2);
        assert_eq!(aaa.foo(), "AA BB");
        assert!(Rc::ptr_eq(&aa, &aaa));

        let dyn_b = Rc::new(StructB {
            i: -99,
            f: 2.5,
            s: "CC".to_string(),
        }) as Rc<dyn Trait>;
        assert!(dyn_b.cast_to::<StructA>().is_err());
        assert!(dyn_b.cast_to::<String>().is_err());

        let bb = dyn_b.cast_to::<StructB>().unwrap();
        assert_eq!(bb.i, -99);
        assert_eq!(bb.f, 2.5);
        assert_eq!(bb.s, "CC");
        assert_eq!(bb.foo(), "CC -99 2.5");
        assert_eq!(Rc::strong_count(&dyn_b), 2);

        let bbb = dyn_b.cast_as::<StructB>().unwrap();
        assert_eq!(Rc::strong_count(&bbb), 2);
        assert_eq!(bbb.foo(), "CC -99 2.5");
        assert!(Rc::ptr_eq(&bb, &bbb));
    }

    #[test]
    fn test_arc_cast_ptr() {
        let dyn_a = Arc::new(StructA {
            s1: "PPP".to_string(),
            s2: "QQ".to_string(),
        }) as Arc<dyn Trait>;
        assert!(dyn_a.cast_to::<StructB>().is_err());
        assert!(dyn_a.cast_to::<String>().is_err());

        let aa = dyn_a.cast_to::<StructA>().unwrap();
        assert_eq!(aa.s1, "PPP");
        assert_eq!(aa.s2, "QQ");
        assert_eq!(aa.foo(), "PPP QQ");
        assert_eq!(Arc::strong_count(&dyn_a), 2);

        let aaa = dyn_a.cast_as::<StructA>().unwrap();
        assert_eq!(Arc::strong_count(&aaa), 2);
        assert_eq!(aaa.foo(), "PPP QQ");
        assert!(Arc::ptr_eq(&aa, &aaa));

        let dyn_b = Arc::new(StructB {
            i: -99,
            f: 2.5,
            s: "$$".to_string(),
        }) as Arc<dyn Trait>;
        assert!(dyn_b.cast_to::<StructA>().is_err());
        assert!(dyn_b.cast_to::<String>().is_err());

        let bb = dyn_b.cast_to::<StructB>().unwrap();
        assert_eq!(bb.i, -99);
        assert_eq!(bb.f, 2.5);
        assert_eq!(bb.s, "$$");
        assert_eq!(bb.foo(), "$$ -99 2.5");
        assert_eq!(Arc::strong_count(&dyn_b), 2);

        let bbb = dyn_b.cast_as::<StructB>().unwrap();
        assert_eq!(Arc::strong_count(&bbb), 2);
        assert_eq!(bbb.foo(), "$$ -99 2.5");
        assert!(Arc::ptr_eq(&bb, &bbb));
    }
}
