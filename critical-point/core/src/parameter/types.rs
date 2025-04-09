use cirtical_point_csgen::CsIn;

use crate::utils::{IDLevel2, IDPlus2, StrID};

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
pub struct ParamPlayer {
    pub character: StrID,
    pub style: StrID,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub equipments: Vec<IDLevel2>,
    #[serde(default)]
    pub perks: Vec<StrID>,
    #[serde(default)]
    pub accessories: Vec<ParamAccessory>,
    #[serde(default)]
    pub jewels: Vec<IDPlus2>,
}

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
pub struct ParamAccessory {
    pub id: StrID,
    pub level: u32,
    pub entries: Vec<StrID>,
}

#[derive(
    Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize, CsIn,
)]
#[cs_attr(Class)]
pub struct ParamStage {
    pub stage: StrID,
}
