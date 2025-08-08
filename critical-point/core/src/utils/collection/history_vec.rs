use std::ops::{Index, IndexMut};
use std::{fmt, slice};

use crate::utils::XResult;

pub struct HistoryVec<T> {
    vec: Vec<T>,
    current_end: u32,
}

impl<T> Default for HistoryVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> HistoryVec<T> {
    #[inline]
    pub fn new() -> HistoryVec<T> {
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
        self.current_end as usize
    }

    #[inline]
    pub fn all_len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn future_len(&self) -> usize {
        self.vec.len() - self.current_end as usize
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < (self.current_end as usize) {
            return Some(&self.vec[index]);
        }
        None
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < (self.current_end as usize) {
            return Some(&mut self.vec[index]);
        }
        None
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        return self.vec[..self.current_end as usize].iter();
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        return self.vec[..self.current_end as usize].iter_mut();
    }

    #[inline]
    pub fn iter_by<F>(&self, func: F) -> HistoryVecIter<'_, T, F>
    where
        F: Fn(&T) -> bool,
    {
        HistoryVecIter {
            slice: Some(&self.vec[..self.current_end as usize]),
            func,
        }
    }

    #[inline]
    pub fn iter_mut_by<F>(&mut self, func: F) -> HistoryVecIterMut<'_, T, F>
    where
        F: Fn(&T) -> bool,
    {
        HistoryVecIterMut {
            slice: Some(&mut self.vec[..self.current_end as usize]),
            func,
        }
    }

    #[inline]
    pub fn append_reuse<R>(&mut self, reuse: R) -> XResult<Option<&mut T>>
    where
        R: FnOnce(&mut T) -> XResult<bool>,
    {
        let end = self.current_end as usize;
        if end < self.vec.len() && reuse(&mut self.vec[end])? {
            self.current_end += 1;
            return Ok(Some(&mut self.vec[end]));
        }
        Ok(None)
    }

    #[inline]
    pub fn append_new(&mut self, item: T) {
        let end = self.current_end as usize;
        while end < self.vec.len() {
            self.vec.pop();
        }
        self.current_end += 1;
        self.vec.push(item);
    }

    #[inline]
    pub fn append<R, N>(&mut self, reuse: R, new: N) -> XResult<&mut T>
    where
        R: FnOnce(&mut T) -> XResult<bool>,
        N: FnOnce() -> XResult<T>,
    {
        let end = self.current_end as usize;
        if end < self.vec.len() && reuse(&mut self.vec[end])? {
            self.current_end += 1;
            return Ok(&mut self.vec[end]);
        }

        while end < self.vec.len() {
            self.vec.pop();
        }
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
        for idx in 0..(self.current_end as usize) {
            let res = func(&mut self.vec[idx])?;
            if res < 0 {
                new_end += 1;
                continue;
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
        let mut new_end: u32 = 0;
        for idx in 0..(self.current_end as usize) {
            if !func(&mut self.vec[idx])? {
                self.vec.swap(new_end as usize, idx);
                new_end += 1;
            }
        }
        self.vec.drain((new_end as usize)..(self.current_end as usize));
        self.current_end = new_end;
        Ok(())
    }
}

impl<T> Index<usize> for HistoryVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        return self.get(index).unwrap();
    }
}

impl<T> IndexMut<usize> for HistoryVec<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        return self.get_mut(index).unwrap();
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

pub struct HistoryVecIter<'t, T, F> {
    slice: Option<&'t [T]>,
    func: F,
}

impl<'t, T, F> Iterator for HistoryVecIter<'t, T, F>
where
    F: Fn(&T) -> bool,
{
    type Item = &'t T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.slice.take() {
                Some([]) | None => return None,
                Some([item, rest @ ..]) => {
                    self.slice = Some(rest);
                    if (self.func)(item) {
                        return Some(item);
                    }
                    else {
                        continue;
                    }
                }
            }
        }
    }
}

pub struct HistoryVecIterMut<'t, T, F> {
    slice: Option<&'t mut [T]>,
    func: F,
}

impl<'t, T, F> Iterator for HistoryVecIterMut<'t, T, F>
where
    F: Fn(&T) -> bool,
{
    type Item = &'t mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.slice.take() {
                Some([]) | None => return None,
                Some([item, rest @ ..]) => {
                    self.slice = Some(rest);
                    if (self.func)(item) {
                        return Some(item);
                    }
                    else {
                        continue;
                    }
                }
            }
        }
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
}
