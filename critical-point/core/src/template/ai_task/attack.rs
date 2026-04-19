use crate::template::base::impl_tmpl;
use crate::utils::TmplID;

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskAttack {
    pub id: TmplID,
    pub character: TmplID,
}

impl_tmpl!(TmplAiTaskAttack, AiTaskAttack, "AiTaskAttack");
