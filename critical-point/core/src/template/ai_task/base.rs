use crate::utils::{XResult, xres};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TmplRepeatLimit {
    pub times: u16,
    pub window: u16,
}

impl Default for TmplRepeatLimit {
    fn default() -> Self {
        Self::NO_LIMIT
    }
}

impl TmplRepeatLimit {
    pub const MAX_WINDOW: u16 = 10;

    pub const ZERO: TmplRepeatLimit = TmplRepeatLimit { times: 0, window: 1 };
    pub const NO_LIMIT: TmplRepeatLimit = TmplRepeatLimit { times: 1, window: 1 };

    #[inline]
    pub fn new(times: u16, window: u16) -> XResult<TmplRepeatLimit> {
        let rt = TmplRepeatLimit { times, window };
        if !rt.is_valid() {
            return xres!(BadArgument; "invalid repeat times");
        }
        Ok(rt)
    }

    #[inline]
    pub fn new_valid(times: u16, window: u16) -> TmplRepeatLimit {
        TmplRepeatLimit { times, window }.make_valid()
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        let window_ok = self.window > 0 && self.window <= Self::MAX_WINDOW;
        let times_ok = self.times <= Self::MAX_WINDOW;
        window_ok && times_ok
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    pub fn make_valid(&self) -> TmplRepeatLimit {
        if self.window == 0 {
            return Self::NO_LIMIT;
        }

        let ratio = self.times as f32 / self.window as f32;
        if ratio >= 1.0 {
            return Self::NO_LIMIT;
        }

        let window = self.window.clamp(1, Self::MAX_WINDOW);
        let times = (window as f32 * ratio).round() as u16;
        TmplRepeatLimit { times, window }
    }

    #[inline]
    pub fn from_rkyv(r: &ArchivedTmplRepeatLimit) -> TmplRepeatLimit {
        TmplRepeatLimit {
            times: r.times.to_native(),
            window: r.window.to_native(),
        }
    }
}
