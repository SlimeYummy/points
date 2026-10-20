use glam::Vec4;
use ozz_animation_rs::{OzzObj, Skeleton};
use rkyv::vec::ArchivedVec;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use crate::consts::{EMPTY_JOINT_WEIGHT_INDEX, MAX_JOINT_WEIGHTS_TABLE};
use crate::utils::{Symbol, SymbolMap, XResult, xresf};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Deserialize, serde::Serialize)]
struct RawJointWeights {
    name: String,
    weights: HashMap<String, f32>,
}

pub type JointWeightsTableRc = JointWeightsTable<Rc<Vec<Vec4>>>;
pub type JointWeightsTableArc = JointWeightsTable<Arc<Vec<Vec4>>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct JointWeightsTable<W: OzzObj<Vec<Vec4>> = Rc<Vec<Vec4>>> {
    names: SymbolMap<u8>,
    table: Vec<W>,
}

impl<W: OzzObj<Vec<Vec4>>> JointWeightsTable<W> {
    #[inline]
    pub fn names(&self) -> &SymbolMap<u8> {
        &self.names
    }

    #[inline]
    pub fn get_index(&self, name: &str) -> u8 {
        match self.names.get(&Symbol::new(name).unwrap()) {
            Some(idx) => *idx,
            None => EMPTY_JOINT_WEIGHT_INDEX,
        }
    }

    #[inline]
    pub fn weights(&self) -> &Vec<W> {
        &self.table
    }

    #[inline]
    pub fn get_weights(&self, idx: u8) -> Option<&W> {
        self.table.get(idx as usize)
    }

    #[inline]
    pub fn get_weights_by_name(&self, name: &str) -> Option<&W> {
        match self.names.get(&Symbol::new(name).unwrap()) {
            Some(idx) => self.table.get(*idx as usize),
            None => None,
        }
    }

    fn from_rkyv_bytes_impl<F>(bytes: &[u8], skeleton: &Skeleton, new: F) -> XResult<JointWeightsTable<W>>
    where
        F: Fn(Vec<Vec4>) -> W,
    {
        let raws = unsafe { rkyv::access_unchecked::<ArchivedVec<RawJointWeights>>(bytes) };
        if raws.len() > MAX_JOINT_WEIGHTS_TABLE {
            return xresf!(BadArgument; "joint_weights len, len={}", raws.len());
        }

        let mut table = JointWeightsTable {
            names: SymbolMap::with_capacity_and_hasher(raws.len(), Default::default()),
            table: Vec::with_capacity(raws.len()),
        };

        for raw in raws.iter() {
            let num_soa = skeleton.num_soa_joints();
            let mut weights = vec![Vec4::ZERO; num_soa];

            for (name, w) in raw.weights.iter() {
                Self::apply_weight(skeleton, &mut weights, name.as_str(), *w)?;
            }

            table.table.push(new(weights));
            table
                .names
                .insert(Symbol::new(raw.name.as_str())?, table.table.len() as u8);
        }

        Ok(table)
    }

    fn from_json_bytes_impl<F>(bytes: &[u8], skeleton: &Skeleton, new: F) -> XResult<JointWeightsTable<W>>
    where
        F: Fn(Vec<Vec4>) -> W,
    {
        let raws: Vec<RawJointWeights> = serde_json::from_slice(bytes).unwrap();
        if raws.len() > MAX_JOINT_WEIGHTS_TABLE {
            return xresf!(BadArgument; "joint_weights len, len={}", raws.len());
        }

        let mut table = JointWeightsTable {
            names: SymbolMap::with_capacity_and_hasher(raws.len(), Default::default()),
            table: Vec::with_capacity(raws.len()),
        };

        for raw in raws {
            let num_soa = skeleton.num_soa_joints();
            let mut weights = vec![Vec4::ZERO; num_soa];

            for (name, w) in raw.weights {
                Self::apply_weight(skeleton, &mut weights, &name, w)?;
            }

            table.names.insert(Symbol::new(&raw.name)?, table.table.len() as u8);
            table.table.push(new(weights));
        }

        Ok(table)
    }

    #[inline]
    fn apply_weight(skeleton: &Skeleton, weights: &mut [Vec4], name: &str, w: f32) -> XResult<()> {
        let joint_idx = match skeleton.joint_by_name(name) {
            Some(idx) => idx as usize,
            None => return xresf!(BadAsset; "unknown joint joint='{}'", name),
        };

        let soa_idx = joint_idx / 4;
        let lane = joint_idx % 4;
        let mut v = weights[soa_idx];
        match lane {
            0 => v.x = w,
            1 => v.y = w,
            2 => v.z = w,
            3 => v.w = w,
            _ => unreachable!(),
        }
        weights[soa_idx] = v;
        Ok(())
    }

    pub fn names_from_path<P: AsRef<Path>>(path: P) -> XResult<Vec<String>> {
        let path = path.as_ref();
        let is_rkyv = path.extension().map_or(false, |ext| ext == "lw-rkyv");
        let bytes = std::fs::read(path)?;
        if is_rkyv {
            Self::names_from_rkyv(&bytes)
        }
        else {
            Self::names_from_json(&bytes)
        }
    }

