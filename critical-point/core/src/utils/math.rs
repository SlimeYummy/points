use glam::{Quat, Vec3A, Vec4};
use glam_ext::Vec2xz;
use std::hint::likely;

use crate::consts::{CFG_FPS, DEFAULT_TOWARD_DIR_2D, DEFAULT_TOWARD_DIR_3D, FPS};

#[inline(always)]
pub fn f2s(frame: u32) -> f32 {
    frame as f32 / FPS
}

#[inline(always)]
pub fn ff2s(frame: f32) -> f32 {
    frame / FPS
}

#[inline(always)]
pub fn cf2s(frame: u32) -> f32 {
    frame as f32 / CFG_FPS
}

#[inline(always)]
pub fn cff2s(frame: f32) -> f32 {
    frame / CFG_FPS
}

#[inline(always)]
pub fn s2f(second: f32) -> u32 {
    (second * FPS).round() as u32
}

#[inline(always)]
pub fn s2ff(second: f32) -> f32 {
    (second * FPS).round()
}

#[inline(always)]
pub fn s2f_round(second: f32) -> u32 {
    (second * FPS).round() as u32
}

#[inline(always)]
pub fn s2ff_round(second: f32) -> f32 {
    (second * FPS).round()
}

#[inline(always)]
pub fn s2f_floor(second: f32) -> u32 {
    (second * FPS).floor() as u32
}

#[inline(always)]
pub fn s2ff_floor(second: f32) -> f32 {
    (second * FPS).floor()
}

#[inline(always)]
pub fn s2f_ceil(second: f32) -> u32 {
    (second * FPS).ceil() as u32
}

