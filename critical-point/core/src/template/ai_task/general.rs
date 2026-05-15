use crate::template::ai_task::base::TmplRepeatLimit;
use crate::template::base::impl_tmpl;
use crate::utils::{F32Range, TimeRange, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskGeneral {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub leave_level_moving: u16,
    pub keep_level_acting: u16,
    // pub repeat_limit: TmplRepeatLimit,
    // pub cold_down: f32,
    pub expected_distance: F32Range,
    pub moves: Vec<TmplAiTaskGeneralMove>, // TODO: verify in CheckBytes
    pub actions: Vec<TmplID>,
}

impl_tmpl!(TmplAiTaskGeneral, AiTaskGeneral, "AiTaskGeneral");

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskGeneralMove {
    #[serde(default)]
    pub action: TmplID,
    pub distance: F32Range,
}

impl TmplAiTaskGeneralMove {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplAiTaskGeneralMove) -> TmplAiTaskGeneralMove {
        TmplAiTaskGeneralMove {
            action: archived.action,
            distance: archived.distance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_ai_task_idle() {
        // let db = TmplDatabase::new(10240, 150).unwrap();

        // let task = db.find_as::<TmplAiTaskGeneral>(id!("AiTask.Enemy.KeepDistance")).unwrap();
        // assert_eq!(task.id, id!("AiTask.Enemy.KeepDistance"));
        // assert_eq!(task.character_npc, id!("CharacterNpc.Enemy"));
    }
}
