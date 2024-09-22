use crate::template::base::{TmplAny, TmplClass};
use crate::utils::StrID;

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplStage {
    pub id: StrID,
    pub name: String,
    pub asset_id: String,
}

#[typetag::deserialize(name = "Stage")]
impl TmplAny for TmplStage {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn class(&self) -> TmplClass {
        TmplClass::Stage
    }
}
