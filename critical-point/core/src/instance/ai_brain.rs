use rustc_hash::FxBuildHasher;
use std::rc::Rc;

use crate::instance::ai_task::{InstAiTaskAny, assemble_ai_task};
use crate::template::{At, TmplAiBrain, TmplAiTaskSequence, TmplDatabase, TmplType};
use crate::utils::{DtHashMap, DtHashSet, ShapeSphere, ShapeSphericalCone, TmplID, XResult, xresf};

#[derive(Debug)]
pub struct InstAiBrain {
    pub tmpl_id: TmplID,
    pub character_npc: TmplID,
    pub alert_sphere: ShapeSphere,
    pub alert_cone: ShapeSphericalCone,
    pub aggro_sphere: ShapeSphere,
    pub aggro_lost_time: f32,
    pub tasks: DtHashMap<TmplID, Rc<dyn InstAiTaskAny>>,
    pub execute: bool,
}

impl InstAiBrain {
    pub(crate) fn new(db: &TmplDatabase, tmpl: At<TmplAiBrain>) -> XResult<Rc<InstAiBrain>> {
        let tasks = Self::collect_tasks(db, tmpl.clone())?;

        Ok(Rc::new(InstAiBrain {
            tmpl_id: tmpl.id,
            character_npc: tmpl.character_npc,
            alert_sphere: tmpl.alert_sphere,
            alert_cone: tmpl.alert_cone,
            aggro_sphere: tmpl.aggro_sphere,
            aggro_lost_time: tmpl.aggro_lost_time.to_native(),
            tasks,
            execute: tmpl.execute,
        }))
    }

    fn collect_tasks(db: &TmplDatabase, tmpl: At<TmplAiBrain>) -> XResult<DtHashMap<TmplID, Rc<dyn InstAiTaskAny>>> {
        let mut task_ids = DtHashSet::with_capacity_and_hasher(tmpl.tasks.len() * 2, FxBuildHasher);
        for &id in tmpl.tasks.iter() {
            if !task_ids.insert(id) {
                continue;
            }

            let task_tmpl = db.find(id)?;
            if let Ok(sequence) = task_tmpl.cast::<TmplAiTaskSequence>() {
                for &sub_id in sequence.tasks.iter() {
                    if !task_ids.insert(sub_id) {
                        continue;
                    }
                    debug_assert!(db.find(sub_id)?.typ() != TmplType::AiTaskSequence);
                }
            }
        }

        let mut tasks = DtHashMap::with_capacity_and_hasher(task_ids.len(), FxBuildHasher);
        for id in task_ids {
            tasks.insert(id, assemble_ai_task(db.find(id)?)?);
        }
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_new_inst_ai_brain() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db.find_as::<TmplAiBrain>(id!("AiBrain.InstanceNpc^1")).unwrap();

        let inst = InstAiBrain::new(&db, tmpl).unwrap();

        assert_eq!(inst.tmpl_id, id!("AiBrain.InstanceNpc^1"));
        assert_eq!(inst.character_npc, id!("CharacterNpc.InstanceNpc^1"));
        assert_eq!(inst.alert_sphere.radius, 5.0);
        assert_eq!(inst.alert_cone.radius, 10.0);
        assert_eq!(inst.alert_cone.half_angle, 45.0f32.to_radians());
        assert_eq!(inst.aggro_sphere.radius, 10.0);
        assert_eq!(inst.aggro_lost_time, 10.0);
        assert_eq!(inst.tasks.len(), 4);
        assert_eq!(inst.execute, true);
    }
}
