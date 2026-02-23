use glam::{Quat, Vec3, Vec3A};
use jolt_physics_rs::{
    self as jolt, BoxShapeSettings, CapsuleShapeSettings, ConvexHullShapeSettings, CylinderShapeSettings,
    HeightFieldShapeSettings, IndexedTriangle, JRef, MeshShapeSettings, Plane, PlaneShapeSettings,
    RotatedTranslatedShapeSettings, ScaledShapeSettings, Shape, SphereShapeSettings, TaperedCapsuleShapeSettings,
    TaperedCylinderShapeSettings, TriangleShapeSettings,
};

use crate::asset::loader::AssetLoader;
use crate::utils::{
    default_axis_y, default_position, default_rotation, default_scale, xfrom, ShapeBox, ShapeCapsule, ShapeCylinder,
    ShapeSphere, ShapeTaperedCapsule, ShapeTaperedCylinder, Symbol, XResult,
};

impl AssetLoader {
    pub fn load_physics_shape_cached(&mut self, _shape: AssetShape) -> XResult<JRef<Shape>> {
        unimplemented!();
    }
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
#[serde(tag = "T")]
pub enum AssetShape {
    Sphere(AssetShapeSphere),
    Box(AssetShapeBox),
    Capsule(AssetShapeCapsule),
    TaperedCapsule(AssetShapeTaperedCapsule),
    Cylinder(AssetShapeCylinder),
    TaperedCylinder(AssetShapeTaperedCylinder),
    ConvexHull(AssetShapeConvexHull),
    Triangle(AssetShapeTriangle),
    Plane(AssetShapePlane),
    Mesh(AssetShapeMesh),
    HeightField(AssetShapeHeightField),
}

impl AssetShape {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        match self {
            AssetShape::Box(shape) => shape.create_physics(),
            AssetShape::Sphere(shape) => shape.create_physics(),
            AssetShape::Capsule(shape) => shape.create_physics(),
            AssetShape::TaperedCapsule(shape) => shape.create_physics(),
            AssetShape::Cylinder(shape) => shape.create_physics(),
            AssetShape::TaperedCylinder(shape) => shape.create_physics(),
            AssetShape::ConvexHull(shape) => shape.create_physics(),
            AssetShape::Triangle(shape) => shape.create_physics(),
            AssetShape::Plane(shape) => shape.create_physics(),
            AssetShape::Mesh(shape) => shape.create_physics(),
            AssetShape::HeightField(shape) => shape.create_physics(),
        }
    }

