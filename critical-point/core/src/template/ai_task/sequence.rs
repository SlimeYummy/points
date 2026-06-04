use crate::template::base::impl_tmpl;
use crate::utils::TmplID;

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskSequence {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub tasks: Vec<TmplID>,
}

impl_tmpl!(TmplAiTaskSequence, AiTaskSequence, "AiTaskSequence");
