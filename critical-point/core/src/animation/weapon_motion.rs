use glam::{Quat, Vec3, Vec3A};
use ozz_animation_rs::{Archive, OzzError, Track, TrackSamplingJobRef};
use std::ops::Index;
use std::{fmt::Debug, io::ErrorKind};
use std::io::Read;
use std::path::Path;

use crate::utils::{xres, Symbol, XResult};

#[derive(Debug)]
pub struct WeaponMotionTrackSet {
    tracks: Vec<WeaponMotionTrack>,
}

impl WeaponMotionTrackSet {
    #[inline]
    pub fn from_archive(archive: &mut Archive<impl Read>) -> XResult<WeaponMotionTrackSet> {
        let mut tracks = Vec::new();
        loop {
            let position = match Track::<Vec3>::from_archive(archive) {
                Ok(track) => track,
                Err(OzzError::IO(ErrorKind::UnexpectedEof)) => break,
                Err(err) => return Err(err.into()),
            };
            let pos_name = Self::parse_name(position.name(), "Pos")?;

            let rotation: Track<Quat> = Track::<Quat>::from_archive(archive)?;
            let dir_name = Self::parse_name(rotation.name(), "Rot")?;

            if pos_name != dir_name {
                return xres!(BadAsset; "name missmatch");
            }

            tracks.push(WeaponMotionTrack {
                name: Symbol::new(pos_name)?,
                position,
                rotation,
            });
        }
        Ok(WeaponMotionTrackSet { tracks })
    }

    fn parse_name<'t>(raw_name: &'t str, excepted_role: &str) -> XResult<&'t str> {
        let mut split = raw_name.split(':');
        
        let role =  match split.next() {
            Some(role) => role,
            None => return xres!(BadAsset; "bad role"),
        };
        if role != excepted_role {
            return xres!(BadAsset; "role missmatch");
        }
        
        let name = match split.next() {
            Some(name) => name,
            None => return xres!(BadAsset; "bad name"),
        };

        if split.next().is_some() {
            return xres!(BadAsset; "bad name end");
        }
        
        Ok(name)
    }

    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> XResult<WeaponMotionTrackSet> {
        let mut archive = Archive::from_path(path)?;
        WeaponMotionTrackSet::from_archive(&mut archive)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &WeaponMotionTrack> {
        self.tracks.iter()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&WeaponMotionTrack> {
        self.tracks.get(index)
    }

    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.tracks.iter().position(|track| track.name() == name)
    }
}

impl Index<usize> for WeaponMotionTrackSet {
    type Output = WeaponMotionTrack;
    
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.tracks[index]
    }
}

#[derive(Debug)]
pub struct WeaponMotionTrack {
    name: Symbol,
    position: Track<Vec3>,
    rotation: Track<Quat>,
}

impl WeaponMotionTrack {
    #[inline]
    pub fn name(&self) -> Symbol {
        self.name
    }

    /// All croodinates are in model space
    pub fn calc(&self, ratio: f32) -> XResult<(Vec3A, Quat)> {
        let mut pos_job = TrackSamplingJobRef::<Vec3>::default();
        pos_job.set_track(&self.position);
        pos_job.set_ratio(ratio);
        pos_job.run()?;
        let pos: Vec3A = pos_job.result().into();

        let mut dir_job = TrackSamplingJobRef::<Quat>::default();
        dir_job.set_track(&self.rotation);
        dir_job.set_ratio(ratio);
        dir_job.run()?;
        let rot = dir_job.result();

        Ok((pos, rot))
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::TEST_ASSET_PATH;

    use super::*;

    #[test]
    fn test_weapon_motion_track_set() {
        WeaponMotionTrackSet::from_path(format!("{}/girl_Attack_1_1.wm-ozz", TEST_ASSET_PATH)).unwrap();
    }
}
