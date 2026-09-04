use std::hint::likely;

use crate::consts::{CFG_FPS, FPS};

#[inline(always)]
pub const fn f2s(frame: u32) -> f32 {
    frame as f32 / FPS
}

#[inline(always)]
pub const fn ff2s(frame: f32) -> f32 {
    frame / FPS
}

#[inline(always)]
pub const fn cf2s(frame: u32) -> f32 {
    frame as f32 / CFG_FPS
}

#[inline(always)]
pub const fn cff2s(frame: f32) -> f32 {
    frame / CFG_FPS
}

#[inline(always)]
pub const fn s2f(second: f32) -> u32 {
    (second * FPS).round() as u32
}

#[inline(always)]
pub const fn s2ff(second: f32) -> f32 {
    (second * FPS).round()
}

#[inline(always)]
pub const fn s2f_round(second: f32) -> u32 {
    (second * FPS).round() as u32
}

#[inline(always)]
pub const fn s2ff_round(second: f32) -> f32 {
    (second * FPS).round()
}

#[inline(always)]
pub const fn s2f_floor(second: f32) -> u32 {
    (second * FPS).floor() as u32
}

#[inline(always)]
pub const fn s2ff_floor(second: f32) -> f32 {
    (second * FPS).floor()
}

#[inline(always)]
pub const fn s2f_ceil(second: f32) -> u32 {
    (second * FPS).ceil() as u32
}

#[inline(always)]
pub const fn s2ff_ceil(second: f32) -> f32 {
    (second * FPS).ceil()
}

/// a (- eps) <= b
#[macro_export]
macro_rules! loose_le {
    ($a:expr, $b:expr) => {
        loose_le!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a - $eps <= $b
    };
}
pub use loose_le;

/// a (+ eps) < b
#[macro_export]
macro_rules! strict_lt {
    ($a:expr, $b:expr) => {
        strict_lt!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a + $eps < $b
    };
}
pub use strict_lt;

/// a (+ eps) >= b
#[macro_export]
macro_rules! loose_ge {
    ($a:expr, $b:expr) => {
        loose_ge!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a + $eps >= $b
    };
}
pub use loose_ge;

/// a (- eps) > b
#[macro_export]
macro_rules! strict_gt {
    ($a:expr, $b:expr) => {
        strict_gt!($a, $b, 1e-4)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a - $eps > $b
    };
}
pub use strict_gt;

#[inline(always)]
pub const fn square(x: f32) -> f32 {
    x * x
}

#[inline(always)]
pub const fn cube(x: f32) -> f32 {
    x * x * x
}

/// Require b > 0
#[inline]
pub const fn ratio_saturating(a: f32, b: f32) -> f32 {
    let bb = b.abs();
    if likely(a > 0.0) { (a / bb).min(1.0) } else { 0.0 }
}

/// Require b > 0
#[inline]
pub const fn ratio_warpping(a: f32, b: f32) -> f32 {
    let bb = b.abs();
    let r = (a % bb) / bb;
    if likely(r >= 0.0) {
        r
    }
    else if likely(r < 0.0) {
        r + 1.0
    }
    else {
        0.0 // NaN/Inf
    }
}

#[inline(always)]
pub const fn calc_fade_in(prev_weight: f32, time_step: f32, duration: f32) -> f32 {
    (prev_weight + time_step / duration).min(1.0)
}

// #[inline]
// pub const fn normalize_radian(rad: f32) -> f32 {
//     let mut norm = rad % (2.0 * PI);
//     if norm > PI {
//         norm -= 2.0 * PI;
//     } else if norm < -PI {
//         norm += 2.0 * PI;
//     }
//     norm
// }

// #[inline]
// pub const fn min_radian_diff(a: f32, b: f32) -> f32 {
//     let diff = (a - b).abs() % (2.0 * PI);
//     if diff > PI {
//         2.0 * PI - diff
//     } else {
//         diff
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::{INFINITY, NAN};

    #[test]
    fn test_ratio_saturating() {
        assert_eq!(ratio_saturating(-0.5, 2.0), 0.0);
        assert_eq!(ratio_saturating(0.0, 2.0), 0.0);
        assert_eq!(ratio_saturating(1.5, 2.0), 0.75);
        assert_eq!(ratio_saturating(2.5, 2.0), 1.0);

        assert_eq!(ratio_saturating(-0.5, -2.0), 0.0);
        assert_eq!(ratio_saturating(0.0, -2.0), 0.0);
        assert_eq!(ratio_saturating(1.5, -2.0), 0.75);
        assert_eq!(ratio_saturating(2.5, -2.0), 1.0);

        assert_eq!(ratio_saturating(-INFINITY, 2.0), 0.0);
        assert_eq!(ratio_saturating(INFINITY, 2.0), 1.0);
        assert_eq!(ratio_saturating(NAN, 2.0), 0.0);

        assert_eq!(ratio_saturating(5.0, 0.0), 1.0);
        assert_eq!(ratio_saturating(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_ratio_wrapping() {
        assert_eq!(ratio_warpping(-2.5, 2.0), 0.75);
        assert_eq!(ratio_warpping(-1.5, 2.0), 0.25);
        assert_eq!(ratio_warpping(0.0, 2.0), 0.0);
        assert_eq!(ratio_warpping(0.5, 2.0), 0.25);
        assert_eq!(ratio_warpping(2.5, 2.0), 0.25);
        assert_eq!(ratio_warpping(4.5, 2.0), 0.25);

        assert_eq!(ratio_warpping(-2.5, -2.0), 0.75);
        assert_eq!(ratio_warpping(0.5, -2.0), 0.25);
        assert_eq!(ratio_warpping(4.5, -2.0), 0.25);

        assert_eq!(ratio_saturating(-INFINITY, 2.0), 0.0);
        assert_eq!(ratio_saturating(INFINITY, 2.0), 1.0);
        assert_eq!(ratio_saturating(NAN, 2.0), 0.0);

        assert_eq!(ratio_saturating(5.0, 0.0), 1.0);
        assert_eq!(ratio_saturating(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_loose_strict_compare() {
        assert_eq!(loose_le!(1.0 + 1e-6, 1.0), true);
        assert_eq!(loose_le!(1.0 + 1e-3, 1.0), false);
        assert_eq!(strict_lt!(1.0 - 1e-3, 1.0), true);
        assert_eq!(strict_lt!(1.0 - 1e-6, 1.0), false);

        assert_eq!(loose_ge!(1.0 - 1e-6, 1.0), true);
        assert_eq!(loose_ge!(1.0 - 1e-3, 1.0), false);
        assert_eq!(strict_gt!(1.0 + 1e-3, 1.0), true);
        assert_eq!(strict_gt!(1.0 + 1e-6, 1.0), false);
    }
}
