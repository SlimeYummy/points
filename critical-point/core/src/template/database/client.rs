use static_assertions::const_assert_eq;
use std::alloc::Layout;
use std::cell::{Cell, UnsafeCell};
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::rc::Rc;
use std::{alloc, fmt, fs, mem, ptr, slice, u32};

use crate::template2::base::{ArchivedTmplAny, TmplAny};
use crate::template2::database::base::{load_json_to_rkyv, load_rkyv_into, TmplIndexCache};
use crate::template2::id::{TmplID, TmplHashMap};
use crate::utils::{xerr, xfrom, xfromf, xres, xresf, IdentityState, XResult};

//
// Database
//

static mut INDEX_CACHE: TmplIndexCache = TmplIndexCache::EMPTY;

#[inline(always)]
fn index_cache() -> &'static TmplIndexCache {
    unsafe { &*(&raw const INDEX_CACHE) }
}

pub(crate) unsafe fn init_database_static<P: AsRef<Path>>(path: P) -> XResult<()> {
    #[allow(static_mut_refs)]
    if INDEX_CACHE.is_empty() {
        INDEX_CACHE = TmplIndexCache::from_file(path)?;
    }
    Ok(())
}

pub(crate) struct TmplDatabaseInner {
    is_rkyv: bool,
    is_aliving: bool,
    file: File,
    map: TmplHashMap<NonNull<AtInnerHeader>>,
    cache_head: *mut AtInnerCached,
    cache_tail: *mut AtInnerCached,
    size: usize,
    cached_size: usize,
    cached_size_limit: usize,
    current_frame: u32,
    cached_frame_limit: u32,
}

impl TmplDatabaseInner {
    fn from_file<P: AsRef<Path>>(
        path: P,
        cached_size_limit: usize,
        cached_frame_limit: u32,
    ) -> XResult<TmplDatabaseInner> {
        let path = PathBuf::from(path.as_ref());
        let rkyv_path = path.join("data.rkyv");
        let json_path = path.join("data.json");

        let (is_rkyv, file_path) = if fs::exists(&rkyv_path).unwrap_or(false) {
            (true, rkyv_path)
        } else if fs::exists(&json_path).unwrap_or(false) {
            (false, json_path)
        } else {
            return xresf!(NotFound; "path={:?}", &path);
        };

        let cache = TmplDatabaseInner {
            is_rkyv,
            is_aliving: true,
            file: File::open(&file_path).map_err(xfromf!("path={:?}", path))?,
            map: TmplHashMap::with_hasher(IdentityState),
            cache_head: Box::into_raw(Box::new(AtInnerCached::EMPTY)),
            cache_tail: Box::into_raw(Box::new(AtInnerCached::EMPTY)),
            size: 0,
            cached_size: 0,
            cached_size_limit,
            current_frame: 0,
            cached_frame_limit,
        };

        unsafe {
            (*cache.cache_head).next = cache.cache_tail;
            (*cache.cache_tail).prev = cache.cache_head;
        }
        Ok(cache)
    }

    fn find(&mut self, id: TmplID, database: &Rc<UnsafeCell<TmplDatabaseInner>>) -> XResult<At<dyn TmplAny>> {
        assert!(self.is_aliving);

        let map = &mut self.map;
        let file = &mut self.file;
        let is_rkyv = self.is_rkyv;
        let cached_size = &mut self.cached_size;
        match map.entry(id.clone()) {
            Entry::Occupied(entry) => Ok(Self::reuse_from_cache(cached_size, *entry.get(), database)),
            Entry::Vacant(entry) => {
                let at = Self::load_from_file(id, is_rkyv, file, database)?;
                entry.insert(AtInnerReferred::as_header(at.inner));
                self.size += at.size();
                Ok(at)
            }
        }
    }

