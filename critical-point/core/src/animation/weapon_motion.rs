use glam::{Quat, Vec3, Vec3A};
use ozz_animation_rs::{Archive, OzzError, Track, TrackSamplingJobRef};
use std::fmt::Debug;
use std::hint::likely;
use std::io::{ErrorKind, Read};
use std::ops::Index;
use std::path::Path;

use crate::animation::utils::WeaponTransform;
use crate::utils::{ifelse, strict_gt, xres, Symbol, XResult};

#[derive(Debug)]
pub struct WeaponMotion {
    tracks: Vec<WeaponTrack>,
}

impl WeaponMotion {
    #[inline]
    pub fn from_archive(archive: &mut Archive<impl Read>) -> XResult<WeaponMotion> {
        let mut tracks = Vec::with_capacity(2);
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

            tracks.push(WeaponTrack {
                name: Symbol::new(pos_name)?,
                position,
                rotation,
            });
        }
        Ok(WeaponMotion { tracks })
    }

    fn parse_name<'t>(raw_name: &'t str, excepted_role: &str) -> XResult<&'t str> {
        let mut split = raw_name.split(':');

        let role = match split.next() {
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
    pub fn from_path<P: AsRef<Path>>(path: P) -> XResult<WeaponMotion> {
        let mut archive = Archive::from_path(path)?;
        WeaponMotion::from_archive(&mut archive)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &WeaponTrack> {
        self.tracks.iter()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&WeaponTrack> {
        self.tracks.get(index)
    }

    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.tracks.iter().position(|track| track.name() == name)
    }
}

impl Index<usize> for WeaponMotion {
    type Output = WeaponTrack;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.tracks[index]
    }
}

#[derive(Debug, Default)]
pub struct WeaponTrack {
    name: Symbol,
    position: Track<Vec3>,
    rotation: Track<Quat>,
}

impl WeaponTrack {
    #[inline]
    pub fn name(&self) -> Symbol {
        self.name
    }

    /// All croodinates are in model space
    pub fn sample(&self, ratio: f32) -> XResult<(Vec3A, Quat)> {
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

pub fn sample_weapons_by_name_with_weight(
    weapon_motion: &WeaponMotion,
    ratio: f32,
    weight: f32,
    transform: &mut Vec<WeaponTransform>,
) -> XResult<()> {
    for track in weapon_motion.iter() {
        debug_assert!(0.0 <= ratio && ratio <= 1.0);
        debug_assert!(weight >= 0.0);

        let (position, rotation) = track.sample(ratio)?;
        if let Some(tmp) = transform.iter_mut().find(|t| t.name == track.name()) {
            tmp.weight += weight;
            tmp.position += position * weight;
            let dot = tmp.rotation.dot(rotation);
            let sign = ifelse!(dot < 0.0, -1.0, 1.0);
            tmp.rotation += rotation * sign * weight;
        }
        else {
            transform.push(WeaponTransform {
                name: track.name(),
                position: position * weight,
                rotation: rotation * weight,
                weight: weight,
            });
        }
    }
    Ok(())
}

pub fn normalize_weapons_by_weight(transform: &mut Vec<WeaponTransform>) {
    for weapon in transform.iter_mut() {
        if likely(strict_gt!(weapon.weight, 0.0)) {
            weapon.position /= weapon.weight;
            weapon.rotation = weapon.rotation.normalize();
        }
        else {
            weapon.position = Vec3A::ZERO;
            weapon.rotation = Quat::IDENTITY;
        }
        weapon.weight = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::TEST_ASSET_PATH;

    use super::*;

    #[test]
    fn test_weapon_motion_track_set() {
        WeaponMotion::from_path(format!("{}/Girl_Attack_01A.wm-ozz", TEST_ASSET_PATH)).unwrap();
    }

    #[test]
    fn test_sample_weapons_by_name_with_weight() {
        let tracks = WeaponMotion::from_path(format!("{}/Girl_Attack_01A.wm-ozz", TEST_ASSET_PATH)).unwrap();
        let mut transform = Vec::new();

        sample_weapons_by_name_with_weight(&tracks, 0.3, 0.5, &mut transform).unwrap();
        assert_eq!(transform.len(), 1);
        assert_eq!(transform[0].name, "Axe");
        assert_eq!(transform[0].weight, 0.5);
        let (pos1, rot1) = tracks.tracks[0].sample(0.3).unwrap();
        assert_eq!(transform[0].position, pos1 * 0.5);
        assert_eq!(transform[0].rotation, rot1 * 0.5);

        sample_weapons_by_name_with_weight(&tracks, 0.6, 0.7, &mut transform).unwrap();
        assert_eq!(transform.len(), 1);
        assert_eq!(transform[0].name, "Axe");
        assert_eq!(transform[0].weight, 1.2);
        let (pos2, rot2) = tracks.tracks[0].sample(0.6).unwrap();
        assert_eq!(transform[0].position, pos2 * 0.7 + pos1 * 0.5);
        assert_eq!(transform[0].rotation, -rot2 * 0.7 + rot1 * 0.5);
    }

    #[test]
    fn test_normalize_weapons_by_weight() {
        let mut transform = vec![WeaponTransform {
            name: Symbol::new("Axe").unwrap(),
            position: Vec3A::new(1.0, 0.0, 1.0),
            rotation: Quat::IDENTITY,
            weight: 0.5,
        }];
        normalize_weapons_by_weight(&mut transform);
        assert_eq!(transform[0].weight, 1.0);
        assert_eq!(transform[0].position, Vec3A::new(2.0, 0.0, 2.0));
        assert_eq!(transform[0].rotation, Quat::IDENTITY);

        let mut transform = vec![WeaponTransform {
            name: Symbol::new("Axe").unwrap(),
            position: Vec3A::new(1.0, 0.0, 1.0),
            rotation: Quat::from_xyzw(1.0, 1.0, 1.0, 1.0),
            weight: 0.0,
        }];
        normalize_weapons_by_weight(&mut transform);
        assert_eq!(transform[0].weight, 1.0);
        assert_eq!(transform[0].position, Vec3A::ZERO);
        assert_eq!(transform[0].rotation, Quat::IDENTITY);
    }
}