    fn apply_shape_transform(
        mut jolt_shape: JRef<Shape>,
        scale: &Vec3A,
        position: &Vec3A,
        rotation: &Quat,
    ) -> XResult<JRef<Shape>> {
        if *scale != Vec3A::ONE {
            let settings = ScaledShapeSettings::new(jolt_shape, *scale);
            jolt_shape = jolt::create_scaled_shape(&settings).map_err(xfrom!())?;
        }
        if *position != Vec3A::ZERO || *rotation != Quat::IDENTITY {
            let settings = RotatedTranslatedShapeSettings::new(jolt_shape, *position, *rotation);
            jolt_shape = jolt::create_rotated_translated_shape(&settings).map_err(xfrom!())?;
        }
        Ok(jolt_shape)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeSphere {
    pub radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl From<ShapeSphere> for AssetShapeSphere {
    fn from(shape: ShapeSphere) -> AssetShapeSphere {
        AssetShapeSphere {
            radius: shape.radius,
            ..Default::default()
        }
    }
}

impl AssetShapeSphere {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let settings = SphereShapeSettings::new(self.radius);
        let jolt_shape = jolt::create_sphere_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &self.scale, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeBox {
    pub half_x: f32,
    pub half_y: f32,
    pub half_z: f32,
    pub convex_radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl From<ShapeBox> for AssetShapeBox {
    fn from(shape: ShapeBox) -> AssetShapeBox {
        AssetShapeBox {
            half_x: shape.half_x,
            half_y: shape.half_y,
            half_z: shape.half_z,
            ..Default::default()
        }
    }
}

impl AssetShapeBox {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let mut settings = BoxShapeSettings::new(self.half_x, self.half_y, self.half_z);
        if self.convex_radius >= 0.0 {
            settings.convex_radius = self.convex_radius;
        }
        let jolt_shape = jolt::create_box_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &self.scale, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeCapsule {
    pub half_height: f32,
    pub radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl From<ShapeCapsule> for AssetShapeCapsule {
    fn from(shape: ShapeCapsule) -> AssetShapeCapsule {
        AssetShapeCapsule {
            half_height: shape.half_height,
            radius: shape.radius,
            ..Default::default()
        }
    }
}

impl AssetShapeCapsule {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let settings = CapsuleShapeSettings::new(self.half_height, self.radius);
        let jolt_shape = jolt::create_capsule_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &self.scale, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeTaperedCapsule {
    pub half_height: f32,
    pub top_radius: f32,
    pub bottom_radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl From<ShapeTaperedCapsule> for AssetShapeTaperedCapsule {
    fn from(shape: ShapeTaperedCapsule) -> AssetShapeTaperedCapsule {
        AssetShapeTaperedCapsule {
            half_height: shape.half_height,
            top_radius: shape.top_radius,
            bottom_radius: shape.bottom_radius,
            ..Default::default()
        }
    }
}

impl AssetShapeTaperedCapsule {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let settings = TaperedCapsuleShapeSettings::new(self.half_height, self.top_radius, self.bottom_radius);
        let jolt_shape = jolt::create_tapered_capsule_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &self.scale, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeCylinder {
    pub half_height: f32,
    pub radius: f32,
    pub convex_radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl From<ShapeCylinder> for AssetShapeCylinder {
    fn from(shape: ShapeCylinder) -> AssetShapeCylinder {
        AssetShapeCylinder {
            half_height: shape.half_height,
            radius: shape.radius,
            ..Default::default()
        }
    }
}

impl AssetShapeCylinder {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let mut settings = CylinderShapeSettings::new(self.radius, self.half_height);
        if self.convex_radius >= 0.0 {
            settings.convex_radius = self.convex_radius;
        }
        let jolt_shape = jolt::create_cylinder_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &self.scale, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeTaperedCylinder {
    pub half_height: f32,
    pub top_radius: f32,
    pub bottom_radius: f32,
    pub convex_radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl From<ShapeTaperedCylinder> for AssetShapeTaperedCylinder {
    fn from(shape: ShapeTaperedCylinder) -> AssetShapeTaperedCylinder {
        AssetShapeTaperedCylinder {
            half_height: shape.half_height,
            top_radius: shape.top_radius,
            bottom_radius: shape.bottom_radius,
            ..Default::default()
        }
    }
}

impl AssetShapeTaperedCylinder {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let mut settings = TaperedCylinderShapeSettings::new(self.half_height, self.top_radius, self.bottom_radius);
        if self.convex_radius >= 0.0 {
            settings.convex_radius = self.convex_radius;
        }
        let jolt_shape = jolt::create_tapered_cylinder_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &self.scale, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeConvexHull {
    pub points: Vec<Vec3A>,
    pub max_convex_radius: f32,
    pub max_error_convex_radius: f32,
    pub hull_tolerance: f32,
    pub convex_radius: f32,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetShapeConvexHull {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let mut settings = ConvexHullShapeSettings::new(&self.points);
        if self.max_convex_radius >= 0.0 {
            settings.max_convex_radius = self.max_convex_radius;
        }
        if self.max_error_convex_radius >= 0.0 {
            settings.max_error_convex_radius = self.max_error_convex_radius;
        }
        let jolt_shape = jolt::create_convex_hull_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &Vec3A::ONE, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeTriangle {
    pub vertices: [Vec3; 3],
    pub convex_radius: f32,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetShapeTriangle {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let mut settings = TriangleShapeSettings::new(self.vertices[0].into(), self.vertices[1].into(), self.vertices[2].into());
        if self.convex_radius >= 0.0 {
            settings.convex_radius = self.convex_radius;
        }
        let jolt_shape = jolt::create_triangle_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &Vec3A::ONE, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapePlane {
    #[serde(default = "default_axis_y")]
    pub normal: Vec3A,
    pub distance: f32,
    pub half_extent: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetShapePlane {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let plane = Plane::new(self.normal.into(), self.distance);
        let settings = PlaneShapeSettings::new(plane, self.half_extent);
        let jolt_shape = jolt::create_plane_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &self.scale, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeMesh {
    pub triangle_vertices: Vec<Vec3>,
    pub indexed_triangles: Vec<IndexedTriangle>,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetShapeMesh {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let settings = MeshShapeSettings::new(&self.triangle_vertices, &self.indexed_triangles);
        let jolt_shape = jolt::create_mesh_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &Vec3A::ONE, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeMeshFile {
    pub file: Symbol,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetShapeMeshFile {
    fn create_physics(&self) -> XResult<JRef<Shape>> {
        unimplemented!()
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeHeightField {
    pub sample_count: u32,
    pub min_height: f32,
    pub max_height: f32,
    pub heights: Vec<f32>,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetShapeHeightField {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        let mut settings = HeightFieldShapeSettings::new(&self.heights, self.sample_count);
        settings.min_height_value = self.min_height;
        settings.max_height_value = self.max_height;
        let jolt_shape = jolt::create_height_field_shape(&settings).map_err(xfrom!())?;
        AssetShape::apply_shape_transform(jolt_shape, &Vec3A::ONE, &self.position, &self.rotation)
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeHeightFieldFile {
    pub file: Symbol,
    pub sample_count: u32,
    pub min_height: f32,
    pub max_height: f32,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetShapeHeightFieldFile {
    pub fn create_physics(&self) -> XResult<JRef<Shape>> {
        unimplemented!()
    }
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetIndxedCompoundShape {
    pub sub_shapes: Vec<AssetIndexedSubshape>,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetIndexedSubshape {
    pub shape_index: u32,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}
