use critical_point_csgen::CsEnum;
use glam::{Quat, Vec3};
use ozz_animation_rs::{Archive, OzzError, Track};
use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::utils::{rkyv_self, xres, xresf, XResult};

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, CsEnum)]
pub enum RootTrackName {
    #[default]
    Default = 0,
    Move = 1,
    MoveEx = 2,
}

rkyv_self!(RootTrackName);

#[derive(Debug)]
pub struct RootMotion {
    pub positions: [Track<Vec3>; 3],
    pub rotation: Track<Quat>,
}

impl RootMotion {
    #[inline]
    pub fn from_archive(archive: &mut Archive<impl Read>) -> XResult<RootMotion> {
        let mut rm = RootMotion {
            positions: Default::default(),
            rotation: Track::<Quat>::default(),
        };
        let mut pos_default = false;
        let mut pos_move = false;
        let mut pos_move_ex = false;
        let mut rot_default = false;

        loop {
            let meta = match Track::<f32>::read_meta(archive) {
                Ok(meta) => meta,
                Err(OzzError::IO(ErrorKind::UnexpectedEof)) => break,
                Err(err) => return Err(err.into()),
            };

            if meta.tag == "ozz-float3_track" {
                let tt = Track::<Vec3>::from_archive_with_meta(archive, meta)?;
                if tt.name() == "Pos:Default" {
                    if pos_default {
                        return xresf!(BadAsset; " name={}, duplication", tt.name());
                    }
                    pos_default = true;
                    rm.positions[0] = tt;
                }
                else if tt.name() == "Pos:Move" {
                    if pos_move {
                        return xresf!(BadAsset; " name={}, duplication", tt.name());
                    }
                    pos_move = true;
                    rm.positions[1] = tt;
                }
                else if tt.name() == "Pos:MoveEx" {
                    if pos_move_ex {
                        return xresf!(BadAsset; " name={}, duplication", tt.name());
                    }
                    pos_move_ex = true;
                    rm.positions[2] = tt;
                }
                else {
                    return xresf!(BadAsset; "name={}, unknown", tt.name());
                }
            }
            else if meta.tag == "ozz-quat_track" {
                let tt = Track::<Quat>::from_archive_with_meta(archive, meta)?;
                if tt.name() == "Rot:Default" {
                    if rot_default {
                        return xresf!(BadAsset; " name={}, duplication", tt.name());
                    }
                    rot_default = true;
                    rm.rotation = tt;
                }
                else {
                    return xresf!(BadAsset; "name={}, unknown", tt.name());
                }
            }
            else {
                return xresf!(BadAsset; "tag={}", meta.tag);
            }
        }

        if !pos_default {
            return xres!(BadAsset; "name=Pos:Default, notfound");
        }
        Ok(rm)
    }

    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> XResult<RootMotion> {
        let mut archive = Archive::from_path(path)?;
        RootMotion::from_archive(&mut archive)
    }

    #[inline]
    pub fn position(&self, tt: RootTrackName) -> &Track<Vec3> {
        &self.positions[tt as usize]
    }

    #[inline]
    pub fn has_position(&self, tt: RootTrackName) -> bool {
        self.positions[tt as usize].key_count() > 0
    }

    #[inline]
    pub fn first_position(&self, tt: RootTrackName) -> Vec3 {
        self.positions[tt as usize]
            .values()
            .first()
            .copied()
            .unwrap_or(Vec3::ZERO)
    }

    #[inline]
    pub fn last_position(&self, tt: RootTrackName) -> Vec3 {
        self.positions[tt as usize]
            .values()
            .last()
            .copied()
            .unwrap_or(Vec3::ZERO)
    }

    #[inline]
    pub fn whole_position(&self, tt: RootTrackName) -> Vec3 {
        self.last_position(tt) - self.first_position(tt)
    }

    #[inline]
    pub fn has_rotation(&self) -> bool {
        self.rotation.key_count() > 0
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
