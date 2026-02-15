use critical_point_csgen::CsIn;
use glam::Vec3A;

use crate::utils::{TmplID, TmplIDLevel, TmplIDPlus};

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
pub struct ParamGame {
    pub zone: ParamZone,
    pub players: Vec<ParamPlayer>,
    #[serde(default)]
    pub npcs: Vec<ParamNpc>,
    #[serde(default)]
    pub local_mode: bool,
}

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
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

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
pub struct ParamAccessory {
    pub id: TmplID,
    pub level: u32,
    pub entries: Vec<TmplID>,
}

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
pub struct ParamNpc {
    pub character: TmplID,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub position: Vec3A,
}

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
pub struct ParamZone {
    pub zone: TmplID,
}