    fn reuse_from_cache(
        cached_size: &mut usize,
        header: NonNull<AtInnerHeader>,
        database: &Rc<UnsafeCell<TmplDatabaseInner>>,
    ) -> At<dyn TmplAny> {
        unsafe {
            if header.as_ref().ref_count.get() != 0 {
                let referred = NonNull::new_unchecked(header.as_ptr() as *mut AtInnerReferred);
                return At::<dyn TmplAny>::from_inner(referred);
            } else {
                let mut cached = NonNull::new_unchecked(header.as_ptr() as *mut AtInnerCached);
                (*cached.as_mut().next).prev = cached.as_mut().prev;
                (*cached.as_mut().prev).next = cached.as_mut().next;

                let mut referred = AtInnerReferred::from_cached(cached);
                referred.as_mut().database = Some(database.clone());
                *cached_size -= referred.as_ref().h.size as usize;
                return At::<dyn TmplAny>::from_inner(referred);
            }
        }
    }

    fn load_from_file(
        id: TmplID,
        is_rkyv: bool,
        file: &mut File,
        database: &Rc<UnsafeCell<TmplDatabaseInner>>,
    ) -> XResult<At<dyn TmplAny>> {
        let index = index_cache().find(id).ok_or_else(|| xerr!(TmplNotFound, id))?;
        let inner = if is_rkyv {
            unsafe { AtInnerReferred::new(index.len as usize, id, |buf| load_rkyv_into(file, id, index, buf))? }
        } else {
            let rkyv_buf = load_json_to_rkyv(file, id, index)?;
            unsafe {
                AtInnerReferred::new(rkyv_buf.len(), id, |buf: &mut [u8]| {
                    buf[..rkyv_buf.len()].copy_from_slice(&rkyv_buf);
                    Ok(())
                })?
            }
        };
        let mut at: At<dyn TmplAny> = unsafe { At::from_inner(inner) };
        at.inner_mut().database = Some(database.clone());
        Ok(at)
    }

    fn free<T: ?Sized>(&mut self, at: &mut At<T>) {
        let inner = at.inner();
        inner.h.ref_count.set(inner.h.ref_count.get() - 1);

        if inner.h.ref_count.get() == 0 {
            let mut cached = AtInnerCached::from_referred(at.inner);

            if !self.is_aliving {
                unsafe { AtInnerCached::delete(cached) };
            } else {
                unsafe {
                    cached.as_mut().cached_frame = self.current_frame;

                    cached.as_mut().next = (*self.cache_head).next;
                    cached.as_mut().prev = self.cache_head;
                    (*self.cache_head).next = cached.as_ptr();
                    (*cached.as_mut().next).prev = cached.as_ptr();

                    self.cached_size += cached.as_mut().h.size as usize;
                }
                self.delete_by(|zelf, _| zelf.cached_size > zelf.cached_size_limit);
            }
        }
    }

    fn free_all(&mut self) {
        if self.is_aliving {
            self.delete_by(|_, _| true);

            self.map.clear();
            self.map.shrink_to_fit();

            unsafe {
                let _ = Box::from_raw(self.cache_head);
                let _ = Box::from_raw(self.cache_tail);
            }
            self.is_aliving = false;
        }
    }

    #[inline(always)]
    fn delete_by<F: Fn(&mut Self, &AtInnerCached) -> bool>(&mut self, by: F) {
        unsafe {
            loop {
                let deleting = (*self.cache_tail).prev;
                if deleting == self.cache_head {
                    break;
                }

                if !by(self, &*deleting) {
                    break;
                }

                self.size -= (*deleting).h.size as usize;
                self.cached_size -= (*deleting).h.size as usize;
                self.map.remove(&(*deleting).h.id);

                (*(*deleting).prev).next = (*deleting).next;
                (*(*deleting).next).prev = (*deleting).prev;
                AtInnerCached::delete(NonNull::new_unchecked(deleting));
            }
        }
    }
}

pub struct TmplDatabase {
    inner: Rc<UnsafeCell<TmplDatabaseInner>>,
}

impl Drop for TmplDatabase {
    fn drop(&mut self) {
        self.inner().free_all();
    }
}

impl TmplDatabase {
    #[inline]
    fn inner(&self) -> &mut TmplDatabaseInner {
        unsafe { &mut *self.inner.get() }
    }

    pub fn new(cached_size: usize, cached_frame_limit: u32) -> XResult<TmplDatabase> {
        Ok(TmplDatabase {
            inner: Rc::new(UnsafeCell::new(TmplDatabaseInner::from_file(
                index_cache().path(),
                cached_size,
                cached_frame_limit,
            )?)),
        })
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.inner().size as usize
    }

