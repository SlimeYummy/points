use crate::template::{At, TmplAiRoutine, TmplAiRoutineItem};
use crate::utils::TmplID;

pub type InstAiRoutineItem = TmplAiRoutineItem;

#[repr(C)]
#[derive(Debug)]
pub struct InstAiRoutine {
    pub tmpl_id: TmplID,
    pub character_npc: TmplID,
    pub tasks: Vec<InstAiRoutineItem>,
}

impl InstAiRoutine {
    pub(crate) fn new(tmpl: At<TmplAiRoutine>) -> InstAiRoutine {
        InstAiRoutine {
            tmpl_id: tmpl.id,
            character_npc: tmpl.character_npc,
            tasks: tmpl.tasks.iter().map(InstAiRoutineItem::from_rkyv).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{LEVEL_ATTACK, id};

    #[test]
    fn test_new_inst_ai_routine() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db
            .find_as::<TmplAiRoutine>(id!("AiRoutine.InstanceNpc.Sequence^1"))
            .unwrap();
        let inst = InstAiRoutine::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiRoutine.InstanceNpc.Sequence^1"));
        assert_eq!(inst.character_npc, id!("CharacterNpc.InstanceNpc^1"));
        assert_eq!(inst.tasks.len(), 3);
        assert_eq!(inst.tasks[0], InstAiRoutineItem::Task {
            id: id!("AiTask.InstanceNpc.Idle^1")
        });
        assert_eq!(inst.tasks[1], InstAiRoutineItem::Task {
            id: id!("AiTask.InstanceNpc.Patrol^1")
        });
        assert_eq!(inst.tasks[2], InstAiRoutineItem::Task {
            id: id!("AiTask.InstanceNpc.MoveTo^1")
        });
    }
}
