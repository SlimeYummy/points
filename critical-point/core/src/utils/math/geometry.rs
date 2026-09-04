use glam::{Quat, Vec3A};
use glam_ext::Vec2xz;

use crate::consts::{DEFAULT_TOWARD_ANGLE_2D, DEFAULT_TOWARD_DIR_2D};

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
pub fn default_axis_x() -> Vec3A {
    Vec3A::X
}

#[inline(always)]
pub fn default_axis_y() -> Vec3A {
    Vec3A::Y
}

#[inline(always)]
pub fn default_axis_z() -> Vec3A {
    Vec3A::Z
}

/// Calculate rotation from two normalized directions in the XZ plane.
#[inline(always)]
pub fn quat_between_dir_xz(from: Vec2xz, to: Vec2xz) -> Quat {
    // Right-handed XZ coordinate is different from the default 2D coordinate system.
    // So swap `from` and `to` parameters here.
    let q = Quat::from_rotation_arc_2d(to.as_vec2(), from.as_vec2());
    Quat::from_xyzw(0.0, q.z, 0.0, q.w)
}

/// Calculate rotation from two angles in the XZ plane (in radians).
#[inline(always)]
pub fn quat_between_rot_y(from: f32, to: f32) -> Quat {
    Quat::from_rotation_y(to - from)
}

#[inline(always)]
pub fn quat_from_default_toward_xz(dir: Vec2xz) -> Quat {
    quat_between_dir_xz(DEFAULT_TOWARD_DIR_2D, dir)
}

