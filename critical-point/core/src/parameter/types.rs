use cirtical_point_csgen::CsIn;

use crate::utils::{TmplID, TmplIDLevel, TmplIDPlus};

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
pub struct ParamZone {
    pub zone: TmplID,
}