    #[inline]
    pub fn cached_size(&self) -> usize {
        self.inner().cached_size as usize
    }

    #[inline]
    pub fn find(&self, id: TmplID) -> XResult<At<dyn TmplAny>> {
        self.inner().find(id, &self.inner)
    }

    #[inline]
    pub fn update_frame(&self, current_frame: u32) {
        self.inner().current_frame = current_frame;
        self.inner()
            .delete_by(|zelf, inner| inner.cached_frame + zelf.cached_frame_limit <= current_frame);
    }
}

//
// Archived Template
//

struct AtInnerHeader {
    id: TmplID,
    ref_count: Cell<u32>,
    size: u32,
    tmpl_size: u32,
}

#[repr(C, align(16))]
struct AtInnerReferred {
    h: AtInnerHeader,
    database: Option<Rc<UnsafeCell<TmplDatabaseInner>>>,
    archived_ref: [*const (); 2],
}

#[repr(C, align(16))]
struct AtInnerCached {
    h: AtInnerHeader,
    cached_frame: u32,
    next: *mut AtInnerCached,
    prev: *mut AtInnerCached,
}

const AT_INNER_SIZE: usize = mem::size_of::<AtInnerReferred>();
const_assert_eq!(AT_INNER_SIZE, mem::size_of::<AtInnerCached>());

impl AtInnerReferred {
    unsafe fn new<F: FnOnce(&mut [u8]) -> XResult<()>>(
        tmpl_size: usize,
        id: TmplID,
        initialize: F,
    ) -> XResult<NonNull<AtInnerReferred>> {
        let size = AT_INNER_SIZE + (tmpl_size + 0xF) & !0xF;
        let mut inner =
            NonNull::new_unchecked(alloc::alloc(Layout::from_size_align_unchecked(size, 16)) as *mut AtInnerReferred);
        inner.as_mut().h.id = id;
        inner.as_mut().h.ref_count = Cell::new(0);
        inner.as_mut().h.size = size as u32;
        inner.as_mut().h.tmpl_size = tmpl_size as u32;
        (&mut inner.as_mut().database as *mut Option<Rc<UnsafeCell<TmplDatabaseInner>>>).write(None);

        initialize(inner.as_mut().buf_mut())?;
        inner.as_mut().padding_buf_mut().fill(0);

        let archived_ref = rkyv::archived_unsized_root::<dyn TmplAny>(inner.as_ref().buf());
        inner.as_mut().archived_ref = mem::transmute::<&dyn ArchivedTmplAny, [*const (); 2]>(archived_ref);
        return Ok(inner);
    }

    #[inline]
    unsafe fn buf(&self) -> &[u8] {
        let ptr = (self as *const _ as *const u8).add(AT_INNER_SIZE);
        slice::from_raw_parts(ptr, self.h.tmpl_size as usize)
    }

    #[inline]
    unsafe fn buf_mut(&mut self) -> &mut [u8] {
        let ptr = (self as *mut _ as *mut u8).add(AT_INNER_SIZE);
        slice::from_raw_parts_mut(ptr, self.h.tmpl_size as usize)
    }

    #[inline]
    unsafe fn padding_buf_mut(&mut self) -> &mut [u8] {
        let unpadding_size = AT_INNER_SIZE + self.h.tmpl_size as usize;
        let ptr = (self as *mut _ as *mut u8).add(unpadding_size);
        slice::from_raw_parts_mut(ptr, self.h.size as usize - unpadding_size)
    }

    #[inline]
    fn clear(&mut self) {
        self.database = None;
        self.archived_ref = [ptr::null(); 2];
    }

    #[inline]
    fn as_header(mut referred: NonNull<AtInnerReferred>) -> NonNull<AtInnerHeader> {
        unsafe { NonNull::new_unchecked(&mut referred.as_mut().h) }
    }

    #[inline]
    fn from_cached(mut cached: NonNull<AtInnerCached>) -> NonNull<AtInnerReferred> {
        unsafe {
            assert_eq!(cached.as_ref().h.ref_count.get(), 0);

            cached.as_mut().clear();
            let mut referred = NonNull::new_unchecked(cached.as_ptr() as *mut _ as *mut AtInnerReferred);
            let archived_ref = rkyv::archived_unsized_root::<dyn TmplAny>(referred.as_ref().buf());
            referred.as_mut().archived_ref = mem::transmute::<&dyn ArchivedTmplAny, [*const (); 2]>(archived_ref);
            referred
        }
    }
}

