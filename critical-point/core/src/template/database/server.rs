use std::alloc::Layout;
use std::fmt::Debug;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::{alloc, fmt, fs, mem, slice, u32};

use crate::template2::base::{ArchivedTmplAny, TmplAny};
use crate::template2::database::base::{load_json_to_rkyv, load_rkyv_into, TmplIndexCache};
use crate::template2::id::{TmplID, TmplHashMap};
use crate::utils::{AsXResultIO, IdentityState, XResult};

//
// Database
//

static mut DATABASE_CACHE: TmplDatabaseCache = TmplDatabaseCache::EMPTY;

#[inline(always)]
fn database_cache() -> &'static TmplDatabaseCache {
    unsafe { &*(&raw const DATABASE_CACHE) }
}

pub(crate) unsafe fn init_database_static<P: AsRef<Path>>(path: P) -> XResult<()> {
    #[allow(static_mut_refs)]
    if DATABASE_CACHE.is_empty() {
        DATABASE_CACHE = TmplDatabaseCache::from_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
#[ctor::ctor]
fn test_init_database_static() {
    unsafe {
        crate::template2::id::init_id_static("../../test-tmp/resource").unwrap();
        init_database_static("../../test-tmp/resource").unwrap();
    };
}

struct TmplDatabaseCache {
    map: TmplHashMap<NonNull<AtInner>>,
    size: usize,
}

impl TmplDatabaseCache {
    const EMPTY: TmplDatabaseCache = TmplDatabaseCache {
        map: TmplHashMap::with_hasher(IdentityState),
        size: 0,
    };

    fn from_file<P: AsRef<Path>>(path: P) -> XResult<TmplDatabaseCache> {
        let index_cache = TmplIndexCache::from_file(path.as_ref())?;

        let path = PathBuf::from(path.as_ref());
        let rkyv_path = path.join("data.rkyv");
        let json_path = path.join("data.json");

        let (is_rkyv, file_path) = if fs::exists(&rkyv_path).unwrap_or(false) {
            (true, rkyv_path)
        } else if fs::exists(&json_path).unwrap_or(false) {
            (false, json_path)
        } else {
            return Err(XError2::bad_argument("TmplDatabaseCache::from_file()"));
        };
        let mut file = File::open(&file_path).xerr_with(&path)?;

        let mut map = TmplHashMap::with_capacity_and_hasher(index_cache.len(), IdentityState);
        let mut size = 0;
        for (id, index) in index_cache.iter() {
            let inner = if is_rkyv {
                unsafe {
                    AtInner::new(index.len as usize, *id, |buf| {
                        load_rkyv_into(&mut file, *id, *index, buf)
                    })?
                }
            } else {
                let rkyv_buf = load_json_to_rkyv(&mut file, *id, *index)?;
                unsafe {
                    AtInner::new(rkyv_buf.len(), *id, |buf: &mut [u8]| {
                        buf[..rkyv_buf.len()].copy_from_slice(&rkyv_buf);
                        Ok(())
                    })?
                }
            };
            map.insert(*id, inner);
            size += unsafe { inner.as_ref().size as usize };
        }

        Ok(TmplDatabaseCache { map, size })
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

pub struct TmplDatabase {}

impl TmplDatabase {
    #[inline]
    pub fn new(_cached_size: usize, _cached_frame_limit: u32) -> XResult<TmplDatabase> {
        Ok(TmplDatabase {})
    }

    #[inline]
    pub fn size(&self) -> usize {
        database_cache().size as usize
    }

    #[inline]
    pub fn cached_size(&self) -> usize {
        database_cache().size as usize
    }

    #[inline]
    pub fn find(&self, id: TmplID) -> XResult<At<dyn TmplAny>> {
        database_cache().find(id)
    }

    #[inline]
    pub fn update_frame(&self, _current_frame: u32) {}
}

//
// Archived Template
//

#[repr(C, align(16))]
struct AtInner {
    id: TmplID,
    size: u32,
    tmpl_size: u32,
    archived_ref: [*const (); 2],
}

const AT_INNER_SIZE: usize = mem::size_of::<AtInner>();

impl AtInner {
    unsafe fn new<F: FnOnce(&mut [u8]) -> XResult<()>>(
        tmpl_size: usize,
        id: TmplID,
        initialize: F,
    ) -> XResult<NonNull<AtInner>> {
        let size = AT_INNER_SIZE + (tmpl_size + 0xF) & !0xF;
        let mut inner =
            NonNull::new_unchecked(alloc::alloc(Layout::from_size_align_unchecked(size, 16)) as *mut AtInner);
        inner.as_mut().id = id;
        inner.as_mut().size = tmpl_size as u32;
        inner.as_mut().tmpl_size = tmpl_size as u32;

        initialize(inner.as_mut().buf_mut())?;
        inner.as_mut().padding_buf_mut().fill(0);

        let archived_ref = rkyv::archived_unsized_root::<dyn TmplAny>(inner.as_ref().buf());
        inner.as_mut().archived_ref = mem::transmute::<&dyn ArchivedTmplAny, [*const (); 2]>(archived_ref);
        return Ok(inner);
    }

    #[inline]
    unsafe fn buf(&self) -> &[u8] {
        let ptr = (self as *const _ as *const u8).add(AT_INNER_SIZE);
        slice::from_raw_parts(ptr, self.tmpl_size as usize)
    }

    #[inline]
    unsafe fn buf_mut(&mut self) -> &mut [u8] {
        let ptr = (self as *mut _ as *mut u8).add(AT_INNER_SIZE);
        slice::from_raw_parts_mut(ptr, self.tmpl_size as usize)
    }

    #[inline]
    unsafe fn padding_buf_mut(&mut self) -> &mut [u8] {
        let unpadding_size = AT_INNER_SIZE + self.tmpl_size as usize;
        let ptr = (self as *mut _ as *mut u8).add(unpadding_size);
        slice::from_raw_parts_mut(ptr, self.size as usize - unpadding_size)
    }
}

pub struct At<T: ?Sized = dyn TmplAny> {
    inner: NonNull<AtInner>,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized + TmplAny> At<T> {
    #[inline]
    fn inner(&self) -> &AtInner {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut AtInner {
        unsafe { self.inner.as_mut() }
    }
}

impl<T: ?Sized + TmplAny> At<T> {
    #[inline]
    unsafe fn from_inner(inner: NonNull<AtInner>) -> At<T> {
        At {
            inner,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn ref_count(&self) -> u32 {
        u32::MAX
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.inner().size as usize
    }
}

impl<T: ?Sized + TmplAny> Clone for At<T> {
    #[inline]
    fn clone(&self) -> At<T> {
        At {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }
}

impl At<dyn TmplAny> {
    #[inline]
    pub fn as_archived(&self) -> &(dyn ArchivedTmplAny + 'static) {
        unsafe { mem::transmute::<[*const (); 2], &dyn ArchivedTmplAny>(self.inner().archived_ref) }
    }
}

impl AsRef<dyn ArchivedTmplAny> for At<dyn TmplAny> {
    #[inline]
    fn as_ref(&self) -> &(dyn ArchivedTmplAny + 'static) {
        self.as_archived()
    }
}

impl Deref for At<dyn TmplAny> {
    type Target = dyn ArchivedTmplAny + 'static;
    #[inline]
    fn deref(&self) -> &(dyn ArchivedTmplAny + 'static) {
        self.as_archived()
    }
}

impl fmt::Debug for At<dyn TmplAny> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_archived().fmt(f)
    }
}

impl<T: ?Sized + TmplAny + rkyv::Archive> At<T> {
    #[inline]
    pub fn as_archived(&self) -> &T::Archived {
        unsafe { &*(self.inner().archived_ref[0] as *const T::Archived) }
    }
}

impl<T: ?Sized + TmplAny + rkyv::Archive> AsRef<T::Archived> for At<T> {
    #[inline]
    fn as_ref(&self) -> &T::Archived {
        self.as_archived()
    }
}

impl<T: ?Sized + TmplAny + rkyv::Archive> Deref for At<T> {
    type Target = T::Archived;
    #[inline]
    fn deref(&self) -> &T::Archived {
        self.as_archived()
    }
}

impl<T> fmt::Debug for At<T>
where
    T: ?Sized + TmplAny + rkyv::Archive,
    T::Archived: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_archived().fmt(f)
    }
}
