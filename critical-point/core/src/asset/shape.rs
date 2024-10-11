use glam::{Quat, Vec3, Vec3A};
use jolt_physics_rs::{
    self as jolt, BoxSettings, CapsuleSettings, ConvexHullSettings, CylinderSettings, HeightFieldSettings,
    IndexedTriangle, MeshSettings, RefShape, RotatedTranslatedSettings, ScaledSettings, SphereSettings,
    TaperedCapsuleSettings,
};
use serde::Deserialize;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::mem;

use crate::asset::loader::AssetLoader;
use crate::utils::{NumID, XError, XResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShapeKey {
    Box(ShapeKeyBox),
    Sphere(ShapeKeySphere),
    Capsule(ShapeKeyCapsule),
    TaperedCapsule(ShapeKeyTaperedCapsule),
    Cylinder(ShapeKeyCylinder),
    Scale(usize, ShapeKeyScale),
    Isometry(usize, ShapeKeyIsometry),
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct ShapeKeyBox {
    pub half_x: f32,
    pub half_y: f32,
    pub half_z: f32,
}

impl PartialEq for ShapeKeyBox {
    fn eq(&self, other: &Self) -> bool {
        float_num_equal(self.half_x, other.half_x)
            & float_num_equal(self.half_y, other.half_y)
            & float_num_equal(self.half_z, other.half_z)
    }
}

impl Eq for ShapeKeyBox {}

impl Hash for ShapeKeyBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_num_hash(self.half_x, state);
        float_num_hash(self.half_y, state);
        float_num_hash(self.half_z, state);
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct ShapeKeySphere {
    pub radius: f32,
}

impl PartialEq for ShapeKeySphere {
    fn eq(&self, other: &Self) -> bool {
        float_num_equal(self.radius, other.radius)
    }
}

impl Eq for ShapeKeySphere {}

impl Hash for ShapeKeySphere {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_num_hash(self.radius, state);
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ShapeKeyCapsule {
    pub half_height: f32,
    pub radius: f32,
}

impl PartialEq for ShapeKeyCapsule {
    fn eq(&self, other: &Self) -> bool {
        float_num_equal(self.half_height, other.half_height) & float_num_equal(self.radius, other.radius)
    }
}

impl Eq for ShapeKeyCapsule {}

impl Hash for ShapeKeyCapsule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_num_hash(self.half_height, state);
        float_num_hash(self.radius, state);
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ShapeKeyTaperedCapsule {
    pub half_height: f32,
    pub top_radius: f32,
    pub bottom_radius: f32,
}

impl PartialEq for ShapeKeyTaperedCapsule {
    fn eq(&self, other: &Self) -> bool {
        float_num_equal(self.half_height, other.half_height)
            & float_num_equal(self.top_radius, other.top_radius)
            & float_num_equal(self.bottom_radius, other.bottom_radius)
    }
}

impl Eq for ShapeKeyTaperedCapsule {}

impl Hash for ShapeKeyTaperedCapsule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_num_hash(self.half_height, state);
        float_num_hash(self.top_radius, state);
        float_num_hash(self.bottom_radius, state);
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ShapeKeyCylinder {
    pub half_height: f32,
    pub radius: f32,
}

impl PartialEq for ShapeKeyCylinder {
    fn eq(&self, other: &Self) -> bool {
        float_num_equal(self.half_height, other.half_height) & float_num_equal(self.radius, other.radius)
    }
}

impl Eq for ShapeKeyCylinder {}

impl Hash for ShapeKeyCylinder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_num_hash(self.half_height, state);
        float_num_hash(self.radius, state);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShapeKeyScale {
    pub scale: Vec3A,
}

impl PartialEq for ShapeKeyScale {
    fn eq(&self, other: &Self) -> bool {
        float_vec3a_equal(self.scale, other.scale)
    }
}

impl Eq for ShapeKeyScale {}

impl Hash for ShapeKeyScale {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_vec3a_hash(self.scale, state);
    }
}

impl Default for ShapeKeyScale {
    fn default() -> ShapeKeyScale {
        ShapeKeyScale { scale: Vec3A::ONE }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShapeKeyIsometry {
    #[serde(default = "crate::utils::default_position")]
    pub position: Vec3A,
    #[serde(default = "crate::utils::default_rotation")]
    pub rotation: Quat,
}

impl Default for ShapeKeyIsometry {
    fn default() -> ShapeKeyIsometry {
        ShapeKeyIsometry {
            position: Vec3A::ZERO,
            rotation: Quat::IDENTITY,
        }
    }
}

impl PartialEq for ShapeKeyIsometry {
    fn eq(&self, other: &Self) -> bool {
        float_vec3a_equal(self.position, other.position) && float_quat_equal(self.rotation, other.rotation)
    }
}

impl Eq for ShapeKeyIsometry {}

impl Hash for ShapeKeyIsometry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        float_vec3a_hash(self.position, state);
        float_quat_hash(self.rotation, state);
    }
}

#[inline(always)]
fn float_num_equal(a: f32, b: f32) -> bool {
    (a == b) | (a.to_bits() == b.to_bits())
}

#[inline(always)]
fn float_num_hash<H: Hasher>(n: f32, state: &mut H) {
    let u = n.to_bits();
    state.write_u32(if u == 0x8000_0000 { 0 } else { u });
}

#[inline(always)]
fn float_vec3a_equal(a: Vec3A, b: Vec3A) -> bool {
    let ua: (u64, u32, u32) = unsafe { mem::transmute(a) };
    let ub: (u64, u32, u32) = unsafe { mem::transmute(b) };
    (a == b) | ((ua.0 == ub.0) & (ua.1 == ub.1))
}

#[inline(always)]
fn float_vec3a_hash<H: Hasher>(v: Vec3A, state: &mut H) {
    float_num_hash(v.x, state);
    float_num_hash(v.y, state);
    float_num_hash(v.z, state);
}

#[inline(always)]
fn float_quat_equal(a: Quat, b: Quat) -> bool {
    let ua: u128 = unsafe { mem::transmute(a) };
    let ub: u128 = unsafe { mem::transmute(b) };
    (a == b) | (ua == ub)
}

#[inline(always)]
fn float_quat_hash<H: Hasher>(q: Quat, state: &mut H) {
    float_num_hash(q.x, state);
    float_num_hash(q.y, state);
    float_num_hash(q.z, state);
    float_num_hash(q.w, state);
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "T")]
pub enum AssetShape {
    Box(AssetShapeBox),
    Sphere(AssetShapeSphere),
    Capsule(AssetShapeCapsule),
    TaperedCapsule(AssetShapeTaperedCapsule),
    Cylinder(AssetShapeCylinder),
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeBox {
    #[serde(flatten)]
    shape: ShapeKeyBox,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeSphere {
    #[serde(flatten)]
    shape: ShapeKeySphere,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeCapsule {
    #[serde(flatten)]
    shape: ShapeKeyCapsule,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeTaperedCapsule {
    #[serde(flatten)]
    shape: ShapeKeyTaperedCapsule,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeCylinder {
    #[serde(flatten)]
    shape: ShapeKeyCylinder,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

impl AssetLoader {
    pub fn load_shape(&mut self, shape: AssetShape) -> XResult<RefShape> {
        match shape {
            AssetShape::Box(shape) => self.load_shape_box(shape),
            AssetShape::Sphere(shape) => self.load_shape_sphere(shape),
            AssetShape::Capsule(shape) => self.load_shape_capsule(shape),
            AssetShape::TaperedCapsule(shape) => self.load_shape_tapered_capsule(shape),
            AssetShape::Cylinder(shape) => self.load_shape_cylinder(shape),
        }
    }

    fn load_shape_box(&mut self, shape: AssetShapeBox) -> XResult<RefShape> {
        let ref_shape = self.get_or_create_shape(ShapeKey::Box(shape.shape), || {
            let settings = BoxSettings::new(shape.shape.half_x, shape.shape.half_y, shape.shape.half_z);
            Ok(jolt::create_shape_box(&settings))
        })?;
        self.apply_shape_transform(ref_shape, &shape.scale, &shape.isometry)
    }

    fn load_shape_sphere(&mut self, shape: AssetShapeSphere) -> XResult<RefShape> {
        let ref_shape = self.get_or_create_shape(ShapeKey::Sphere(shape.shape), || {
            let settings = SphereSettings::new(shape.shape.radius);
            Ok(jolt::create_shape_sphere(&settings))
        })?;
        self.apply_shape_transform(ref_shape, &shape.scale, &shape.isometry)
    }

    fn load_shape_capsule(&mut self, shape: AssetShapeCapsule) -> XResult<RefShape> {
        let ref_shape = self.get_or_create_shape(ShapeKey::Capsule(shape.shape.clone()), || {
            let settings = CapsuleSettings::new(shape.shape.half_height, shape.shape.radius);
            Ok(jolt::create_shape_capsule(&settings))
        })?;
        self.apply_shape_transform(ref_shape, &shape.scale, &shape.isometry)
    }

    fn load_shape_tapered_capsule(&mut self, shape: AssetShapeTaperedCapsule) -> XResult<RefShape> {
        let ref_shape = self.get_or_create_shape(ShapeKey::TaperedCapsule(shape.shape.clone()), || {
            let settings = TaperedCapsuleSettings::new(
                shape.shape.half_height,
                shape.shape.top_radius,
                shape.shape.bottom_radius,
            );
            Ok(jolt::create_shape_tapered_capsule(&settings))
        })?;
        self.apply_shape_transform(ref_shape, &shape.scale, &shape.isometry)
    }

    fn load_shape_cylinder(&mut self, shape: AssetShapeCylinder) -> XResult<RefShape> {
        let ref_shape = self.get_or_create_shape(ShapeKey::Cylinder(shape.shape.clone()), || {
            let settings = CylinderSettings::new(shape.shape.half_height, shape.shape.radius);
            Ok(jolt::create_shape_cylinder(&settings))
        })?;
        self.apply_shape_transform(ref_shape, &shape.scale, &shape.isometry)
    }

    fn apply_shape_transform(
        &mut self,
        ref_shape: RefShape,
        scale: &Option<ShapeKeyScale>,
        isometry: &Option<ShapeKeyIsometry>,
    ) -> XResult<RefShape> {
        let mut new_shape = ref_shape;
        if let Some(scale) = scale {
            new_shape = self.get_or_create_shape(ShapeKey::Scale(new_shape.as_usize(), scale.clone()), || {
                let settings = ScaledSettings::new(new_shape, scale.scale);
                Ok(jolt::create_shape_scaled(&settings))
            })?;
        }
        if let Some(isometry) = isometry {
            new_shape = self.get_or_create_shape(ShapeKey::Isometry(new_shape.as_usize(), isometry.clone()), || {
                let settings = RotatedTranslatedSettings::new(new_shape, isometry.position, isometry.rotation);
                Ok(jolt::create_shape_rotated_translated(&settings))
            })?;
        }
        Ok(new_shape)
    }

    #[inline(always)]
    fn get_or_create_shape<F>(&mut self, key: ShapeKey, func: F) -> XResult<RefShape>
    where
        F: FnOnce() -> XResult<RefShape>,
    {
        match self.shape_cache.entry(key) {
            Entry::Occupied(entry) => return Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                let ref_shape = func()?;
                return Ok(entry.insert(ref_shape).clone());
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "T")]
pub enum AssetShapeEx {
    Box(AssetShapeExBox),
    Sphere(AssetShapeExSphere),
    Capsule(AssetShapeExCapsule),
    TaperedCapsule(AssetShapeExTaperedCapsule),
    Cylinder(AssetShapeExCylinder),
    ConvexHull(AssetShapeExConvexHull),
    Mesh(AssetShapeExMesh),
    HeightField(AssetShapeExHeightField),
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExBox {
    shape_id: NumID,
    half_x: f32,
    half_y: f32,
    half_z: f32,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExSphere {
    shape_id: NumID,
    radius: f32,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExCapsule {
    shape_id: NumID,
    half_height: f32,
    radius: f32,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExTaperedCapsule {
    shape_id: NumID,
    half_height: f32,
    top_radius: f32,
    bottom_radius: f32,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExCylinder {
    shape_id: NumID,
    half_height: f32,
    radius: f32,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExConvexHull {
    shape_id: NumID,
    vertices: Vec<Vec3A>,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExMesh {
    shape_id: NumID,
    vertices: Vec<Vec3>,
    indices: Vec<IndexedTriangle>,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct AssetShapeExHeightField {
    shape_id: NumID,
    samples: Vec<f32>,
    sample_count: u32,
    #[serde(flatten)]
    scale: Option<ShapeKeyScale>,
    #[serde(flatten)]
    isometry: Option<ShapeKeyIsometry>,
}

impl AssetLoader {
    pub fn get_shape_ex(&mut self, shape_id: NumID) -> XResult<RefShape> {
        match self.shape_ex_cache.get(&shape_id) {
            Some(ref_shape) => Ok(ref_shape.clone()),
            None => Err(XError::PhysicShapeNotFound),
        }
    }

    pub fn load_shape_ex(&mut self, shape: &AssetShapeEx) -> XResult<()> {
        let (shape_id, ref_shape) = match shape {
            AssetShapeEx::Box(shape) => self.load_shape_ex_box(shape),
            AssetShapeEx::Sphere(shape) => self.load_shape_ex_sphere(shape),
            AssetShapeEx::Capsule(shape) => self.load_shape_ex_capsule(shape),
            AssetShapeEx::TaperedCapsule(shape) => self.load_shape_ex_tapered_capsule(shape),
            AssetShapeEx::Cylinder(shape) => self.load_shape_ex_cylinder(shape),
            AssetShapeEx::ConvexHull(shape) => self.load_shape_ex_convex_hull(shape),
            AssetShapeEx::Mesh(shape) => self.load_shape_ex_mesh(shape),
            AssetShapeEx::HeightField(shape) => self.load_shape_ex_height_field(shape),
        }?;
        self.shape_ex_cache.insert(shape_id, ref_shape);
        Ok(())
    }

    fn load_shape_ex_box(&mut self, shape: &AssetShapeExBox) -> XResult<(NumID, RefShape)> {
        let settings = BoxSettings::new(shape.half_x, shape.half_y, shape.half_z);
        let mut ref_shape = jolt::create_shape_box(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn load_shape_ex_sphere(&mut self, shape: &AssetShapeExSphere) -> XResult<(NumID, RefShape)> {
        let settings = SphereSettings::new(shape.radius);
        let mut ref_shape = jolt::create_shape_sphere(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn load_shape_ex_capsule(&mut self, shape: &AssetShapeExCapsule) -> XResult<(NumID, RefShape)> {
        let settings = CapsuleSettings::new(shape.half_height, shape.radius);
        let mut ref_shape = jolt::create_shape_capsule(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn load_shape_ex_tapered_capsule(&mut self, shape: &AssetShapeExTaperedCapsule) -> XResult<(NumID, RefShape)> {
        let settings = TaperedCapsuleSettings::new(shape.half_height, shape.top_radius, shape.bottom_radius);
        let mut ref_shape = jolt::create_shape_tapered_capsule(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn load_shape_ex_cylinder(&mut self, shape: &AssetShapeExCylinder) -> XResult<(NumID, RefShape)> {
        let settings = CylinderSettings::new(shape.half_height, shape.radius);
        let mut ref_shape = jolt::create_shape_cylinder(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn load_shape_ex_convex_hull(&mut self, shape: &AssetShapeExConvexHull) -> XResult<(NumID, RefShape)> {
        let settings = ConvexHullSettings::new(shape.vertices.as_slice());
        let mut ref_shape = jolt::create_shape_convex_hull(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn load_shape_ex_mesh(&mut self, shape: &AssetShapeExMesh) -> XResult<(NumID, RefShape)> {
        let settings = MeshSettings::new(shape.vertices.as_slice(), shape.indices.as_slice());
        let mut ref_shape = jolt::create_shape_mesh(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn load_shape_ex_height_field(&mut self, shape: &AssetShapeExHeightField) -> XResult<(NumID, RefShape)> {
        let settings = HeightFieldSettings::new(shape.samples.as_slice(), shape.sample_count);
        let mut ref_shape = jolt::create_shape_height_field(&settings);
        ref_shape = self.apply_shape_ex_transform(ref_shape, &shape.scale, &shape.isometry)?;
        Ok((shape.shape_id, ref_shape))
    }

    fn apply_shape_ex_transform(
        &mut self,
        ref_shape: RefShape,
        scale: &Option<ShapeKeyScale>,
        isometry: &Option<ShapeKeyIsometry>,
    ) -> XResult<RefShape> {
        let mut new_shape = ref_shape;
        if let Some(scale) = scale {
            let settings = ScaledSettings::new(new_shape, scale.scale);
            new_shape = jolt::create_shape_scaled(&settings);
        }
        if let Some(isometry) = isometry {
            let settings = RotatedTranslatedSettings::new(new_shape, isometry.position, isometry.rotation);
            new_shape = jolt::create_shape_rotated_translated(&settings);
        }
        Ok(new_shape)
    }
}
