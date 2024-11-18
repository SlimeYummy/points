use crate::template::base::{TmplAny, TmplType};
use crate::utils::StrID;

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplStage {
    pub id: StrID,
    pub name: String,
    pub stage_file: String,
    pub view_stage_file: String,
}

#[typetag::deserialize(name = "Stage")]
impl TmplAny for TmplStage {
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
    use crate::template::database::TmplDatabase;
    use crate::utils::s;

    #[test]
    fn test_load_stage() {
        let db = TmplDatabase::new("../test-res").unwrap();
        let stage = db.find_as::<TmplStage>(&s!("Stage.Demo")).unwrap();
        assert_eq!(stage.id, "Stage.Demo");
        assert_eq!(stage.name, "Demo");
        assert_eq!(stage.stage_file, "stage-demo.json");
        assert_eq!(stage.view_stage_file, "stage-demo.tscn");
    }
}
