use std::hint::unlikely;
use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};
use std::{fmt, slice};

use crate::utils::XResult;

pub struct HistoryVec<T> {
    vec: Vec<T>,
    current_end: usize,
}

impl<T> Default for HistoryVec<T> {
    #[inline]
    fn default() -> HistoryVec<T> {
        Self::new()
    }
}

impl<T> HistoryVec<T> {
    #[inline]
    pub const fn new() -> HistoryVec<T> {
        HistoryVec {
            vec: Vec::new(),
            current_end: 0,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> HistoryVec<T> {
        HistoryVec {
            vec: Vec::with_capacity(capacity),
            current_end: 0,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current_end == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.current_end
    }

    #[inline]
    pub fn all_len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn future_len(&self) -> usize {
        self.vec.len() - self.current_end
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < (self.current_end) {
            return Some(&self.vec[index]);
        }
        None
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < (self.current_end) {
            return Some(&mut self.vec[index]);
        }
        None
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        return self.vec[..self.current_end].iter();
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        return self.vec[..self.current_end].iter_mut();
    }

    #[inline]
    pub fn append_reuse<R>(&mut self, reuse: R) -> XResult<Option<&mut T>>
    where
        R: FnOnce(&mut T) -> XResult<bool>,
    {
        let end = self.current_end;
        if end < self.vec.len() && reuse(&mut self.vec[end])? {
            self.current_end += 1;
            return Ok(Some(&mut self.vec[end]));
        }
        Ok(None)
    }

    #[inline]
    pub fn append_new(&mut self, item: T) {
        self.vec.truncate(self.current_end as usize);
        self.current_end += 1;
        self.vec.push(item);
    }

    #[inline]
    pub fn append<R, N>(&mut self, reuse: R, new: N) -> XResult<&mut T>
    where
        R: FnOnce(&mut T) -> XResult<bool>,
        N: FnOnce() -> XResult<T>,
    {
        let end = self.current_end;
        if end < self.vec.len() && reuse(&mut self.vec[end])? {
            self.current_end += 1;
            return Ok(&mut self.vec[end]);
        }

        self.vec.truncate(end);
        self.current_end += 1;
        self.vec.push(new()?);
        Ok(&mut self.vec[end])
    }

    // func returns:
    // - 0 to restore the element
    // - -1 to skip the element
    // - 1 to stop restoring
    #[inline]
    pub fn restore<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut T) -> i32,
    {
        let _ = self.restore_when(|p| Ok(func(p)));
    }

    #[inline]
    pub fn restore_when<F>(&mut self, mut func: F) -> XResult<()>
    where
        F: FnMut(&mut T) -> XResult<i32>,
    {
        let mut new_end = 0;
        for idx in 0..(self.current_end) {
            let res = func(&mut self.vec[idx])?;
            if res < 0 {
                new_end += 1;
            }
            else if res == 0 {
                new_end += 1;
            }
            else {
                break;
            }
        }
        self.current_end = new_end;
        Ok(())
    }

    // func returns:
    // - true to discard the element
    // - false to stop discarding
    #[inline]
    pub fn discard<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let _ = self.discard_when(|p| Ok(func(p)));
    }

    #[inline]
    pub fn discard_when<F>(&mut self, mut func: F) -> XResult<()>
    where
        F: FnMut(&mut T) -> XResult<bool>,
    {
        let mut idx = 0;
        let mut new_end = 0;

        let mut res: XResult<()> = Ok(());
        self.vec.retain_mut(|item| {
            idx += 1;
            if idx > self.current_end {
                return true;
            }

            match func(item) {
                Ok(true) => false,
                Ok(false) => {
                    new_end += 1;
                    true
                }
                Err(e) => {
                    if res.is_ok() {
                        res = Err(e);
                    }
                    true
                }
            }
        });
        self.current_end = new_end;
        Ok(())
    }

    /// Take a element and return it with a rest vec.
    pub fn taken_rest<'t>(&'t mut self, idx: usize) -> (Option<&'t mut T>, HistoryVecRest<'t, T>) {
        let item_mut = match self.get_mut(idx) {
            Some(item) => {
                let item_ptr = item as *mut T;
                // SAFETY:
                // We split self.vec into two parts. A mutable reference, and the other immutable parts.
                // HistoryVecRest will ensure that mutable references are not accessed.
                // This is similar to slice::split_at_mut, partitioned access to non-overlapping regions.
                unsafe { Some(&mut *item_ptr) }
            }
            None => None,
        };

        let rest = HistoryVecRest {
            vec: &*self,
            taken_idx: idx,
        };

        (item_mut, rest)
    }

