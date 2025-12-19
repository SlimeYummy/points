use std::fmt;

use crate::template::variable::TmplVar;
use crate::utils::{TimeFragment, TmplID, VirtualKeyDir};

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
    pub key: VirtualKeyDir,
    pub level: u16,
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
pub struct TmplTimelineRange<T> {
    pub fragments: Vec<TimeFragment>,
    pub values: Vec<T>,
}

impl<T> Default for TmplTimelineRange<T> {
    fn default() -> Self {
        Self {
            fragments: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<T> fmt::Debug for ArchivedTmplTimelineRange<T>
where
    T: rkyv::Archive,
    T::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchivedTmplTimelineRange")
            .field("fragments", &self.fragments)
            .field("values", &self.values)
            .finish()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplTimelinePoint<T> {
    pub pairs: Vec<(f32, T)>,
}

impl<T> Default for TmplTimelinePoint<T> {
    fn default() -> Self {
        Self { pairs: Vec::new() }
    }
}

impl<T> fmt::Debug for ArchivedTmplTimelinePoint<T>
where
    T: rkyv::Archive,
    T::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchivedTmplTimelinePoint")
            .field("pairs", &self.pairs)
            .finish()
    }
}
