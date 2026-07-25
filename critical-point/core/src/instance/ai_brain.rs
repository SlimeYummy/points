use rustc_hash::FxBuildHasher;
use std::rc::Rc;

use crate::instance::ai_routine::InstAiRoutine;
use crate::instance::ai_task::{InstAiTaskAny, assemble_ai_task};
use crate::template::{At, TmplAiBrain, TmplAiRoutine, TmplDatabase, TmplType};
use crate::utils::{DtHashMap, DtHashSet, ShapeSphere, ShapeSphericalCone, TmplID, TmplPrefix, XResult, xresf};

#[derive(Debug)]
pub struct InstAiBrain {
    pub tmpl_id: TmplID,
    pub character_npc: TmplID,
    pub alert_sphere: ShapeSphere,
    pub alert_cone: ShapeSphericalCone,
    pub aggro_sphere: ShapeSphere,
    pub aggro_lost_time: f32,
    pub tasks: DtHashMap<TmplID, Rc<dyn InstAiTaskAny>>,
    pub routines: DtHashMap<TmplID, Rc<InstAiRoutine>>,
    pub execute: bool,
}

impl InstAiBrain {
    pub(crate) fn new(db: &TmplDatabase, tmpl: At<TmplAiBrain>) -> XResult<Rc<InstAiBrain>> {
        let (tasks, routines) = Self::collect_tasks_and_routines(db, tmpl.clone())?;

        Ok(Rc::new(InstAiBrain {
            tmpl_id: tmpl.id,
            character_npc: tmpl.character_npc,
            alert_sphere: tmpl.alert_sphere,
            alert_cone: tmpl.alert_cone,
            aggro_sphere: tmpl.aggro_sphere,
            aggro_lost_time: tmpl.aggro_lost_time.to_native(),
            tasks,
            routines,
            execute: tmpl.execute,
        }))
    }

    fn collect_tasks_and_routines(
        db: &TmplDatabase,
        tmpl: At<TmplAiBrain>,
    ) -> XResult<(
        DtHashMap<TmplID, Rc<dyn InstAiTaskAny>>,
        DtHashMap<TmplID, Rc<InstAiRoutine>>,
    )> {
        let mut ids = DtHashSet::with_capacity_and_hasher(tmpl.tasks.len() * 2, FxBuildHasher);
        for &id in tmpl.tasks.iter() {
            if !ids.insert(id) {
                continue;
            }

            let task_tmpl = db.find(id)?;
            if let Ok(routine) = task_tmpl.cast::<TmplAiRoutine>() {
                for &sub_id in routine.iter_tasks() {
                    debug_assert!(db.find(sub_id)?.typ() != TmplType::AiRoutine);
                    ids.insert(sub_id);
                }
            }
        }

        let mut tasks = DtHashMap::with_capacity_and_hasher(ids.len(), FxBuildHasher);
        let mut routines = DtHashMap::with_capacity_and_hasher(ids.len(), FxBuildHasher);
        for id in ids {
            if id.prefix == TmplPrefix::AiRoutine {
                let tmpl = db.find_as::<TmplAiRoutine>(id)?;
                routines.insert(id, Rc::new(InstAiRoutine::new(tmpl)));
            }
            else {
                tasks.insert(id, assemble_ai_task(db.find(id)?)?);
            }
        }
        Ok((tasks, routines))
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