    /// Iterate over each element and return it with a rest view.
    #[inline]
    pub fn taken_rest_iter<'t>(&'t mut self) -> HistoryVecTakenRestIter<'t, T> {
        HistoryVecTakenRestIter {
            len: self.len(),
            vec: self,
            current: 0,
        }
    }
}

impl<T> Index<usize> for HistoryVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        return self.get(index).expect("HistoryVec out of index");
    }
}

impl<T> IndexMut<usize> for HistoryVec<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        return self.get_mut(index).expect("HistoryVec out of index");
    }
}

impl<T: fmt::Debug> fmt::Debug for HistoryVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f
            .debug_struct("HistoryVec")
            .field("vec", &self.vec)
            .field("current_end", &self.current_end)
            .finish();
    }
}

pub struct HistoryVecRest<'t, T> {
    vec: &'t HistoryVec<T>,
    taken_idx: usize,
}

impl<'t, T> HistoryVecRest<'t, T> {
    #[inline]
    pub(crate) const unsafe fn new(vec: &'static HistoryVec<T>, taken_idx: usize) -> HistoryVecRest<'static, T> {
        HistoryVecRest { vec, taken_idx }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&'t T> {
        if unlikely(index == self.taken_idx) {
            return None;
        }
        self.vec.get(index)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &'t T> {
        self.index_iter().map(|(_, item)| item)
    }

    #[inline]
    pub fn index_iter(&self) -> impl Iterator<Item = (usize, &'t T)> {
        let taken_idx = self.taken_idx;
        self.vec
            .iter()
            .enumerate()
            .filter_map(move |(idx, item)| if idx == taken_idx { None } else { Some((idx, item)) })
    }
}

impl<T> Index<usize> for HistoryVecRest<'_, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        return self.get(index).expect("HistoryVecRest out of index");
    }
}

pub struct HistoryVecTakenRestIter<'t, T> {
    vec: &'t mut HistoryVec<T>,
    current: usize,
    len: usize,
}

impl<'t, T> ExactSizeIterator for HistoryVecTakenRestIter<'t, T> {}
impl<'t, T> FusedIterator for HistoryVecTakenRestIter<'t, T> {}