#[inline(always)]
pub fn quat_from_default_toward_rot_y(rot: f32) -> Quat {
    quat_between_rot_y(DEFAULT_TOWARD_ANGLE_2D, rot)
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

#[inline]
pub fn calc_dir(from: Vec3A, to: Vec3A, def: Vec3A) -> Vec3A {
    let dir = to - from;
    match dir.length_squared() > 1e-6 {
        true => dir.normalize(),
        false => def,
    }
}

#[inline]
pub fn calc_dir_xz(from: Vec2xz, to: Vec2xz, def: Vec2xz) -> Vec2xz {
    let dir = to - from;
    match dir.length_squared() > 1e-6 {
        true => dir.normalize(),
        false => def,
    }
}

/// Compute intersection of two circles in 2D XZ plane.
/// Returns (p1, p2) - the two intersection points, or two copies of the same point if tangent.
///
/// # Arguments
/// * `c1_pos` - center of circle 1
/// * `c1_rad` - radius of circle 1
/// * `c2_pos` - center of circle 2
/// * `c2_rad` - radius of circle 2
///
/// # Returns
/// `Option<(Vec2xz, Vec2xz)>` - two intersection points, or None if circles don't intersect
#[inline]
pub fn circle_circle_intersection(
    c1_pos: Vec2xz,
    c1_rad: f32,
    c2_pos: Vec2xz,
    c2_rad: f32,
) -> Option<(Vec2xz, Vec2xz)> {
    let to_other = c2_pos - c1_pos;
    let d = to_other.length();

    // Check if circles intersect
    if d < 1e-4 || (c1_rad - c2_rad).abs() > d || d > c1_rad + c2_rad {
        return None;
    }

    // Distance from c1 along the line to c2 to the chord midpoint
    let a = (c1_rad * c1_rad - c2_rad * c2_rad + d * d) / (2.0 * d);
    let h_sq = c1_rad * c1_rad - a * a;
    let h = if h_sq > 0.0 { h_sq.sqrt() } else { 0.0 };

    // Unit vector from c1 toward c2
    let ex = to_other / d;
    // Perpendicular vector (rotate 90°)
    let perp = Vec2xz::new(-ex.z, ex.x);

    // Midpoint along the line from c1 to c2
    let mid = c1_pos + ex * a;

    // Two intersection points
    let p1 = mid + perp * h;
    let p2 = mid - perp * h;

    Some((p1, p2))
}

/// Binary Exponentiation for Quaternions with integer exponent.
#[inline]
pub fn quat_pow_i64(mut base: Quat, exponent: i64) -> Quat {
    if exponent == 0 {
        return Quat::IDENTITY;
    }
    else if exponent == 1 {
        return base;
    }
    else if exponent == -1 {
        return base.inverse();
    }

    if exponent < 0 {
        base = base.inverse();
    }

    let mut result = Quat::IDENTITY;
    let mut power = exponent.unsigned_abs();
    while power > 0 {
        if (power & 1) != 0 {
            result = (result * base).normalize();
        }
        power >>= 1;
        if power > 0 {
            base = (base * base).normalize();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::DEFAULT_TOWARD_DIR_3D;
    use approx::assert_ulps_eq;
    use glam::Vec3;

    #[test]
    fn test_quat_between_dir_xz() {
        let identity = quat_between_dir_xz(DEFAULT_TOWARD_DIR_2D, DEFAULT_TOWARD_DIR_2D);
        assert_eq!(identity, Quat::IDENTITY);
        let identity = quat_between_dir_xz(Vec2xz::X, Vec2xz::X);
        assert_eq!(identity, Quat::IDENTITY);

        let quat_xz = quat_between_dir_xz(Vec2xz::X, DEFAULT_TOWARD_DIR_2D);
        let quat_3d = Quat::from_rotation_arc(Vec3::X, DEFAULT_TOWARD_DIR_3D.into());
        assert_eq!(quat_xz, quat_3d);

        let quat_xz = quat_between_dir_xz(DEFAULT_TOWARD_DIR_2D, Vec2xz::NEG_X);
        let quat_3d = Quat::from_rotation_arc(DEFAULT_TOWARD_DIR_3D.into(), Vec3::NEG_X);
        assert_eq!(quat_xz, quat_3d);

        let quat_xz = quat_between_dir_xz(DEFAULT_TOWARD_DIR_2D, Vec2xz::new(1.0, 1.0).normalize());
        let quat_3d = Quat::from_rotation_arc(DEFAULT_TOWARD_DIR_3D.into(), Vec3::new(1.0, 0.0, 1.0).normalize());
        assert_ulps_eq!(quat_xz, quat_3d);
    }

    #[test]
    fn test_quat_between_rot_y() {
        let identity = quat_between_rot_y(0.0, 0.0);
        assert_eq!(identity, Quat::IDENTITY);
        let identity = quat_between_rot_y(DEFAULT_TOWARD_ANGLE_2D, DEFAULT_TOWARD_ANGLE_2D);
        assert_eq!(identity, Quat::IDENTITY);

        let quat_ry = quat_between_rot_y(0.0, DEFAULT_TOWARD_ANGLE_2D);
        let quat_3d = Quat::from_rotation_arc(Vec3::X, DEFAULT_TOWARD_DIR_3D.into());
        assert_eq!(quat_ry, quat_3d);

        let quat_ry = quat_between_rot_y(DEFAULT_TOWARD_ANGLE_2D, -std::f32::consts::PI);
        let quat_3d = Quat::from_rotation_arc(DEFAULT_TOWARD_DIR_3D.into(), Vec3::NEG_X);
        assert_eq!(quat_ry, quat_3d);

        let quat_ry = quat_between_rot_y(DEFAULT_TOWARD_ANGLE_2D, std::f32::consts::FRAC_PI_4);
        let quat_3d = Quat::from_rotation_arc(DEFAULT_TOWARD_DIR_3D.into(), Vec3::new(1.0, 0.0, -1.0).normalize());
        assert_ulps_eq!(quat_ry, quat_3d);
    }

    #[test]
    fn test_circle_circle_intersection() {
        // Two circles that intersect
        let c1 = Vec2xz::ZERO;
        let r1 = 5.0;
        let c2 = Vec2xz::new(6.0, 0.0);
        let r2 = 5.0;

        let result = circle_circle_intersection(c1, r1, c2, r2);
        assert!(result.is_some());
        let (p1, p2) = result.unwrap();

        // Both points should be at distance r1 from c1
        assert_ulps_eq!((p1 - c1).length(), r1, epsilon = 1e-4);
        assert_ulps_eq!((p2 - c1).length(), r1, epsilon = 1e-4);

        // Both points should be at distance r2 from c2
        assert_ulps_eq!((p1 - c2).length(), r2, epsilon = 1e-4);
        assert_ulps_eq!((p2 - c2).length(), r2, epsilon = 1e-4);
    }

    #[test]
    fn test_circle_circle_no_intersection() {
        // Two circles that don't intersect (too far apart)
        let c1 = Vec2xz::ZERO;
        let r1 = 2.0;
        let c2 = Vec2xz::new(10.0, 0.0);
        let r2 = 2.0;

        let result = circle_circle_intersection(c1, r1, c2, r2);
        assert!(result.is_none());
    }

    #[test]
    fn test_circle_circle_one_inside() {
        // One circle inside the other
        let c1 = Vec2xz::ZERO;
        let r1 = 10.0;
        let c2 = Vec2xz::new(2.0, 0.0);
        let r2 = 2.0;

        let result = circle_circle_intersection(c1, r1, c2, r2);
        assert!(result.is_none());
    }
}
