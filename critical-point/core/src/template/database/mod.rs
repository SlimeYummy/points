mod base;
#[cfg(not(feature = "server-side"))]
mod client;
#[cfg(feature = "server-side")]
mod server;

#[cfg(not(feature = "server-side"))]
pub use client::*;
#[cfg(feature = "server-side")]
pub use server::*;

use std::any::TypeId;
use std::mem;

use crate::template::base::TmplAny;
use crate::utils::{xres, TmplID, XResult};

impl At<dyn TmplAny> {
    #[inline]
    pub fn cast<T: TmplAny + rkyv::Archive + 'static>(self) -> XResult<At<T>> {
        if self.as_archived().type_id() != TypeId::of::<T::Archived>() {
            return xres!(BadType; "invalid cast");
        }
        Ok(unsafe { mem::transmute::<At<dyn TmplAny>, At<T>>(self) })
    }

    #[inline]
    pub unsafe fn cast_unchecked<T: TmplAny + rkyv::Archive + 'static>(self) -> At<T> {
        mem::transmute::<At<dyn TmplAny>, At<T>>(self)
    }
}

impl TmplDatabase {
    #[inline]
    pub fn find_as<T: TmplAny + rkyv::Archive + 'static>(&self, id: TmplID) -> XResult<At<T>> {
        self.find(id)?.cast()
    }
}
