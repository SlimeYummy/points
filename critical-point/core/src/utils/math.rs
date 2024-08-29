use glam::{Quat, Vec3A};
use std::mem;

#[inline(always)]
pub fn default_position() -> Vec3A {
    return Vec3A::ZERO;
}

#[inline(always)]
pub fn default_rotation() -> Quat {
    return Quat::IDENTITY;
}

#[inline(always)]
pub fn default_scale() -> Vec3A {
    return Vec3A::ONE;
}

#[inline(always)]
pub fn to_ratio(a: u32, b: u32) -> f32 {
    if b == 0 {
        return 1.0;
    } else {
        return (a as f32) / (b as f32);
    }
}

#[inline(always)]
pub fn to_ratio_clamp(a: u32, b: u32) -> f32 {
    if a >= b {
        return 1.0;
    }
    return to_ratio(a, b);
}
