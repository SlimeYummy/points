mod base;
#[cfg(not(feature = "server"))]
mod client;
#[cfg(feature = "server")]
mod server;

#[cfg(not(feature = "server"))]
pub use client::{At, TmplDatabase};
#[cfg(feature = "server")]
pub use server::{At, TmplDatabase};

use crate::template2::base::TmplAny;
use crate::template2::id::TmplID;
use crate::utils::{CastRef, XResult};
use std::mem;

impl At<dyn TmplAny> {
    #[inline]
    pub fn cast_as<T: TmplAny + rkyv::Archive + 'static>(self) -> XResult<At<T>> {
        self.as_archived().cast_ref::<T::Archived>()?;
        Ok(unsafe { mem::transmute::<At<dyn TmplAny>, At<T>>(self) })
    }

    #[inline]
    pub unsafe fn cast_as_unchecked<T: TmplAny + rkyv::Archive + 'static>(self) -> At<T> {
        unsafe { mem::transmute::<At<dyn TmplAny>, At<T>>(self) }
    }

    #[inline]
    pub fn cast_to<T: TmplAny + rkyv::Archive + 'static>(&self) -> XResult<At<T>> {
        self.as_archived().cast_ref::<T::Archived>()?;
        Ok(unsafe { mem::transmute::<At<dyn TmplAny>, At<T>>(self.clone()) })
    }

    #[inline]
    pub unsafe fn cast_to_unchecked<T: TmplAny + rkyv::Archive + 'static>(&self) -> At<T> {
        unsafe { mem::transmute::<At<dyn TmplAny>, At<T>>(self.clone()) }
    }
}

impl TmplDatabase {
    #[inline]
    pub fn find_as<T: TmplAny + rkyv::Archive + 'static>(&self, id: TmplID) -> XResult<At<T>> {
        self.find(id)?.cast_as()
    }
}
