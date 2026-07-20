use ozz_animation_rs::{Archive, OzzError, Track, TrackSamplingJobRef};
use std::fmt::Debug;
use std::hint::likely;
use std::io::{ErrorKind, Read};
use std::ops::Index;
use std::path::Path;

use crate::animation::utils::ShapeKeyValue;
use crate::utils::{Symbol, XResult, sb, strict_gt};

#[derive(Debug)]
pub struct ShapeKey {
    tracks: Vec<ShapeTrack>,
}

impl ShapeKey {
    #[inline]
    pub fn from_archive(archive: &mut Archive<impl Read>) -> XResult<ShapeKey> {
        let mut tracks = Vec::new();
        loop {
            let val_track = match Track::<f32>::from_archive(archive) {
                Ok(track) => track,
                Err(OzzError::IO(ErrorKind::UnexpectedEof)) => break,
                Err(err) => return Err(err.into()),
            };
            tracks.push(ShapeTrack {
                name: sb!(val_track.name()),
                val_track,
            });
        }
        Ok(ShapeKey { tracks })
    }

    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> XResult<ShapeKey> {
        let mut archive = Archive::from_path(path)?;
        ShapeKey::from_archive(&mut archive)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ShapeTrack> {
        self.tracks.iter()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&ShapeTrack> {
        self.tracks.get(index)
    }

    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.tracks.iter().position(|track| track.name() == name)
    }
}

impl Index<usize> for ShapeKey {
    type Output = ShapeTrack;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.tracks[index]
    }
}

#[derive(Debug, Default)]
pub struct ShapeTrack {
    name: Symbol,
    val_track: Track<f32>,
}

impl ShapeTrack {
    #[inline]
    pub fn name(&self) -> Symbol {
        self.name
    }

    pub fn sample(&self, ratio: f32) -> XResult<f32> {
        let mut job = TrackSamplingJobRef::<f32>::default();
        job.set_track(&self.val_track);
        job.set_ratio(ratio);
        job.run()?;
        Ok(job.result())
    }
}

pub fn sample_shape_key_by_name_weight(
    shape_key: &ShapeKey,
    ratio: f32,
    weight: f32,
    values: &mut Vec<ShapeKeyValue>,
) -> XResult<()> {
    debug_assert!(0.0 <= ratio && ratio <= 1.0);
    debug_assert!(weight >= 0.0);

    for track in shape_key.iter() {
        let value = track.sample(ratio)?;
        if let Some(skv) = values.iter_mut().find(|skv| skv.name == track.name()) {
            skv.value += value * weight;
            skv.weight += weight;
        }
        else {
            values.push(ShapeKeyValue {
                name: track.name(),
                value: value * weight,
                weight,
            });
        }
    }
    Ok(())
}

pub fn normalize_shape_key_by_weight(values: &mut Vec<ShapeKeyValue>) {
    for value in values.iter_mut() {
        if likely(strict_gt!(value.weight, 0.0)) {
            value.value /= value.weight;
        }
        else {
            value.value = 0.0;
        }
        value.weight = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;

    #[test]
    fn test_shape_key_set() {
        ShapeKey::from_path(format!("{}/Slime/RunLoop.sk-ozz", TEST_ASSET_PATH)).unwrap();
    }

    #[test]
    fn test_sample_shape_key_by_name_weight() {
        let shape_key = ShapeKey::from_path(format!("{}/Slime/RunLoop.sk-ozz", TEST_ASSET_PATH)).unwrap();
        let mut values = Vec::new();

        sample_shape_key_by_name_weight(&shape_key, 0.3, 0.5, &mut values).unwrap();
        assert_eq!(values.len(), shape_key.len());
        assert_eq!(values[1].name, shape_key[1].name());
        assert_eq!(values[1].weight, 0.5);
        let val01 = shape_key[1].sample(0.3).unwrap();
        assert_eq!(values[1].value, val01 * 0.5);

        sample_shape_key_by_name_weight(&shape_key, 0.6, 0.7, &mut values).unwrap();
        assert_eq!(values.len(), shape_key.len());
        assert_eq!(values[0].name, shape_key[0].name());
        assert_eq!(values[0].weight, 1.2);
        let val00 = shape_key[0].sample(0.3).unwrap();
        let val10 = shape_key[0].sample(0.6).unwrap();
        assert_eq!(values[0].value, val10 * 0.7 + val00 * 0.5);
    }

    #[test]
    fn test_normalize_shape_key_by_weight() {
        let mut values = vec![ShapeKeyValue {
            name: sb!("Idle_Vert"),
            value: 2.0,
            weight: 0.5,
        }];
        normalize_shape_key_by_weight(&mut values);
        assert_eq!(values[0].weight, 1.0);
        assert_eq!(values[0].value, 4.0);

        let mut values = vec![ShapeKeyValue {
            name: sb!("Idle_Vert"),
            value: 2.0,
            weight: 0.0,
        }];
        normalize_shape_key_by_weight(&mut values);
        assert_eq!(values[0].weight, 1.0);
        assert_eq!(values[0].value, 0.0);
    }
}
