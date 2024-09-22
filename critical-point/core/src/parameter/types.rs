use cirtical_point_csgen::CsGen;

use crate::utils::{IDLevel, IDPlus, StrID};

#[derive(
    Debug,
    Default,
    Clone,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    CsGen,
)]
#[cs_attr(Cs, Class)]
pub struct ParamPlayer {
    pub character: StrID,
    pub style: StrID,
    pub level: u32,
    pub equipments: Vec<IDLevel>,
    pub accessories: Vec<ParamAccessory>,
    pub jewels: Vec<IDPlus>,
    pub perks: Vec<StrID>,
}

#[derive(
    Debug,
    Default,
    Clone,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    CsGen,
)]
#[cs_attr(Cs, Class)]
pub struct ParamAccessory {
    pub id: StrID,
    pub level: u32,
    pub entries: Vec<StrID>,
}

#[derive(
    Debug,
    Default,
    Clone,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    CsGen,
)]
#[cs_attr(Cs, Class)]
pub struct ParamStage {
    pub stage: StrID,
}
