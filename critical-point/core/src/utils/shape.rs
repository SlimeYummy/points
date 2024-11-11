#[derive(
    Debug, Default, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct ShapeBox {
    pub half_x: f32,
    pub half_y: f32,
    pub half_z: f32,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct ShapeSphere {
    pub radius: f32,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct ShapeCapsule {
    pub half_height: f32,
    pub radius: f32,
}
