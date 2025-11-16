use std::collections::{vec_deque, VecDeque};
use std::fmt;
use std::ops::{Bound, Index, IndexMut, RangeBounds};

use crate::utils::{xres, XResult};

pub struct HistoryQueue<T> {
    queue: VecDeque<T>,
    current_start: u32,
    current_end: u32,
}

impl<T> HistoryQueue<T> {
    #[inline]
    pub fn new() -> HistoryQueue<T> {
        HistoryQueue {
            queue: VecDeque::new(),
            current_start: 0,
            current_end: 0,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> HistoryQueue<T> {
        HistoryQueue {
            queue: VecDeque::with_capacity(capacity),
            current_start: 0,
            current_end: 0,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current_start == self.current_end
    }

    #[inline]
    pub fn len(&self) -> usize {
        (self.current_end - self.current_start) as usize
    }

    #[inline]
    pub fn all_len(&self) -> usize {
        self.queue.len()
    }

    #[inline]
    pub fn past_len(&self) -> usize {
        self.current_start as usize
    }

    #[inline]
    pub fn future_len(&self) -> usize {
        self.queue.len() - self.current_end as usize
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            return Some(&self.queue[(self.current_start as usize) + index]);
        }
        None
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            return Some(&mut self.queue[(self.current_start as usize) + index]);
        }
        None
    }

    #[inline]
    pub fn first(&self) -> Option<&T> {
        if self.len() > 0 {
            return Some(&self.queue[self.current_start as usize]);
        }
        None
    }

    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.len() > 0 {
            return Some(&mut self.queue[self.current_start as usize]);
        }
        None
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.len() > 0 {
            return Some(&self.queue[(self.current_end as usize) - 1]);
        }
        None
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len() > 0 {
            return Some(&mut self.queue[(self.current_end as usize) - 1]);
        }
        None
    }

    // return (len-2, len-1)
    pub fn last2(&mut self) -> (Option<&T>, Option<&T>) {
        if self.len() >= 2 {
            let current_end = self.current_end as usize;
            (Some(&self.queue[current_end - 2]), Some(&self.queue[current_end - 1]))
        }
        else if self.len() == 1 {
            (None, Some(&mut self.queue[(self.current_end as usize) - 1]))
        }
        else {
            (None, None)
        }
    }

    // return (len-2, len-1)
    pub fn last2_mut(&mut self) -> (Option<&mut T>, Option<&mut T>) {
        if self.len() >= 2 {
            let current_end = self.current_end as usize;
            let mut range = self.queue.range_mut(current_end - 2..current_end);
            (range.next(), range.next())
        }
        else if self.len() == 1 {
            (None, Some(&mut self.queue[(self.current_end as usize) - 1]))
        }
        else {
            (None, None)
        }
    }

