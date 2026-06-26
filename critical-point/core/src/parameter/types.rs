use critical_point_macros::csharp_in;
use glam::Vec3A;

use crate::utils::{TmplID, TmplIDLevel, TmplIDPlus};

#[csharp_in(Class)]
#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct ParamGame {
    pub zone: ParamZone,
    pub players: Vec<ParamPlayer>,
    #[serde(default)]
    pub npcs: Vec<ParamNpc>,
    #[serde(default)]
    pub local_mode: bool,
}

#[csharp_in(Class)]
#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct ParamPlayer {
    pub character: TmplID,
    pub style: TmplID,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub equipments: Vec<TmplIDLevel>,
    #[serde(default)]
    pub perks: Vec<TmplIDLevel>,
    #[serde(default)]
    pub accessories: Vec<ParamAccessory>,
    #[serde(default)]
    pub jewels: Vec<TmplIDPlus>,
    #[serde(default)]
    pub position: Vec3A,
}

#[csharp_in(Class)]
#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct ParamAccessory {
    pub id: TmplID,
    pub level: u32,
    pub entries: Vec<TmplID>,
}

#[csharp_in(Class)]
#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct ParamNpc {
    pub character: TmplID,
    #[serde(default)]
    pub level: u32,
    pub ai_brain: TmplID,
    #[serde(default)]
    pub position: Vec3A,
}

#[csharp_in(Class)]
#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct ParamZone {
    pub zone: TmplID,
}
