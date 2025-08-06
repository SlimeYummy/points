use glam::{Quat, Vec3};
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

    #[inline]
    pub fn first_position(&self) -> Vec3 {
        self.position.values().first().copied().unwrap_or(Vec3::ZERO)
    }

    #[inline]
    pub fn last_position(&self) -> Vec3 {
        self.position.values().last().copied().unwrap_or(Vec3::ZERO)
    }

    #[inline]
    pub fn whole_position(&self) -> Vec3 {
        self.last_position() - self.first_position()
    }

    #[inline]
    pub fn first_rotation(&self) -> Quat {
        self.rotation.values().first().copied().unwrap_or(Quat::IDENTITY)
    }

    #[inline]
    pub fn last_rotation(&self) -> Quat {
        self.rotation.values().last().copied().unwrap_or(Quat::IDENTITY)
    }

    #[inline]
    pub fn whole_rotation(&self) -> Quat {
        self.last_rotation() * self.first_rotation().inverse()
    }
}
