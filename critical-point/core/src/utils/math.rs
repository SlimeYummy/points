use glam_ext::{Quat, Transform3A, Vec3A, Vec4};

#[macro_export]
macro_rules! near {
    ($a:expr, $b:expr) => {
        $a.abs_diff_eq($b, crate::consts::FLOAT_EPSILON)
    };
    ($a:expr, $b:expr, $eps:expr) => {
        $a.abs_diff_eq($b, $eps)
    };
}
pub(crate) use near;

#[inline(always)]
pub fn calc_ratio(a: u32, b: u32) -> f32 {
    if b == 0 {
        1.0
    } else {
        (a as f32) / (b as f32)
    }
}

#[inline(always)]
pub fn calc_ratio_clamp(a: u32, b: u32) -> f32 {
    if a >= b {
        return 1.0;
    }
    calc_ratio(a, b)
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
#[archive_attr(derive(Debug))]
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
#[archive_attr(derive(Debug))]
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
#[archive_attr(derive(Debug))]
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
