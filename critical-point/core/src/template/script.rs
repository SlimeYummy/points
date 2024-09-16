use std::ops::Deref;
use std::rc::Rc;

use crate::script::ScriptBlocks;

#[derive(Debug, Clone, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplScript {
    pub id: u32,
    #[serde(flatten)]
    pub script: Rc<ScriptBlocks>,
}

impl Deref for TmplScript {
    type Target = ScriptBlocks;

    fn deref(&self) -> &Self::Target {
        &self.script
    }
}