    fn names_from_rkyv(bytes: &[u8]) -> XResult<Vec<String>> {
        let raws = unsafe { rkyv::access_unchecked::<ArchivedVec<RawJointWeights>>(bytes) };
        if raws.len() > MAX_JOINT_WEIGHTS_TABLE {
            return xresf!(BadArgument; "joint_weights len, len={}", raws.len());
        }

        Ok(raws.iter().map(|raw| raw.name.to_string()).collect())
    }

    fn names_from_json(bytes: &[u8]) -> XResult<Vec<String>> {
        let raws: Vec<RawJointWeights> = serde_json::from_slice(bytes)?;
        if raws.len() > MAX_JOINT_WEIGHTS_TABLE {
            return xresf!(BadArgument; "joint_weights len, len={}", raws.len());
        }

        Ok(raws.into_iter().map(|raw| raw.name).collect())
    }
}

impl JointWeightsTableRc {
    #[inline]
    pub fn from_path_rc<P: AsRef<Path>>(path: P, skeleton: &Skeleton) -> XResult<JointWeightsTableRc> {
        let path = path.as_ref();
        let is_rkyv = path.extension().map_or(false, |ext| ext == "lw-rkyv");
        let bytes = std::fs::read(path)?;
        if is_rkyv {
            Self::from_rkyv_bytes_impl(&bytes, skeleton, Rc::new)
        }
        else {
            Self::from_json_bytes_impl(&bytes, skeleton, Rc::new)
        }
    }

    #[inline]
    pub fn from_rkyv_bytes_rc(bytes: &[u8], skeleton: &Skeleton) -> XResult<JointWeightsTableRc> {
        Self::from_rkyv_bytes_impl(bytes, skeleton, Rc::new)
    }

    #[inline]
    pub fn from_json_bytes_rc(bytes: &[u8], skeleton: &Skeleton) -> XResult<JointWeightsTableRc> {
        Self::from_json_bytes_impl(bytes, skeleton, Rc::new)
    }
}

impl JointWeightsTableArc {
    #[inline]
    pub fn from_path_arc<P: AsRef<Path>>(path: P, skeleton: &Skeleton) -> XResult<JointWeightsTableArc> {
        let path = path.as_ref();
        let is_rkyv = path.extension().map_or(false, |ext| ext == "lw-rkyv");
        let bytes = std::fs::read(path)?;
        if is_rkyv {
            Self::from_rkyv_bytes_impl(&bytes, skeleton, Arc::new)
        }
        else {
            Self::from_json_bytes_impl(&bytes, skeleton, Arc::new)
        }
    }

    #[inline]
    pub fn from_rkyv_bytes_arc(bytes: &[u8], skeleton: &Skeleton) -> XResult<JointWeightsTableArc> {
        Self::from_rkyv_bytes_impl(bytes, skeleton, Arc::new)
    }

    #[inline]
    pub fn from_json_bytes_arc(bytes: &[u8], skeleton: &Skeleton) -> XResult<JointWeightsTableArc> {
        Self::from_json_bytes_impl(bytes, skeleton, Arc::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetLoader;
    use crate::consts::TEST_ASSET_PATH;
    use crate::utils::sb;
    use std::rc::Rc;

    fn load_girl_skeleton() -> Rc<Skeleton> {
        let mut loader = AssetLoader::new(TEST_ASSET_PATH).unwrap();
        loader.load_skeleton(sb!("Girl/Girl.*")).unwrap()
    }

    #[test]
    fn test_load_joint_weights_rc() {
        let skeleton = load_girl_skeleton();
        let table =
            JointWeightsTable::from_path_rc(format!("{}/Girl/Girl.lw-json", TEST_ASSET_PATH), &*skeleton).unwrap();
        assert_eq!(table.weights().len(), 8);

        let arm = table.get_weights_by_name("Arm").unwrap();

        let idx_rs = skeleton.joint_by_name("RightShoulder").unwrap() as usize;
        assert_eq!(arm[idx_rs / 4][idx_rs % 4], 0.5);

        let idx_lh = skeleton.joint_by_name("LeftHand").unwrap() as usize;
        assert_eq!(arm[idx_lh / 4][idx_lh % 4], 1.0);
    }

    #[test]
    fn test_load_joint_weights_arc() {
        let skeleton = load_girl_skeleton();
        let table =
            JointWeightsTable::from_path_arc(format!("{}/Girl/Girl.lw-json", TEST_ASSET_PATH), &*skeleton).unwrap();
        assert_eq!(table.weights().len(), 8);

        let no_arm = table.get_weights_by_name("NoArm").unwrap();

        let idx_hips = skeleton.joint_by_name("Hips").unwrap() as usize;
        assert_eq!(no_arm[idx_hips / 4][idx_hips % 4], 1.0);

        let idx_rs = skeleton.joint_by_name("RightShoulder").unwrap() as usize;
        assert_eq!(no_arm[idx_rs / 4][idx_rs % 4], 0.5);
    }
}
