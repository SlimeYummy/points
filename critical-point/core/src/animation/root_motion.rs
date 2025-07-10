use glam::{Quat, Vec3, Vec3Swizzles};
use ozz_animation_rs::{Archive, Track};
use std::io::Read;
use std::path::Path;

use crate::utils::XResult;

#[derive(Debug)]
pub struct RootMotionTrack {
    pub position: Track<Vec3>,
    pub rotation: Track<Quat>,
}

impl RootMotionTrack {
    #[inline]
    pub fn from_archive(archive: &mut Archive<impl Read>) -> XResult<RootMotionTrack> {
        let position = Track::<Vec3>::from_archive(archive)?;
        let rotation = Track::<Quat>::from_archive(archive)?;
        Ok(RootMotionTrack { position, rotation })
    }

    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> XResult<RootMotionTrack> {
        let mut archive = Archive::from_path(path)?;
        RootMotionTrack::from_archive(&mut archive)
    }

    #[inline]
    pub fn has_position(&self) -> bool {
        self.position.key_count() > 0
    }

    #[inline]
    pub fn has_rotation(&self) -> bool {
        self.rotation.key_count() > 0
    }

    pub fn max_xz_distance(&self) -> f32 {
        let mut max2: f32 = 0.0;
        for val in self.position.values() {
            max2 = max2.max(val.xz().length_squared());
        }
        max2.sqrt()
    }
}
