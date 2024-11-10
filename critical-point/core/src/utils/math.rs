use glam::{Quat, Vec3A};

#[inline(always)]
pub fn default_position() -> Vec3A {
    Vec3A::ZERO
}

#[inline(always)]
pub fn default_rotation() -> Quat {
    Quat::IDENTITY
}

#[inline(always)]
pub fn default_scale() -> Vec3A {
    Vec3A::ONE
}

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
