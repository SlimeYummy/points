use critical_point_csgen::CsOut;

use crate::utils::macros::{rkyv_self, serde_by};

//
// TimeRange
//

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, CsOut)]
pub struct TimeRange {
    pub begin: f32,
    pub end: f32,
}

rkyv_self!(TimeRange);
serde_by!(TimeRange, (f32, f32), TimeRange::from, TimeRange::to_tuple);

impl TimeRange {
    #[inline]
    pub fn new(begin: f32, end: f32) -> TimeRange {
        TimeRange { begin, end }
    }

    #[inline]
    pub fn duration(&self) -> f32 {
        self.end - self.begin
    }

    #[inline]
    pub fn to_array(&self) -> [f32; 2] {
        [self.begin, self.end]
    }

    #[inline]
    pub fn to_tuple(&self) -> (f32, f32) {
        (self.begin, self.end)
    }

    #[inline]
    pub fn step(&self, step: f32) -> TimeRangeStep {
        TimeRangeStep {
            range: *self,
            step,
            current: self.begin,
        }
    }

    #[inline]
    pub fn contains(&self, time: f32) -> bool {
        self.begin <= time && time <= self.end
    }

    #[inline]
    pub fn contains_no_left(&self, time: f32) -> bool {
        self.begin < time && time <= self.end
    }

    #[inline]
    pub fn contains_no_right(&self, time: f32) -> bool {
        self.begin <= time && time < self.end
    }
}

impl From<(f32, f32)> for TimeRange {
    #[inline]
    fn from((begin, end): (f32, f32)) -> Self {
        TimeRange { begin, end }
    }
}

impl From<TimeRange> for (f32, f32) {
    #[inline]
    fn from(val: TimeRange) -> Self {
        val.to_tuple()
    }
}

impl From<[f32; 2]> for TimeRange {
    #[inline]
    fn from([begin, end]: [f32; 2]) -> Self {
        TimeRange { begin, end }
    }
}

impl From<TimeRange> for [f32; 2] {
    #[inline]
    fn from(val: TimeRange) -> Self {
        val.to_array()
    }
}

#[derive(Debug)]
pub struct TimeRangeStep {
    range: TimeRange,
    step: f32,
    current: f32,
}

impl Iterator for TimeRangeStep {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.range.end {
            let value = self.current;
            self.current += self.step;
            Some(value)
        }
        else {
            None
        }
    }
}

//
// TimeFragment
//

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimeFragment {
    pub begin: f32,
    pub end: f32,
    pub index: u32,
}

rkyv_self!(TimeFragment);

impl TimeFragment {
    #[inline]
    pub fn new(begin: f32, end: f32, index: u32) -> TimeFragment {
        TimeFragment { begin, end, index }
    }

    #[inline]
    pub fn duration(&self) -> f32 {
        self.end - self.begin
    }

    #[inline]
    pub fn to_time_range(&self) -> TimeRange {
        TimeRange {
            begin: self.begin,
            end: self.end,
        }
    }

    #[inline]
    pub fn to_array(&self) -> [f32; 2] {
        [self.begin, self.end]
    }

    #[inline]
    pub fn to_tuple(&self) -> (f32, f32) {
        (self.begin, self.end)
    }
}

impl From<TimeFragment> for TimeRange {
    #[inline]
    fn from(val: TimeFragment) -> Self {
        val.to_time_range()
    }
}

impl From<TimeFragment> for (f32, f32) {
    #[inline]
    fn from(val: TimeFragment) -> Self {
        val.to_tuple()
    }
}

impl From<TimeFragment> for [f32; 2] {
    #[inline]
    fn from(val: TimeFragment) -> Self {
        val.to_array()
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct TimeWith<T> {
    pub time: f32,
    pub value: T,
}

impl<T> TimeWith<T> {
    #[inline]
    pub fn new(time: f32, value: T) -> TimeWith<T> {
        TimeWith { time, value }
    }
}

impl<T> From<(f32, T)> for TimeWith<T> {
    #[inline]
    fn from(item: (f32, T)) -> Self {
        TimeWith::new(item.0, item.1)
    }
}

impl<T> From<TimeWith<T>> for (f32, T) {
    #[inline]
    fn from(item: TimeWith<T>) -> Self {
        (item.time, item.value)
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct TimeRangeWith<T> {
    pub range: TimeRange,
    pub value: T,
}

impl<T> TimeRangeWith<T> {
    #[inline]
    pub fn new(range: TimeRange, value: T) -> TimeRangeWith<T> {
        TimeRangeWith { range, value }
    }
}

impl<T> From<(TimeRange, T)> for TimeRangeWith<T> {
    #[inline]
    fn from(item: (TimeRange, T)) -> Self {
        TimeRangeWith::new(item.0, item.1)
    }
}

impl<T> From<TimeRangeWith<T>> for (TimeRange, T) {
    #[inline]
    fn from(item: TimeRangeWith<T>) -> Self {
        (item.range, item.value)
    }
}