    #[inline]
    pub fn iter(&self) -> vec_deque::Iter<'_, T> {
        return self.queue.range(self.current_start as usize..self.current_end as usize);
    }

    #[inline]
    pub fn iter_mut(&mut self) -> vec_deque::IterMut<'_, T> {
        return self
            .queue
            .range_mut(self.current_start as usize..self.current_end as usize);
    }

    #[inline]
    pub fn range<R>(&mut self, range: R) -> vec_deque::Iter<'_, T>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(idx) => self.current_start as usize + *idx,
            Bound::Excluded(idx) => self.current_start as usize + *idx + 1,
            Bound::Unbounded => self.current_start as usize,
        };
        let end = match range.end_bound() {
            Bound::Included(idx) => self.current_start as usize + *idx + 1,
            Bound::Excluded(idx) => self.current_start as usize + *idx,
            Bound::Unbounded => self.current_end as usize,
        };
        self.queue.range(start..end)
    }

    #[inline]
    pub fn range_mut<R>(&mut self, range: R) -> vec_deque::IterMut<'_, T>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(idx) => self.current_start as usize + *idx,
            Bound::Excluded(idx) => self.current_start as usize + *idx + 1,
            Bound::Unbounded => self.current_start as usize,
        };
        let end = match range.end_bound() {
            Bound::Included(idx) => self.current_start as usize + *idx + 1,
            Bound::Excluded(idx) => self.current_start as usize + *idx,
            Bound::Unbounded => self.current_end as usize,
        };
        self.queue.range_mut(start..end)
    }

    #[inline]
    pub fn enqueue_reuse<R>(&mut self, reuse: R) -> XResult<Option<&mut T>>
    where
        R: FnOnce(&mut T) -> XResult<bool>,
    {
        let end = self.current_end as usize;
        if end < self.queue.len() && reuse(&mut self.queue[end])? {
            self.current_end += 1;
            return Ok(Some(&mut self.queue[end]));
        }
        Ok(None)
    }

    #[inline]
    pub fn enqueue_new(&mut self, value: T) {
        let end = self.current_end as usize;
        while end < self.queue.len() {
            self.queue.pop_back();
        }
        self.current_end += 1;
        self.queue.push_back(value);
    }

    #[inline]
    pub fn enqueue<R, N>(&mut self, reuse: R, new: N) -> XResult<&mut T>
    where
        R: FnOnce(&mut T) -> XResult<bool>,
        N: FnOnce() -> XResult<T>,
    {
        return self.enqueue_with(&mut (), |_, r| reuse(r), |_| new());
    }

    #[inline]
    pub fn enqueue_with<R, N, C>(&mut self, ctx: &mut C, reuse: R, new: N) -> XResult<&mut T>
    where
        R: FnOnce(&mut C, &mut T) -> XResult<bool>,
        N: FnOnce(&mut C) -> XResult<T>,
    {
        let end = self.current_end as usize;
        if end < self.queue.len() && reuse(ctx, &mut self.queue[end])? {
            self.current_end += 1;
            return Ok(&mut self.queue[end]);
        }

        while end < self.queue.len() {
            self.queue.pop_back();
        }
        self.current_end += 1;
        self.queue.push_back(new(ctx)?);
        Ok(&mut self.queue[end])
    }

    // func returns:
    // - true to dequeue the element
    // - false to stop dequeuing
    #[inline]
    pub fn dequeue<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let _ = self.dequeue_when(|p| Ok(func(p)));
    }

    #[inline]
    pub fn dequeue_when<F>(&mut self, mut func: F) -> XResult<()>
    where
        F: FnMut(&mut T) -> XResult<bool>,
    {
        let mut new_start = self.current_start;
        for idx in self.current_start..self.current_end {
            if func(&mut self.queue[idx as usize])? {
                new_start += 1;
            }
            else {
                break;
            }
        }
        self.current_start = new_start;
        Ok(())
    }

    // func returns:
    // - 0 to restore the element
    // - -1 to skip the element
    // - 1 to stop restoring
    #[inline]
    pub fn restore<F>(&mut self, mut func: F) -> XResult<()>
    where
        F: FnMut(&mut T) -> i32,
    {
        self.restore_when(|p| Ok(func(p)))
    }

    #[inline]
    pub fn restore_when<F>(&mut self, mut func: F) -> XResult<()>
    where
        F: FnMut(&mut T) -> XResult<i32>,
    {
        let mut new_start = 0;
        let mut new_end = 0;
        while new_start < self.current_start {
            let res = func(&mut self.queue[new_start as usize])?;
            new_end += 1;
            if res < 0 {
                new_start += 1;
            }
            else if res == 0 {
                break;
            }
            else {
                return xres!(BadOperation; "start");
            }
        }
        while new_end < self.current_end {
            let res = func(&mut self.queue[new_end as usize])?;
            if res == 0 {
                new_end += 1;
            }
            else if res > 0 {
                break;
            }
            else {
                return xres!(BadOperation; "end");
            }
        }
        self.current_start = new_start;
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
        let mut count = 0;
        for _ in 0..self.current_start {
            if func(&mut self.queue[0])? {
                count += 1;
                self.queue.pop_front();
            }
            else {
                break;
            }
        }
        self.current_start -= count;
        self.current_end -= count;
        Ok(())
    }
}

impl<T> Default for HistoryQueue<T> {
    #[inline]
    fn default() -> Self {
        HistoryQueue::new()
    }
}

impl<T> Index<usize> for HistoryQueue<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        return self.get(index).unwrap();
    }
}

impl<T> IndexMut<usize> for HistoryQueue<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        return self.get_mut(index).unwrap();
    }
}

