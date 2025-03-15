use super::rkyv_self;

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ShapeBox {
    pub half_x: f32,
    pub half_y: f32,
    pub half_z: f32,
}

rkyv_self!(ShapeBox);

impl ShapeBox {
    #[inline]
    pub fn new(half_x: f32, half_y: f32, half_z: f32) -> ShapeBox {
        ShapeBox { half_x, half_y, half_z }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ShapeSphere {
    pub radius: f32,
}

rkyv_self!(ShapeSphere);

impl ShapeSphere {
    #[inline]
    pub fn new(radius: f32) -> ShapeSphere {
        ShapeSphere { radius }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ShapeCapsule {
    pub half_height: f32,
    pub radius: f32,
}

rkyv_self!(ShapeCapsule);

impl ShapeCapsule {
    #[inline]
    pub fn new(half_height: f32, radius: f32) -> ShapeCapsule {
        ShapeCapsule { half_height, radius }
    }
}
