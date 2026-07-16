use crate::template::base::impl_tmpl;
use crate::utils::TmplID;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(tag = "T")]
pub enum TmplAiRoutineItem {
    Task { id: TmplID },
    If { script: u16, jump: u32 },
    Else { jump: u32 },
}

impl TmplAiRoutineItem {
    pub fn from_rkyv(r: &ArchivedTmplAiRoutineItem) -> TmplAiRoutineItem {
        match r {
            ArchivedTmplAiRoutineItem::Task { id } => TmplAiRoutineItem::Task { id: *id },
            ArchivedTmplAiRoutineItem::If { script, jump } => TmplAiRoutineItem::If {
                script: script.to_native(),
                jump: jump.to_native(),
            },
            ArchivedTmplAiRoutineItem::Else { jump } => TmplAiRoutineItem::Else { jump: jump.to_native() },
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiRoutine {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub tasks: Vec<TmplAiRoutineItem>,
}

impl TmplAiRoutine {
    #[inline]
    pub fn iter_tasks(&self) -> impl Iterator<Item = &TmplID> {
        self.tasks.iter().filter_map(|item| match item {
            TmplAiRoutineItem::Task { id } => Some(id),
            _ => None,
        })
    }
}

impl ArchivedTmplAiRoutine {
    #[inline]
    pub fn iter_tasks(&self) -> impl Iterator<Item = &TmplID> {
        self.tasks.iter().filter_map(|item| match item {
            ArchivedTmplAiRoutineItem::Task { id } => Some(id),
            _ => None,
        })
    }
}

impl_tmpl!(TmplAiRoutine, AiRoutine, "AiRoutine");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{LEVEL_MOVE, id};

    #[test]
    fn test_tmpl_ai_routine() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db.find_as::<TmplAiRoutine>(id!("AiRoutine.Enemy.Sequence")).unwrap();

        assert_eq!(tmpl.id, id!("AiRoutine.Enemy.Sequence"));
        assert_eq!(tmpl.character_npc, id!("CharacterNpc.Enemy"));
        assert_eq!(tmpl.tasks.len(), 3);
        assert_eq!(TmplAiRoutineItem::from_rkyv(&tmpl.tasks[0]), TmplAiRoutineItem::Task {
            id: id!("AiTask.Enemy.Idle")
        });
        assert_eq!(TmplAiRoutineItem::from_rkyv(&tmpl.tasks[1]), TmplAiRoutineItem::Task {
            id: id!("AiTask.Enemy.Patrol")
        });
        assert_eq!(TmplAiRoutineItem::from_rkyv(&tmpl.tasks[2]), TmplAiRoutineItem::Task {
            id: id!("AiTask.Enemy.MoveTo")
        });
    }
}