#[inline(always)]
pub fn s2ff_ceil(second: f32) -> f32 {
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

#[macro_export]
macro_rules! lerp {
    ($a:expr, $b:expr, $t:expr) => {
        $a + ($b - $a) * $t
    };
}
pub use lerp;

#[macro_export]
macro_rules! ratio_safe {
    ($a:expr, $b:expr) => {{
        let aa = $a as f32;
        let bb = ($b as f32);
        if bb > 0.0 {
            aa / bb
        }
        else {
            0.0
        }
    }};
}
pub use ratio_safe;

#[macro_export]
macro_rules! ratio_saturating {
    ($a:expr, $b:expr) => {{
        let aa = $a as f32;
        let bb = ($b as f32).abs();
        if std::hint::likely(aa > 0.0) {
            (aa / bb).min(1.0)
        }
        else {
            0.0
        }
    }};
}
pub use ratio_saturating;

#[macro_export]
macro_rules! ratio_warpping {
    ($a:expr, $b:expr) => {{
        let aa = $a as f32;
        let bb = ($b as f32).abs();
        let r = (aa % bb) / bb;
        if std::hint::likely(r >= 0.0) {
            r
        }
        else if std::hint::likely(r < 0.0) {
            r + 1.0
        }
        else {
            0.0 // NaN/Inf
        }
    }};
}
pub use ratio_warpping;

#[inline(always)]
pub fn calc_fade_in(prev_weight: f32, time_step: f32, duration: f32) -> f32 {
    (prev_weight + time_step / duration).min(1.0)
}

#[inline]
pub fn quat_from_dir_xz(dir: Vec2xz) -> Quat {
    // 2D coordinate system is left-handed.
    // 3D coordinate system (used by CriticalPoint) is right-handed.
    // So swap `from` and `to` parameters here.
    let q = Quat::from_rotation_arc_2d(dir.as_vec2(), DEFAULT_TOWARD_DIR_2D.as_vec2());
    Quat::from_xyzw(0.0, q.z, 0.0, q.w)
}

#[inline]
pub fn dir_xz_from_quat(quat: Quat) -> Vec2xz {
    let dir = quat * DEFAULT_TOWARD_DIR_3D;
    let dir_xz = if likely(dir.y.abs() < 0.999) {
        Vec2xz::new(dir.x, dir.z)
    }
    else if dir.y > 0.0 {
        let dir = quat * Vec3A::NEG_Y;
        Vec2xz::new(dir.x, dir.z)
    }
    else {
        let dir = quat * Vec3A::Y;
        Vec2xz::new(dir.x, dir.z)
    };
    dir_xz.normalize()
}

#[inline]
pub fn cos_degree(deg: f32) -> f32 {
    deg.to_radians().cos()
}

#[inline]
pub fn sin_degree(deg: f32) -> f32 {
    deg.to_radians().sin()
}

#[inline]
pub fn to_euler_radius(quat: Quat) -> (f32, f32, f32) {
    quat.to_euler(glam::EulerRot::XYZ)
}

#[inline]
pub fn to_euler_degree(quat: Quat) -> (f32, f32, f32) {
    let euler = quat.to_euler(glam::EulerRot::XYZ);
    (euler.0.to_degrees(), euler.1.to_degrees(), euler.2.to_degrees())
}

//
// CSharp math
//

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct CsVec3A {
    // TODO: remove this in .net core
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub _w: f32,
}

impl CsVec3A {
    pub const ZERO: CsVec3A = CsVec3A::new(0.0, 0.0, 0.0);
    pub const ONE: CsVec3A = CsVec3A::new(1.0, 1.0, 1.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> CsVec3A {
        CsVec3A { x, y, z, _w: 0.0 }
    }
}

impl From<Vec3A> for CsVec3A {
    #[inline]
    fn from(v: Vec3A) -> CsVec3A {
        CsVec3A::new(v.x, v.y, v.z)
    }
}

impl From<CsVec3A> for Vec3A {
    #[inline]
    fn from(v: CsVec3A) -> Vec3A {
        Vec3A::new(v.x, v.y, v.z)
    }
}

impl PartialEq<Vec3A> for CsVec3A {
    #[inline]
    fn eq(&self, other: &Vec3A) -> bool {
        let zelf: Vec3A = (*self).into();
        &zelf == other
    }
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct CsVec4 {
    // TODO: remove this in .net core
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl CsVec4 {
    pub const ZERO: CsVec4 = CsVec4::new(0.0, 0.0, 0.0, 0.0);
    pub const ONE: CsVec4 = CsVec4::new(1.0, 1.0, 1.0, 1.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> CsVec4 {
        CsVec4 { x, y, z, w }
    }
}

impl From<Vec4> for CsVec4 {
    #[inline]
    fn from(v: Vec4) -> CsVec4 {
        CsVec4::new(v.x, v.y, v.z, v.w)
    }
}

impl From<CsVec4> for Vec4 {
    #[inline]
    fn from(v: CsVec4) -> Vec4 {
        Vec4::new(v.x, v.y, v.z, v.w)
    }
}

impl PartialEq<Vec4> for CsVec4 {
    #[inline]
    fn eq(&self, other: &Vec4) -> bool {
        let zelf: Vec4 = (*self).into();
        &zelf == other
    }
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct CsQuat {
    // TODO: remove this in .net core
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl CsQuat {
    pub const ZERO: CsQuat = CsQuat::new(0.0, 0.0, 0.0, 0.0);
    pub const IDENTITY: CsQuat = CsQuat::new(0.0, 0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> CsQuat {
        CsQuat { x, y, z, w }
    }
}

impl From<Quat> for CsQuat {
    #[inline]
    fn from(v: Quat) -> CsQuat {
        CsQuat::new(v.x, v.y, v.z, v.w)
    }
}

impl From<CsQuat> for Quat {
    #[inline]
    fn from(v: CsQuat) -> Quat {
        Quat::from_xyzw(v.x, v.y, v.z, v.w)
    }
}

impl PartialEq<Quat> for CsQuat {
    #[inline]
    fn eq(&self, other: &Quat) -> bool {
        let zelf: Quat = (*self).into();
        &zelf == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_ulps_eq;
    use std::f32::consts::{FRAC_PI_2, PI};
    use std::f32::{INFINITY, NAN};

    #[test]
    fn test_ratio_saturating() {
        assert_eq!(ratio_saturating!(-0.5, 2.0), 0.0);
        assert_eq!(ratio_saturating!(0.0, 2.0), 0.0);
        assert_eq!(ratio_saturating!(1.5, 2.0), 0.75);
        assert_eq!(ratio_saturating!(2.5, 2.0), 1.0);

        assert_eq!(ratio_saturating!(-0.5, -2.0), 0.0);
        assert_eq!(ratio_saturating!(0.0, -2.0), 0.0);
        assert_eq!(ratio_saturating!(1.5, -2.0), 0.75);
        assert_eq!(ratio_saturating!(2.5, -2.0), 1.0);

        assert_eq!(ratio_saturating!(-INFINITY, 2.0), 0.0);
        assert_eq!(ratio_saturating!(INFINITY, 2.0), 1.0);
        assert_eq!(ratio_saturating!(NAN, 2.0), 0.0);

        assert_eq!(ratio_saturating!(5.0, 0.0), 1.0);
        assert_eq!(ratio_saturating!(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_ratio_wrapping() {
        assert_eq!(ratio_warpping!(-2.5, 2.0), 0.75);
        assert_eq!(ratio_warpping!(-1.5, 2.0), 0.25);
        assert_eq!(ratio_warpping!(0.0, 2.0), 0.0);
        assert_eq!(ratio_warpping!(0.5, 2.0), 0.25);
        assert_eq!(ratio_warpping!(2.5, 2.0), 0.25);
        assert_eq!(ratio_warpping!(4.5, 2.0), 0.25);

        assert_eq!(ratio_warpping!(-2.5, -2.0), 0.75);
        assert_eq!(ratio_warpping!(0.5, -2.0), 0.25);
        assert_eq!(ratio_warpping!(4.5, -2.0), 0.25);

        assert_eq!(ratio_saturating!(-INFINITY, 2.0), 0.0);
        assert_eq!(ratio_saturating!(INFINITY, 2.0), 1.0);
        assert_eq!(ratio_saturating!(NAN, 2.0), 0.0);

        assert_eq!(ratio_saturating!(5.0, 0.0), 1.0);
        assert_eq!(ratio_saturating!(0.0, 0.0), 0.0);
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

    #[test]
    fn test_quat_dir_xz() {
        assert_ulps_eq!(quat_from_dir_xz(DEFAULT_TOWARD_DIR_2D), Quat::IDENTITY);
        assert_ulps_eq!(quat_from_dir_xz(Vec2xz::NEG_Z), Quat::from_rotation_y(PI));
        assert_ulps_eq!(quat_from_dir_xz(Vec2xz::X), Quat::from_rotation_y(FRAC_PI_2));
        assert_ulps_eq!(quat_from_dir_xz(Vec2xz::NEG_X), Quat::from_rotation_y(-FRAC_PI_2));

        assert_ulps_eq!(dir_xz_from_quat(Quat::IDENTITY), DEFAULT_TOWARD_DIR_2D);
        assert_ulps_eq!(dir_xz_from_quat(Quat::from_rotation_y(PI)), Vec2xz::NEG_Z);
        assert_ulps_eq!(dir_xz_from_quat(Quat::from_rotation_y(FRAC_PI_2)), Vec2xz::X);
        assert_ulps_eq!(dir_xz_from_quat(Quat::from_rotation_y(-FRAC_PI_2)), Vec2xz::NEG_X);
        assert_ulps_eq!(dir_xz_from_quat(Quat::from_rotation_z(FRAC_PI_2)), Vec2xz::Z);
        assert_ulps_eq!(dir_xz_from_quat(Quat::from_rotation_z(-FRAC_PI_2)), Vec2xz::Z);
        assert_ulps_eq!(
            dir_xz_from_quat(Quat::from_rotation_y(FRAC_PI_2) * Quat::from_rotation_z(FRAC_PI_2)),
            Vec2xz::X
        );
        assert_ulps_eq!(
            dir_xz_from_quat(Quat::from_rotation_y(-FRAC_PI_2) * Quat::from_rotation_z(-FRAC_PI_2)),
            Vec2xz::NEG_X
        );
    }
}
