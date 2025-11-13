use std::fmt;

use crate::template::variable::TmplVar;
use crate::utils::{TimeFragment, TmplID, VirtualDir, VirtualKey};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAnimation {
    pub files: String,
    pub local_id: u16,
    pub duration: f32,
    pub fade_in: f32,
    pub root_motion: bool,
    pub weapon_motion: bool,
}

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TmplDeriveRule {
    pub key: VirtualKey,
    pub dir: Option<VirtualDir>,
    pub action: TmplVar<TmplID>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionAttributes {
    pub damage_rdc: TmplVar<f32>,
    pub shield_dmg_rdc: TmplVar<f32>,
    pub poise_level: TmplVar<u16>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplTimeline<T> {
    pub fragments: Vec<TimeFragment>,
    pub values: Vec<T>,
}

impl<T> fmt::Debug for ArchivedTmplTimeline<T>
where
    T: rkyv::Archive,
    T::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchivedTmplTimeline")
            .field("fragments", &self.fragments)
            .field("values", &self.values)
            .finish()
    }
}
