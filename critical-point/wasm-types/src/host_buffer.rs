use std::{cmp, fmt, slice};

pub struct HostBuffer<'a, T: Copy> {
    inner: &'a mut [T],
    cur_len: usize,
}

impl<'a, T: Copy> HostBuffer<'a, T> {
    #[inline]
    pub unsafe fn new(addr: *mut T, len: u32) -> Self {
        Self {
            inner: unsafe { slice::from_raw_parts_mut(addr, len as usize) },
            cur_len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cur_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cur_len == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.inner.len() - self.cur_len
    }

    #[inline]
    pub fn push(&mut self, item: T) -> bool {
        if self.cur_len < self.inner.len() {
            self.inner[self.cur_len] = item;
            self.cur_len += 1;
            true
        }
        else {
            false
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.cur_len > 0 {
            self.cur_len -= 1;
            Some(self.inner[self.cur_len])
        }
        else {
            None
        }
    }

    #[inline]
    pub fn insert(&mut self, index: usize, item: T) -> bool {
        if self.cur_len < self.inner.len() && index <= self.cur_len {
            if index < self.cur_len {
                self.inner.copy_within(index..self.cur_len, index + 1);
            }
            self.inner[index] = item;
            self.cur_len += 1;
            true
        }
        else {
            false
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.cur_len {
            let item = self.inner[index];
            if index < self.cur_len - 1 {
                self.inner.copy_within(index + 1..self.cur_len, index);
            }
            self.cur_len -= 1;
            Some(item)
        }
        else {
            None
        }
    }

    #[inline]
    pub fn extend(&mut self, src: &[T]) -> usize {
        let amt = cmp::min(src.len(), self.remaining());
        if amt > 0 {
            self.inner[self.cur_len..self.cur_len + amt].copy_from_slice(&src[..amt]);
            self.cur_len += amt;
        }
        amt
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.inner[..self.cur_len]
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.inner[..self.cur_len]
    }
}

impl<'a, T: Copy + fmt::Debug> fmt::Debug for HostBuffer<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostBuffer")
            .field("cur_len", &self.cur_len)
            .field("capacity", &self.inner.len())
            .field("written", &self.as_slice())
            .finish()
    }
}
