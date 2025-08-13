use std::borrow::{Borrow, BorrowMut};
use std::hint::{likely, unlikely};
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};
use std::{fmt, mem, ptr, slice};

use crate::utils::{XResult, xres};

// #[macro_export]
// macro_rules! array_vec {
//   ($array_type:ty => $($elem:expr),* $(,)?) => {
//     {
//       let mut av: $crate::TinyVec<$array_type> = Default::default();
//       $( av.push($elem); )*
//       av
//     }
//   };
//   ($array_type:ty) => {
//     $crate::TinyVec::<$array_type>::default()
//   };
//   ($($elem:expr),*) => {
//     $crate::array_vec!(_ => $($elem),*)
//   };
//   ($elem:expr; $n:expr) => {
//     $crate::TinyVec::from([$elem; $n])
//   };
//   () => {
//     $crate::array_vec!(_)
//   };
// }

pub struct TinyVec<T, const C: usize> {
    len: u16,
    data: MaybeUninit<[T; C]>,
}

// impl<T, const C: usize> Deref for TinyVec<T, C> {
//     type Target = [T];
//     #[inline(always)]
//     fn deref(&self) -> &Self::Target {
//         &self.as_slice()
//     }
// }

// impl<T, const C: usize> DerefMut for TinyVec<T, C> {
//     #[inline(always)]
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.as_mut_slice()
//     }
// }

impl<T, const C: usize> Default for TinyVec<T, C> {
    #[inline]
    fn default() -> Self {
        debug_assert!(C < u16::MAX as usize);
        Self {
            len: 0,
            data: MaybeUninit::uninit(),
        }
    }
}

impl<T, const C: usize> Drop for TinyVec<T, C> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const C: usize> Clone for TinyVec<T, C>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let mut other = Self::default();
        other.extend_from_slice(self.as_slice());
        other
    }
}

impl<T, const C: usize> TinyVec<T, C> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn len(&self) -> u16 {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        C
    }

    #[inline(always)]
    pub fn push(&mut self, val: T) {
        self.try_push(val).unwrap();
    }

    #[inline(always)]
    pub fn try_push(&mut self, val: T) -> XResult<()> {
        let data = unsafe { self.data.assume_init_mut() };
        let Some(itemref) = data.get_mut(self.len as usize) else {
            return xres!(Overflow)
        };
        *itemref = val;
        self.len += 1;
        Ok(())
    }
    
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            unsafe {
                let data = self.data.assume_init_mut();
                Some(ptr::read(&data[self.len as usize]))
            }
        }
        else {
            None
        }
    }
    
    #[inline(always)]
    pub fn clear(&mut self) {
        if mem::needs_drop::<T>() {
            unsafe {
                let data = self.data.assume_init_mut();
                for idx in 0..self.len as usize {
                    ptr::drop_in_place::<T>(&mut data[idx]);
                }
            }
        }
        self.len = 0;
    }

    #[inline]
    pub fn insert(&mut self, pos: usize, item: T) {
        self.try_insert(pos, item).unwrap();
    }

    #[inline]
    pub fn try_insert(&mut self, pos: usize, mut item: T) -> XResult<()> {
        if likely((self.len as usize) < C) {
            self.len += 1;
        }
        else {
            return xres!(Overflow);
        }

        let data = unsafe { self.data.assume_init_mut() };
        for idx in pos..self.len as usize {
            mem::swap(&mut item, &mut data[idx]);
        }
        Ok(())
    }

    #[inline]
    pub fn remove(&mut self, pos: usize) -> T {
        if unlikely(pos > self.len as usize) {
            panic!("TinyVec::remove> index {} is out of bounds {}", pos, self.len);
        }

        let data = unsafe { self.data.assume_init_mut() };
        let item = unsafe { ptr::read(&mut data[pos]) };
        for idx in pos..(self.len-1) as usize {
            data.swap(idx, idx + 1);
        }
        self.len -= 1;
        item
    }

    #[inline]
    pub fn swap_remove(&mut self, pos: usize) -> T {
        if unlikely(pos > self.len as usize) {
            panic!("TinyVec::remove> index {} is out of bounds {}", pos, self.len);
        }

        let data = unsafe { self.data.assume_init_mut() };
        let item = unsafe { ptr::read(&mut data[pos]) };
        data.swap(pos, self.len as usize - 1);
        self.len -= 1;
        item
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.len as usize {
            return;
        }

        if mem::needs_drop::<T>() {
            unsafe {
                let data = self.data.assume_init_mut();
                for idx in new_len..self.len as usize {
                    ptr::drop_in_place::<T>(&mut data[idx]);
                }
            }
        }
        self.len = new_len as u16;
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let data = self.data.assume_init_ref();
            &data[..self.len as usize]
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let data = self.data.assume_init_mut();
            &mut data[..self.len as usize]
        }
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    // TODO: retain retain_mut drain
}

impl<T: Clone, const C: usize> TinyVec<T, C> {
    pub fn extend_from_slice(&mut self, other: &[T]) {
        for item in other {
            self.push(item.clone());
        }
    }

    pub fn try_extend_from_slice(&mut self, other: &[T]) -> XResult<()> {
        for item in other {
            self.try_push(item.clone())?;
        }
        Ok(())
    }

    pub fn append(&mut self, other: &mut Self) {
        self.extend_from_slice(other.as_slice());
    }

    pub fn try_append(&mut self, other: &mut Self) -> XResult<()> {
        self.try_extend_from_slice(other.as_slice())
    }
}

impl<T, const C: usize> Index<usize> for TinyVec<T, C> {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T, const C: usize> IndexMut<usize> for TinyVec<T, C> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T, const C: usize> AsMut<[T]> for TinyVec<T, C> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const C: usize> AsRef<[T]> for TinyVec<T, C> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const C: usize> Borrow<[T]> for TinyVec<T, C> {
    #[inline(always)]
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const C: usize> BorrowMut<[T]> for TinyVec<T, C> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const C: usize> PartialEq for TinyVec<T, C>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T, const C: usize> Eq for TinyVec<T, C> where T: Eq {}

impl<T, const C: usize> fmt::Debug for TinyVec<T, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> core::fmt::Result {
        self.as_slice().fmt(f)
    }
}
