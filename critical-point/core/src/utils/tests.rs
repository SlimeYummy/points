use std::ops::Range;
use std::path::PathBuf;
use std::{env, fs};

use crate::consts::{FPS, SPF};

pub(crate) fn prepare_tmp_dir(name: &str) -> PathBuf {
    let mut dir = env::current_dir().unwrap();
    dir.pop();
    dir.pop();
    dir.push("test-tmp");
    dir.push(name);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::create_dir(&dir);
    dir
}

pub(crate) fn write_json<T: ?Sized + serde::Serialize>(path: &PathBuf, data: &T) {
    let buf = serde_json::to_vec(data).unwrap();
    fs::write(&path, buf).unwrap();
}

pub(crate) fn write_rkyv<T>(path: &PathBuf, data: &T)
where
    T: for<'a> rkyv::Serialize<
        rkyv::api::high::HighSerializer<
            rkyv::util::AlignedVec,
            rkyv::ser::allocator::ArenaHandle<'a>,
            rkyv::rancor::Error,
        >,
    >,
{
    let buf = rkyv::to_bytes(data).unwrap();
    fs::write(&path, buf).unwrap();
}

#[derive(Debug)]
pub(crate) struct FrameTicker<I: Iterator<Item = u32>> {
    iter: I,
    idx: usize,
    frame: Option<u32>,
    time: f32,
}

impl<I: Iterator<Item = u32>> FrameTicker<I> {
    pub fn new(iter: I) -> FrameTicker<I> {
        let mut ticker = FrameTicker {
            iter,
            idx: 0,
            frame: None,
            time: 0.0,
        };
        ticker.frame = ticker.iter.next();
        ticker.time = ticker.frame.map(|x| x as f32 * SPF).unwrap_or(f32::NAN);
        ticker
    }
}

impl<I: Iterator<Item = u32>> Iterator for FrameTicker<I> {
    type Item = FrameTick;

    fn next(&mut self) -> Option<Self::Item> {
        let a = 0..3;
        if let Some(frame) = self.frame {
            let mut tick = FrameTick {
                idx: self.idx,
                frame,
                time: self.time,
                first: self.idx == 0,
                last: false,
            };
            self.idx += 1;
            self.frame = self.iter.next();
            self.time += SPF;
            tick.last = self.frame.is_none();
            Some(tick)
        } else {
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct FrameTick {
    pub idx: usize,
    pub frame: u32,
    pub time: f32,
    pub first: bool,
    pub last: bool,
}

#[allow(dead_code)]
impl FrameTick {
    pub(crate) fn time(&self, frame: i32) -> f32 {
        let mut time = self.time;
        if frame > 0 {
            for _ in 0..frame {
                time += SPF;
            }
        } else {
            for _ in frame..0 {
                time -= SPF;
            }
        }
        time
    }

    pub(crate) fn first_or<T>(&self, first: T, other: T) -> T {
        match self.first {
            true => first,
            false => other,
        }
    }

    pub(crate) fn or_first<T>(&self, other: T, first: T) -> T {
        match self.first {
            true => first,
            false => other,
        }
    }

    pub(crate) fn last_or<T>(&self, last: T, other: T) -> T {
        match self.last {
            true => last,
            false => other,
        }
    }

    pub(crate) fn or_last<T>(&self, other: T, last: T) -> T {
        match self.last {
            true => last,
            false => other,
        }
    }
}
