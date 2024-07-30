use glam::{Quat, Vec3A};

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
