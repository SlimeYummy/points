use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;
use std::{mem, ptr};

use crate::utils::{XError, XResult};

pub fn const_ptr<T, U>(val: &T) -> *const U {
    return (val as *const T) as *const U;
}

pub fn mut_ptr<T, U>(val: &mut T) -> *mut U {
    return (val as *mut T) as *mut U;
}

pub fn size_of_type<T: Sized>() -> usize {
    return (mem::size_of::<T>() + mem::size_of::<usize>() - 1) / mem::size_of::<usize>();
}

pub fn size_of_array<T: Sized>(len: usize) -> usize {
    return (mem::size_of::<T>() * len + mem::size_of::<usize>() - 1) / mem::size_of::<usize>();
}

pub trait CastRef {
    fn cast<T: Any>(&self) -> XResult<&T>;
    fn cast_mut<T: Any>(&mut self) -> XResult<&mut T>;
    unsafe fn cast_unchecked<T: Any>(&self) -> &T;
    unsafe fn cast_mut_unchecked<T: Any>(&mut self) -> &mut T;
}

impl<TO: ?Sized> CastRef for TO {
    fn cast<T: Any>(&self) -> XResult<&T> {
        ref_check_variant::<TO, T>(&self)?;
        return Ok(unsafe { self.cast_unchecked() });
    }

    fn cast_mut<T: Any>(&mut self) -> XResult<&mut T> {
        ref_check_variant::<TO, T>(&self)?;
        return Ok(unsafe { self.cast_mut_unchecked() });
    }

    unsafe fn cast_unchecked<T: Any>(&self) -> &T {
        let (src_data, _) = (self as *const TO).to_raw_parts();
        return &*(src_data as *const T);
    }

    unsafe fn cast_mut_unchecked<T: Any>(&mut self) -> &mut T {
        let (src_data, _) = (self as *mut TO).to_raw_parts();
        return &mut *(src_data as *mut T);
    }
}

fn ref_check_variant<TO: ?Sized, T: Any>(re: &TO) -> XResult<()> {
    let src_meta = ptr::metadata(re as *const TO);
    let src_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&src_meta) };

    let dst_ref: &dyn Any = unsafe { mem::transmute_copy::<usize, &T>(&0) };
    let dst_meta = ptr::metadata(dst_ref);
    let dst_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&dst_meta) };

    if src_drop != dst_drop {
        return Err(XError::BadType);
    }
    return Ok(());
}

pub trait CastRc {
    fn cast_as<T: Any>(self) -> XResult<Rc<T>>;
    fn cast_to<T: Any>(&self) -> XResult<Rc<T>>;
    fn cast_ref<T: Any>(&self) -> XResult<&T>;
    unsafe fn cast_as_unchecked<T: Any>(self) -> Rc<T>;
    unsafe fn cast_to_unchecked<T: Any>(&self) -> Rc<T>;
    unsafe fn cast_ref_unchecked<T: Any>(&self) -> &T;
}

impl<TO: ?Sized> CastRc for Rc<TO> {
    fn cast_as<T: Any>(self) -> XResult<Rc<T>> {
        rc_check_variant::<TO, T>(&self)?;
        return Ok(unsafe { self.cast_as_unchecked() });
    }

    fn cast_to<T: Any>(&self) -> XResult<Rc<T>> {
        return self.clone().cast_as();
    }

    fn cast_ref<T: Any>(&self) -> XResult<&T> {
        rc_check_variant::<TO, T>(&self)?;
        return Ok(unsafe { self.cast_ref_unchecked() });
    }

    unsafe fn cast_as_unchecked<T: Any>(self) -> Rc<T> {
        let (src_data, _) = Rc::into_raw(self).to_raw_parts();
        let dst_arc = unsafe { Rc::from_raw(src_data as *const T) };
        return dst_arc;
    }

    unsafe fn cast_to_unchecked<T: Any>(&self) -> Rc<T> {
        return self.clone().cast_as_unchecked();
    }

    unsafe fn cast_ref_unchecked<T: Any>(&self) -> &T {
        let (src_data, _) = Rc::into_raw(self.clone()).to_raw_parts();
        let dst_ref = unsafe { &*(src_data as *const T) };
        return dst_ref;
    }
}

fn rc_check_variant<TO: ?Sized, T: Any>(rc: &Rc<TO>) -> XResult<()> {
    let src_meta = ptr::metadata(Rc::as_ptr(rc));
    let src_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&src_meta) };

    let dst_ref: &dyn Any = unsafe { mem::transmute_copy::<usize, &T>(&0) };
    let dst_meta = ptr::metadata(dst_ref);
    let dst_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&dst_meta) };

    if src_drop != dst_drop {
        return Err(XError::BadType);
    }
    return Ok(());
}

pub trait CastArc {
    fn cast_as<T: Any>(self) -> XResult<Arc<T>>;
    fn cast_to<T: Any>(&self) -> XResult<Arc<T>>;
    fn cast_ref<T: Any>(&self) -> XResult<&T>;
    unsafe fn cast_as_unchecked<T: Any>(self) -> Arc<T>;
    unsafe fn cast_to_unchecked<T: Any>(&self) -> Arc<T>;
    unsafe fn cast_ref_unchecked<T: Any>(&self) -> &T;
}

impl<TO: ?Sized> CastArc for Arc<TO> {
    fn cast_as<T: Any>(self) -> XResult<Arc<T>> {
        arc_check_variant::<TO, T>(&self)?;
        return Ok(unsafe { self.cast_as_unchecked() });
    }

    fn cast_to<T: Any>(&self) -> XResult<Arc<T>> {
        return self.clone().cast_as();
    }

    fn cast_ref<T: Any>(&self) -> XResult<&T> {
        arc_check_variant::<TO, T>(&self)?;
        return Ok(unsafe { self.cast_ref_unchecked() });
    }

    unsafe fn cast_as_unchecked<T: Any>(self) -> Arc<T> {
        let (src_data, _) = Arc::into_raw(self).to_raw_parts();
        let dst_arc = Arc::from_raw(src_data as *const T);
        return dst_arc;
    }

    unsafe fn cast_to_unchecked<T: Any>(&self) -> Arc<T> {
        return self.clone().cast_as_unchecked();
    }

    unsafe fn cast_ref_unchecked<T: Any>(&self) -> &T {
        let (src_data, _) = Arc::into_raw(self.clone()).to_raw_parts();
        let dst_ref = &*(src_data as *const T);
        return dst_ref;
    }
}

fn arc_check_variant<TO: ?Sized, T: Any>(arc: &Arc<TO>) -> XResult<()> {
    let src_meta = ptr::metadata(Arc::as_ptr(arc));
    let src_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&src_meta) };

    let dst_ref: &dyn Any = unsafe { mem::transmute_copy::<usize, &T>(&0) };
    let dst_meta = ptr::metadata(dst_ref);
    let dst_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&dst_meta) };

    if src_drop != dst_drop {
        return Err(XError::BadType);
    }
    return Ok(());
}

#[cfg(not(feature = "server-side"))]
mod x {
    pub type Xrc<T> = std::rc::Rc<T>;
    pub type Xweak<T> = std::rc::Weak<T>;
    pub trait Xcast = super::CastRc;
}

#[cfg(feature = "server-side")]
mod x {
    pub type Xrc<T> = std::sync::Arc<T>;
    pub type Xweak<T> = std::sync::Weak<T>;
    pub trait Xcast = super::CastArc;
}

pub use x::*;
