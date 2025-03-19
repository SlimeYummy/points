use glam::{Quat, Vec3, Vec3A};
use jolt_physics_rs::{
    self as jolt, BoxShapeSettings, CapsuleShapeSettings, ConvexHullShapeSettings, CylinderShapeSettings,
    IndexedTriangle, JRef, MeshShapeSettings, Plane, PlaneShapeSettings, RotatedTranslatedShapeSettings,
    ScaledShapeSettings, Shape, SphereShapeSettings, TaperedCapsuleShapeSettings, TaperedCylinderShapeSettings,
    TriangleShapeSettings,
};

use crate::asset::loader::AssetLoader;
use crate::utils::{xfrom, ShapeBox, ShapeCapsule, ShapeSphere, Symbol, XResult};

#[inline(always)]
pub(crate) fn default_position() -> Vec3A {
    Vec3A::ZERO
}

#[inline(always)]
pub(crate) fn default_rotation() -> Quat {
    Quat::IDENTITY
}

#[inline(always)]
pub(crate) fn default_scale() -> Vec3A {
    Vec3A::ONE
}

#[inline(always)]
fn default_axis_y() -> Vec3A {
    Vec3A::Y
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
    MeshEmbedded(AssetShapeMeshEmbedded),
    Mesh(AssetShapeMesh),
    HeightField(AssetShapeHeightField),
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

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeConvexHull {
    pub points: Vec<Vec3A>,
    pub max_convex_radius: f32,
    pub max_error_convex_radius: f32,
    pub hull_tolerance: f32,
    pub convex_radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeTriangle {
    pub vertices: [Vec3A; 3],
    pub convex_radius: f32,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
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

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeMeshEmbedded {
    pub triangle_vertices: Vec<Vec3>,
    pub indexed_triangles: Vec<IndexedTriangle>,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeMesh {
    pub file: Symbol,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

#[derive(
    Default, Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AssetShapeHeightField {
    pub file: Symbol,
    #[serde(default = "default_scale")]
    pub scale: Vec3A,
    #[serde(default = "default_position")]
    pub position: Vec3A,
    #[serde(default = "default_rotation")]
    pub rotation: Quat,
}

impl AssetLoader {
    pub fn load_shape(&mut self, shape: &AssetShape) -> XResult<JRef<Shape>> {
        match shape {
            AssetShape::Box(shape) => self.load_shape_box(&shape),
            AssetShape::Sphere(shape) => self.load_shape_sphere(&shape),
            AssetShape::Capsule(shape) => self.load_shape_capsule(&shape),
            AssetShape::TaperedCapsule(shape) => self.load_shape_tapered_capsule(&shape),
            AssetShape::Cylinder(shape) => self.load_shape_cylinder(&shape),
            AssetShape::TaperedCylinder(shape) => self.load_shape_tapered_cylinder(&shape),
            AssetShape::ConvexHull(shape) => self.load_shape_convex_hull(&shape),
            AssetShape::Triangle(shape) => self.load_shape_triangle(&shape),
            AssetShape::Plane(shape) => self.load_shape_plane(&shape),
            AssetShape::MeshEmbedded(shape) => self.load_shape_mesh_embedded(&shape),
            AssetShape::Mesh(shape) => self.load_shape_mesh(&shape),
            AssetShape::HeightField(shape) => self.load_shape_height_field(&shape),
        }
    }

    fn load_shape_box(&mut self, shape: &AssetShapeBox) -> XResult<JRef<Shape>> {
        let mut settings = BoxShapeSettings::new(shape.half_x, shape.half_y, shape.half_z);
        if shape.convex_radius >= 0.0 {
            settings.convex_radius = shape.convex_radius;
        }
        let jolt_shape = jolt::create_box_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_sphere(&mut self, shape: &AssetShapeSphere) -> XResult<JRef<Shape>> {
        let settings = SphereShapeSettings::new(shape.radius);
        let jolt_shape = jolt::create_sphere_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_capsule(&mut self, shape: &AssetShapeCapsule) -> XResult<JRef<Shape>> {
        let settings = CapsuleShapeSettings::new(shape.half_height, shape.radius);
        let jolt_shape = jolt::create_capsule_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_tapered_capsule(&mut self, shape: &AssetShapeTaperedCapsule) -> XResult<JRef<Shape>> {
        let settings = TaperedCapsuleShapeSettings::new(shape.half_height, shape.top_radius, shape.bottom_radius);
        let jolt_shape = jolt::create_tapered_capsule_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_cylinder(&mut self, shape: &AssetShapeCylinder) -> XResult<JRef<Shape>> {
        let mut settings = CylinderShapeSettings::new(shape.radius, shape.half_height);
        if shape.convex_radius >= 0.0 {
            settings.convex_radius = shape.convex_radius;
        }
        let jolt_shape = jolt::create_cylinder_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_tapered_cylinder(&mut self, shape: &AssetShapeTaperedCylinder) -> XResult<JRef<Shape>> {
        let mut settings = TaperedCylinderShapeSettings::new(shape.half_height, shape.top_radius, shape.bottom_radius);
        if shape.convex_radius >= 0.0 {
            settings.convex_radius = shape.convex_radius;
        }
        let jolt_shape = jolt::create_tapered_cylinder_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_convex_hull(&mut self, shape: &AssetShapeConvexHull) -> XResult<JRef<Shape>> {
        let mut settings = ConvexHullShapeSettings::new(&shape.points);
        if shape.max_convex_radius >= 0.0 {
            settings.max_convex_radius = shape.max_convex_radius;
        }
        if shape.max_error_convex_radius >= 0.0 {
            settings.max_error_convex_radius = shape.max_error_convex_radius;
        }
        let jolt_shape = jolt::create_convex_hull_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_triangle(&mut self, shape: &AssetShapeTriangle) -> XResult<JRef<Shape>> {
        let mut settings = TriangleShapeSettings::new(shape.vertices[0], shape.vertices[1], shape.vertices[2]);
        if shape.convex_radius >= 0.0 {
            settings.convex_radius = shape.convex_radius;
        }
        let jolt_shape = jolt::create_triangle_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_plane(&mut self, shape: &AssetShapePlane) -> XResult<JRef<Shape>> {
        let plane = Plane::new(shape.normal.into(), shape.distance);
        let settings = PlaneShapeSettings::new(plane, shape.half_extent);
        let jolt_shape = jolt::create_plane_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_mesh_embedded(&mut self, shape: &AssetShapeMeshEmbedded) -> XResult<JRef<Shape>> {
        let settings = MeshShapeSettings::new(&shape.triangle_vertices, &shape.indexed_triangles);
        let jolt_shape = jolt::create_mesh_shape(&settings).map_err(xfrom!())?;
        self.apply_shape_transform(jolt_shape, &shape.scale, &shape.position, &shape.rotation)
    }

    fn load_shape_mesh(&mut self, _shape: &AssetShapeMesh) -> XResult<JRef<Shape>> {
        unimplemented!()
    }

    fn load_shape_height_field(&mut self, _shape: &AssetShapeHeightField) -> XResult<JRef<Shape>> {
        unimplemented!()
    }

    fn apply_shape_transform(
        &mut self,
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
