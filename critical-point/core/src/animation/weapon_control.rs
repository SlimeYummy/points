use critical_point_macros::csharp_out;
use ozz_animation_rs::{Archive, OzzError, Track, TrackSamplingJobRef};
use std::fmt::Debug;
use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::utils::{XResult, xres};

const WEAPON_CONTROL_ENABLE: u32 = 0x1;
const WEAPON_CONTROL_LEFT_HAND: u32 = 0x2;
const WEAPON_CONTROL_RIGHT_HAND: u32 = 0x4;

#[derive(Debug)]
pub struct WeaponControl {
    left: Option<Track<f32>>,
    right: Option<Track<f32>>,
}

impl WeaponControl {
    #[inline]
    pub fn from_archive(archive: &mut Archive<impl Read>) -> XResult<WeaponControl> {
        let mut wc = WeaponControl {
            left: None,
            right: None,
        };

        loop {
            let pos_track = match Track::<f32>::from_archive(archive) {
                Ok(track) => track,
                Err(OzzError::IO(ErrorKind::UnexpectedEof)) => break,
                Err(err) => return Err(err.into()),
            };

            if pos_track.name() == "LeftWeapon" {
                if wc.left.is_none() {
                    wc.left = Some(pos_track);
                }
                else {
                    return xres!(BadAsset; "left weapon duplication");
                }
            }
            else if pos_track.name() == "RightWeapon" {
                if wc.right.is_none() {
                    wc.right = Some(pos_track);
                }
                else {
                    return xres!(BadAsset; "right weapon duplication");
                }
            }
            else {
                return xres!(BadAsset; "unknown track name");
            }
        }

        Ok(wc)
    }

    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> XResult<WeaponControl> {
        let mut archive = Archive::from_path(path)?;
        WeaponControl::from_archive(&mut archive)
    }

    #[inline]
    pub fn has_left_weapon(&self) -> bool {
        self.left.is_some()
    }

    #[inline]
    pub fn has_right_weapon(&self) -> bool {
        self.right.is_some()
    }

    pub fn sample(&self, ratio: f32) -> XResult<WeaponStates> {
        Ok(WeaponStates {
            left: Self::sample_track(&self.left, ratio)?,
            right: Self::sample_track(&self.right, ratio)?,
        })
    }

    fn sample_track(track: &Option<Track<f32>>, ratio: f32) -> XResult<WeaponState> {
        match track {
            Some(track) => {
                let mut job = TrackSamplingJobRef::<f32>::default();
                job.set_track(track);
                job.set_ratio(ratio);
                job.run()?;
                let flags = job.result() as u32;
                Ok(WeaponState {
                    enabled: flags & WEAPON_CONTROL_ENABLE != 0,
                    left_hand: flags & WEAPON_CONTROL_LEFT_HAND != 0,
                    right_hand: flags & WEAPON_CONTROL_RIGHT_HAND != 0,
                })
            }
            None => Ok(WeaponState::default()),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct WeaponStates {
    pub left: WeaponState,
    pub right: WeaponState,
}

impl WeaponStates {
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct WeaponState {
    pub enabled: bool,
    pub left_hand: bool,
    pub right_hand: bool,
}

impl WeaponState {
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;

    #[test]
    fn test_weapon_control() {
        WeaponControl::from_path(format!("{}/Girl/Idle_Axe.wc-ozz", TEST_ASSET_PATH)).unwrap();
    }
}