impl AtInnerCached {
    const EMPTY: AtInnerCached = AtInnerCached {
        h: AtInnerHeader {
            id: TmplID::INVALID,
            ref_count: Cell::new(0),
            size: 0,
            tmpl_size: 0,
        },
        cached_frame: 0,
        next: ptr::null_mut(),
        prev: ptr::null_mut(),
    };

    #[inline]
    unsafe fn delete(inner: NonNull<AtInnerCached>) {
        alloc::dealloc(
            inner.as_ptr() as *mut u8,
            Layout::from_size_align_unchecked(inner.as_ref().h.size as usize, 16),
        );
    }

    #[inline]
    fn clear(&mut self) {
        self.next = ptr::null_mut();
        self.prev = ptr::null_mut();
        self.cached_frame = 0;
    }

    #[inline]
    fn from_referred(mut referred: NonNull<AtInnerReferred>) -> NonNull<AtInnerCached> {
        unsafe {
            assert_eq!(referred.as_ref().h.ref_count.get(), 0);

            referred.as_mut().clear();
            NonNull::new_unchecked(referred.as_ptr() as *mut _ as *mut AtInnerCached)
        }
    }
}

pub struct At<T: ?Sized = dyn TmplAny> {
    inner: NonNull<AtInnerReferred>,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized> At<T> {
    #[inline]
    fn header(&self) -> &AtInnerHeader {
        unsafe { &self.inner.as_ref().h }
    }

    #[inline]
    fn inner(&self) -> &AtInnerReferred {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut AtInnerReferred {
        unsafe { self.inner.as_mut() }
    }
}

impl<T: ?Sized + TmplAny> At<T> {
    #[inline]
    unsafe fn from_inner(inner: NonNull<AtInnerReferred>) -> At<T> {
        let at: At<T> = At {
            inner,
            _phantom: PhantomData,
        };
        let header = at.header();
        header.ref_count.set(header.ref_count.get() + 1);
        at
    }

    #[inline]
    pub fn ref_count(&self) -> u32 {
        self.header().ref_count.get()
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.header().size as usize
    }
}

impl<T: ?Sized + TmplAny> Clone for At<T> {
    #[inline]
    fn clone(&self) -> At<T> {
        let header = self.header();
        header.ref_count.set(header.ref_count.get() + 1);
        At {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for At<T> {
    #[inline]
    fn drop(&mut self) {
        let db = self.inner_mut().database.as_mut().unwrap().get();
        unsafe { &mut *db }.free(self)
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

#[cfg(test)]
#[ctor::ctor]
fn test_init_database_static() {
    unsafe {
        crate::template2::id::init_id_static("../../test-tmp/resource").unwrap();
        init_database_static("../../test-tmp/resource").unwrap();
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template2::id::id;

    fn db_all_count(db: &TmplDatabase) -> usize {
        db.inner().map.len()
    }

    fn db_cache_count(db: &TmplDatabase) -> usize {
        let inner = db.inner();
        let mut count = 0;
        unsafe {
            let mut iter = (*inner.cache_head).next;
            while iter != inner.cache_tail {
                count += 1;
                iter = (*iter).next;
            }
        }
        count
    }

    #[test]
    fn test_tmpl_database_lifecycle() {
        let db = TmplDatabase::new(1024, 2).unwrap();
        assert_eq!(db_all_count(&db), 0);
        assert_eq!(db_cache_count(&db), 0);

        let stage1 = db.find(id!("Stage.Demo")).unwrap();
        let stage_size = stage1.size();
        assert_eq!(stage1.ref_count(), 1);
        assert_eq!(db_all_count(&db), 1);
        assert_eq!(db_cache_count(&db), 0);
        assert_eq!(db.size(), stage_size);
        assert_eq!(db.cached_size(), 0);

        let stage2 = stage1.clone();
        assert_eq!(stage2.ref_count(), 2);
        assert_eq!(db_all_count(&db), 1);
        assert_eq!(db_cache_count(&db), 0);
        assert_eq!(db.size(), stage_size);
        assert_eq!(db.cached_size(), 0);
        mem::drop(stage2);
        assert_eq!(stage1.ref_count(), 1);

        mem::drop(stage1);
        assert_eq!(db_all_count(&db), 1);
        assert_eq!(db_cache_count(&db), 1);
        assert_eq!(db.size(), stage_size);
        assert_eq!(db.cached_size(), stage_size);

        let entry = db.find(id!("Entry.MaxHealthUp")).unwrap();
        let entry_size = entry.size();
        assert_eq!(entry.ref_count(), 1);
        assert_eq!(db.size(), stage_size + entry_size);
        assert_eq!(db.cached_size(), stage_size);

        let stage3 = db.find(id!("Stage.Demo")).unwrap();
        assert_eq!(stage3.ref_count(), 1);
        assert_eq!(db_all_count(&db), 2);
        assert_eq!(db_cache_count(&db), 0);
        assert_eq!(db.size(), stage_size + entry_size);
        assert_eq!(db.cached_size(), 0);

        mem::drop(stage3);
        assert_eq!(db_all_count(&db), 2);
        assert_eq!(db_cache_count(&db), 1);
        assert_eq!(db.size(), stage_size + entry_size);
        assert_eq!(db.cached_size(), stage_size);

        db.inner().free_all();
        assert_eq!(db_all_count(&db), 0);
        assert_eq!(db_cache_count(&db), 0);
        assert_eq!(db.size(), entry_size);
        assert_eq!(db.cached_size(), 0);

        let inner = db.inner.clone();
        mem::drop(db);
        assert_eq!(Rc::strong_count(&inner), 2);
        mem::drop(entry);
        assert_eq!(Rc::strong_count(&inner), 1);
    }

    #[test]
    fn test_tmpl_database_auto_free() {
        // Cached size must > 2 Entry && < 3 Entry
        let db = TmplDatabase::new(600, 2).unwrap();
        assert_eq!(db_all_count(&db), 0);
        assert_eq!(db_cache_count(&db), 0);

        let entry1 = db.find(id!("Entry.MaxHealthUp")).unwrap();
        let entry1_size = entry1.size();
        assert_eq!(db_all_count(&db), 1);
        assert_eq!(db_cache_count(&db), 0);
        assert_eq!(db.size(), entry1_size);
        assert_eq!(db.cached_size(), 0);

        let entry2 = db.find(id!("Entry.AttackUp")).unwrap();
        let entry2_size = entry2.size();
        assert_eq!(db_all_count(&db), 2);
        assert_eq!(db_cache_count(&db), 0);
        assert_eq!(db.size(), entry1_size + entry2_size);
        assert_eq!(db.cached_size(), 0);

        let entry3 = db.find(id!("Entry.CriticalDamage")).unwrap();
        let entry3_size = entry3.size();
        assert_eq!(db_all_count(&db), 3);
        assert_eq!(db_cache_count(&db), 0);
        assert_eq!(db.size(), entry1_size + entry2_size + entry3_size);
        assert_eq!(db.cached_size(), 0);

        mem::drop(entry1);
        assert_eq!(db.size(), entry1_size + entry2_size + entry3_size);
        assert_eq!(db.cached_size(), entry1_size);
        assert_eq!(db_all_count(&db), 3);
        assert_eq!(db_cache_count(&db), 1);

        mem::drop(entry2);
        assert_eq!(db.size(), entry1_size + entry2_size + entry3_size);
        assert_eq!(db.cached_size(), entry1_size + entry2_size);
        assert_eq!(db_all_count(&db), 3);
        assert_eq!(db_cache_count(&db), 2);

        mem::drop(entry3);
        assert_eq!(db.size(), entry2_size + entry3_size);
        assert_eq!(db.cached_size(), entry2_size + entry3_size);
        assert_eq!(db_all_count(&db), 2);
        assert_eq!(db_cache_count(&db), 2);

        db.update_frame(1);
        assert_eq!(db_all_count(&db), 2);
        assert_eq!(db_cache_count(&db), 2);

        db.update_frame(2);
        assert_eq!(db_all_count(&db), 0);
        assert_eq!(db_cache_count(&db), 0);
    }
}