impl<'t, T> Iterator for HistoryVecTakenRestIter<'t, T> {
    type Item = (&'t mut T, HistoryVecRest<'t, T>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.len {
            return None;
        }

        let idx = self.current;
        self.current += 1;

        // SAFETY:
        // We split self.vec into two parts. A mutable reference, and the other immutable parts.
        // HistoryVecRest will ensure that mutable references are not accessed.
        // This is similar to slice::split_at_mut, partitioned access to non-overlapping regions.
        unsafe {
            let vec_ptr = self.vec as *mut HistoryVec<T>;
            let item_ptr = (*vec_ptr).get_mut(idx).unwrap() as *mut T;

            let rest = HistoryVecRest {
                vec: &*vec_ptr,
                taken_idx: idx,
            };

            Some((&mut *item_ptr, rest))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.current;
        return (remaining, Some(remaining));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xres;

    #[derive(Debug, PartialEq)]
    struct Payload {
        key: i32,
        value: String,
    }

    impl Payload {
        fn new(key: i32, value: &str) -> Payload {
            Payload {
                key: key,
                value: value.into(),
            }
        }
    }

    #[test]
    fn test_history_vec_empty() {
        let mut hv = HistoryVec::<Payload>::new();
        hv.restore(|_| 1);
        assert_eq!(hv.all_len(), 0);
        hv.discard(|_| false);
        assert_eq!(hv.all_len(), 0);

        assert!(hv.is_empty());
        assert_eq!(hv.len(), 0);
        assert_eq!(hv.all_len(), 0);
        assert_eq!(hv.future_len(), 0);
        assert_eq!(hv.capacity(), 0);
        assert_eq!(hv.get(0), None);
        assert_eq!(hv.get_mut(0), None);
        assert_eq!(hv.iter().count(), 0);
        assert_eq!(hv.iter_mut().count(), 0);
        assert_eq!(hv.append_reuse(|_| Ok(false)).unwrap(), None);
    }

    #[test]
    fn test_history_vec_append() {
        let mut hv = HistoryVec::<Payload>::new();
        hv.append_new(Payload::new(1, "one"));
        hv.append_new(Payload::new(2, "two"));
        hv.append_new(Payload::new(3, "three"));
        let res = hv.append(|_| Ok(false), || Ok(Payload::new(4, "four"))).unwrap();
        assert_eq!(*res, Payload::new(4, "four"));
        assert_eq!(hv.current_end, 4);
        assert_eq!(hv.len(), 4);
        assert_eq!(hv.future_len(), 0);

        assert!(hv.append_reuse(|_| Ok(true)).unwrap().is_none());
        hv.append(|_| Ok(true), || Ok(Payload::new(5, "five"))).unwrap();
        assert_eq!(hv.current_end, 5);
        assert_eq!(hv.len(), 5);
        assert_eq!(hv.future_len(), 0);
    }

    fn new_history_vec() -> HistoryVec<Payload> {
        let mut hv = HistoryVec::<Payload>::new();
        hv.append_new(Payload::new(1, "one"));
        hv.append_new(Payload::new(2, "two"));
        hv.append_new(Payload::new(3, "three"));
        hv.append_new(Payload::new(4, "four"));
        hv.append_new(Payload::new(5, "five"));
        hv
    }

    #[test]
    fn test_history_queue_restore() {
        let mut hv = new_history_vec();
        hv.restore(|p| match p.key {
            1 => -1,
            2 => 0,
            3 => -1,
            4 => 0,
            5 => 1,
            _ => 1,
        });
        assert_eq!(hv.current_end, 4);
        assert_eq!(hv.len(), 4);
        assert_eq!(hv.future_len(), 1);

        hv.restore(|p| p.key - 2);
        assert_eq!(hv.current_end, 2);
        assert_eq!(hv.len(), 2);
        assert_eq!(hv.future_len(), 3);

        hv.append_reuse(|p| {
            if p.key == 3 {
                p.value = "three-three".into();
            }
            Ok(p.key == 3)
        })
        .unwrap();
        assert_eq!(hv.current_end, 3);
        assert_eq!(hv.len(), 3);
        assert_eq!(hv.future_len(), 2);

        hv.append(
            |p| {
                if p.key == 4 {
                    p.value = "four-four".into();
                }
                Ok(p.key == 4)
            },
            || xres!(Unexpected),
        )
        .unwrap();
        assert_eq!(hv.current_end, 4);
        assert_eq!(hv.len(), 4);
        assert_eq!(hv.future_len(), 1);

        hv.append(|_| Ok(false), || Ok(Payload::new(0, "zero"))).unwrap();
        assert_eq!(hv.iter().collect::<Vec<_>>(), vec![
            &Payload::new(1, "one"),
            &Payload::new(2, "two"),
            &Payload::new(3, "three-three"),
            &Payload::new(4, "four-four"),
            &Payload::new(0, "zero"),
        ]);
    }

    #[test]
    fn test_history_vec_discard() {
        let mut hv = new_history_vec();
        hv.restore(|p| p.key - 4);
        hv.discard(|p| p.key % 2 == 0);
        assert_eq!(hv.current_end, 2);
        assert_eq!(hv.len(), 2);
        assert_eq!(hv.future_len(), 1);
        assert_eq!(hv.iter().collect::<Vec<_>>(), vec![
            &Payload::new(1, "one"),
            &Payload::new(3, "three"),
        ]);

        let mut hv = new_history_vec();
        hv.discard(|_| false);
        assert_eq!(hv.current_end, 5);

        let mut hv = new_history_vec();
        hv.discard(|_| true);
        assert_eq!(hv.current_end, 0);
    }

    #[test]
    fn test_history_vec_taken_rest() {
        let mut hv = new_history_vec();
        let (chara_mut, rest) = hv.taken_rest(2);

        // 1. Check mutable element
        let chara = chara_mut.unwrap();
        assert_eq!(chara.key, 3);
        chara.value = "taken".into();

        // 2. Check rest view
        assert_eq!(rest.get(1).unwrap().key, 2);
        assert_eq!(rest.get(2), None); // Taken index
        assert_eq!(rest.get(3).unwrap().key, 4);

        // 3. Check rest iterator
        let keys: Vec<_> = rest.iter().map(|p| p.key).collect();
        assert_eq!(keys, vec![1, 2, 4, 5]);

        // 4. Index trait
        assert_eq!(rest[1].key, 2);
    }

    #[test]
    fn test_history_vec_taken_rest_iter() {
        let mut hv = new_history_vec();
        for (idx, (item, rest)) in hv.taken_rest_iter().enumerate() {
            assert_eq!(item.key, (idx + 1) as i32);
            assert_eq!(rest.iter().count(), 4);
            for r in rest.iter() {
                assert_ne!(r.key, item.key);
            }
        }
    }
}
