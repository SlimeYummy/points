use crate::template::base::{TmplAny, TmplType};
use crate::utils::StrID;

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplZone {
    pub id: StrID,
    pub name: String,
    pub stage_file: String,
    pub view_stage_file: String,
}

#[typetag::deserialize(name = "Stage")]
impl TmplAny for TmplZone {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::Stage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_TEMPLATE_PATH;
    use crate::template::database::TmplDatabase;
    use crate::utils::sb;

    #[test]
    fn test_load_stage() {
        let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();
        let stage = db.find_as::<TmplZone>(&sb!("Stage.Demo")).unwrap();
        assert_eq!(stage.id, "Stage.Demo");
        assert_eq!(stage.name, "Demo");
        assert_eq!(stage.stage_file, "stage-demo.json");
        assert_eq!(stage.view_stage_file, "stage-demo.tscn");
    }
}