impl<T: fmt::Debug> fmt::Debug for HistoryQueue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f
            .debug_struct("HistoryQueue")
            .field("queue", &self.queue)
            .field("start", &self.current_start)
            .field("end", &self.current_end)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_history_queue_empty() {
        let mut hq = HistoryQueue::<Payload>::new();
        hq.dequeue(|_| false);
        assert_eq!(hq.all_len(), 0);
        hq.restore(|_| 1).unwrap();
        assert_eq!(hq.all_len(), 0);
        hq.discard(|_| false);
        assert_eq!(hq.all_len(), 0);

        assert!(hq.is_empty());
        assert_eq!(hq.len(), 0);
        assert_eq!(hq.all_len(), 0);
        assert_eq!(hq.past_len(), 0);
        assert_eq!(hq.future_len(), 0);
        assert_eq!(hq.capacity(), 0);
        assert_eq!(hq.get(0), None);
        assert_eq!(hq.get_mut(0), None);
        assert_eq!(hq.iter().count(), 0);
        assert_eq!(hq.iter_mut().count(), 0);
        assert_eq!(hq.enqueue_reuse(|_| Ok(false)).unwrap(), None);
    }

    #[test]
    fn test_history_queue_enqueue_dequeue() {
        let mut hq = HistoryQueue::<Payload>::new();
        hq.enqueue_new(Payload::new(1, "one"));
        hq.enqueue_new(Payload::new(2, "two"));
        hq.enqueue_new(Payload::new(3, "three"));
        let res = hq.enqueue(|_| Ok(false), || Ok(Payload::new(4, "four"))).unwrap();
        assert_eq!(*res, Payload::new(4, "four"));
        assert_eq!(hq.current_start, 0);
        assert_eq!(hq.current_end, 4);
        assert_eq!(hq.len(), 4);
        assert_eq!(hq.past_len(), 0);
        assert_eq!(hq.future_len(), 0);

        hq.dequeue(|p| p.key <= 2);
        assert_eq!(hq.current_start, 2);
        assert_eq!(hq.current_end, 4);
        assert_eq!(hq.len(), 2);
        assert_eq!(hq.past_len(), 2);

        hq.dequeue(|p| p.key == 3);
        assert_eq!(hq.current_start, 3);
        assert_eq!(hq.current_end, 4);
        assert_eq!(hq.len(), 1);
        assert_eq!(hq.past_len(), 3);

        assert!(hq.enqueue_reuse(|_| Ok(true)).unwrap().is_none());
        hq.enqueue(|_| Ok(true), || Ok(Payload::new(5, "five"))).unwrap();
        assert_eq!(hq.current_start, 3);
        assert_eq!(hq.current_end, 5);
        assert_eq!(hq.len(), 2);
        assert_eq!(hq.past_len(), 3);
        assert_eq!(hq.future_len(), 0);
    }

    fn new_history_queue() -> HistoryQueue<Payload> {
        let mut hq = HistoryQueue::<Payload>::new();
        hq.enqueue_new(Payload::new(1, "one"));
        hq.enqueue_new(Payload::new(2, "two"));
        hq.enqueue_new(Payload::new(3, "three"));
        hq.enqueue_new(Payload::new(4, "four"));
        hq.enqueue_new(Payload::new(5, "five"));
        hq.enqueue_new(Payload::new(6, "six"));
        hq
    }

    #[test]
    fn test_history_queue_restore() {
        let mut hq = new_history_queue();
        hq.dequeue(|p| p.key <= 3);
        hq.restore(|p| match p.key {
            1 => -1,
            2 => {
                p.value = "two-two".into();
                0
            }
            3 => {
                p.value = "three-three".into();
                0
            }
            _ => 1,
        })
        .unwrap();
        assert_eq!(hq.current_start, 1);
        assert_eq!(hq.current_end, 3);
        assert_eq!(hq.len(), 2);
        assert_eq!(hq.past_len(), 1);
        assert_eq!(hq.future_len(), 3);

        hq.enqueue_reuse(|p| {
            if p.key == 4 {
                p.value = "four-four".into();
            }
            Ok(p.key == 4)
        })
        .unwrap();
        assert_eq!(hq[2], Payload::new(4, "four-four"));
        assert_eq!(hq.future_len(), 2);

        hq.enqueue(
            |p| {
                if p.key == 5 {
                    p.value = "five-five".into();
                }
                Ok(p.key == 5)
            },
            || xres!(BadOperation),
        )
        .unwrap();
        assert_eq!(hq[3], Payload::new(5, "five-five"));
        assert_eq!(hq.future_len(), 1);

        hq.enqueue(|_| Ok(false), || Ok(Payload::new(0, "zero"))).unwrap();
        assert_eq!(hq.iter().collect::<Vec<_>>(), vec![
            &Payload::new(2, "two-two"),
            &Payload::new(3, "three-three"),
            &Payload::new(4, "four-four"),
            &Payload::new(5, "five-five"),
            &Payload::new(0, "zero"),
        ]);
        assert_eq!(hq.past_len(), 1);
        assert_eq!(hq.future_len(), 0);

        let mut hq = new_history_queue();
        assert!(hq.restore(|_| -1).is_err());
        hq.dequeue(|p| p.key == 1);
        assert!(hq.restore(|_| 1).is_err());
    }

    #[test]
    fn test_history_queue_discard() {
        let mut hq = new_history_queue();
        hq.dequeue(|p| p.key <= 3);
        hq.discard(|p| p.key <= 3);
        assert_eq!(hq.current_start, 0);
        assert_eq!(hq.current_end, 3);
        assert_eq!(hq.iter_mut().collect::<Vec<_>>(), vec![
            &mut Payload::new(4, "four"),
            &mut Payload::new(5, "five"),
            &mut Payload::new(6, "six")
        ]);

        let mut hq = new_history_queue();
        hq.dequeue(|p| p.key <= 4);
        hq.discard(|p| p.key <= 2);
        assert_eq!(hq.current_start, 2);
        assert_eq!(hq.current_end, 4);
        hq.restore(|_| 0).unwrap();
        assert_eq!(hq.current_start, 0);
        assert_eq!(hq.current_end, 4);
        assert_eq!(hq.iter().collect::<Vec<_>>(), vec![
            &Payload::new(3, "three"),
            &Payload::new(4, "four"),
            &Payload::new(5, "five"),
            &Payload::new(6, "six"),
        ]);
    }
}
