use rustc_hash::FxBuildHasher;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use crate::instance::ai_task::{InstAiTaskAny, assemble_ai_task};
use crate::template::{ArchivedAiNode, ArchivedAiNodeBranch, At, TmplAiBrain, TmplDatabase};
use crate::utils::{DtHashMap, ShapeSphere, ShapeSphericalCone, TmplID, XResult, xresf};

#[derive(Debug, Clone)]
struct NodeTask {
    condition: [u32; 2],
    task: Rc<dyn InstAiTaskAny>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct NodeBranch {
    condition: [u32; 2],
    task: [u32; 2],
    branch: [u32; 2],
}

#[derive(Debug)]
pub struct InstAiBrain {
    pub tmpl_id: TmplID,
    pub character: TmplID,
    pub alert_sphere: ShapeSphere,
    pub alert_cone: ShapeSphericalCone,
    pub attack_exit_delay: f32,

    conditions: Vec<()>,
    tasks: Vec<NodeTask>,
    branches: Vec<NodeBranch>,
    idle_pos: u32,
}

impl InstAiBrain {
    pub(crate) fn new(db: &TmplDatabase, tmpl: At<TmplAiBrain>) -> XResult<Rc<InstAiBrain>> {
        let mut inst = InstAiBrain {
            tmpl_id: tmpl.id,
            character: tmpl.character,
            alert_sphere: tmpl.alert_sphere,
            alert_cone: tmpl.alert_cone,
            attack_exit_delay: tmpl.attack_exit_delay.to_native(),
            conditions: Vec::with_capacity(32),
            tasks: Vec::with_capacity(32),
            branches: Vec::with_capacity(8),
            idle_pos: 0,
        };

        let mut task_map = DtHashMap::with_capacity_and_hasher(32, FxBuildHasher);

        if !tmpl.idle.conditions.is_empty() {
            return xresf!(BadAsset; "tmpl_id={}, idle condition", tmpl.id);
        }
        inst.idle_pos = inst.branches.len() as u32;
        inst.branches.push(NodeBranch::default());
        inst.init_recursive(db, &mut task_map, &tmpl.idle, inst.idle_pos as usize)?;

        Ok(Rc::new(inst))
    }

    fn init_recursive(
        &mut self,
        db: &TmplDatabase,
        task_map: &mut DtHashMap<TmplID, Rc<dyn InstAiTaskAny>>,
        archived_branch: &ArchivedAiNodeBranch,
        branch_pos: usize,
    ) -> XResult<()> {
        let new_branch_start = self.branches.len() as u32;
        let mut new_branch_count = 0;

        self.branches[branch_pos] = NodeBranch {
            condition: [self.conditions.len() as u32; 2],
            task: [self.tasks.len() as u32; 2],
            branch: [new_branch_start; 2],
        };

        // Collect tasks and allocate slots for direct sub-branches.
        // This ensures direct sub-branches are contiguous in the vec.
        for (idx, node) in archived_branch.nodes.iter().enumerate() {
            if let ArchivedAiNode::Task(archived) = node {
                // Get or create task
                let inst_task = match task_map.entry(archived.task) {
                    Entry::Occupied(entry) => entry.get().clone(),
                    Entry::Vacant(entry) => {
                        let tmpl_task = db.find(archived.task)?;
                        let inst_task = assemble_ai_task(tmpl_task)?;
                        entry.insert(inst_task.clone());
                        inst_task
                    }
                };
                self.branches[branch_pos].task[1] += 1;

                // Init condition
                // TODO: ...

                self.tasks.push(NodeTask {
                    condition: [self.conditions.len() as u32; 2],
                    task: inst_task,
                });
            }
            else {
                new_branch_count += 1;
                self.branches.push(NodeBranch::default());
                // Temporary store archived node index in new branch.
                self.branches.last_mut().unwrap().branch[0] = idx as u32;
            }
        }

        self.branches[branch_pos].branch[1] += new_branch_count;

        // Recursively initialize sub-branches already allocated.
        // Note: Recursion will extend self.branches further.
        for i in 0..new_branch_count {
            let sub_branch_idx = (new_branch_start + i) as usize;

            // Retrieve the archived node by index we stored previously.
            let archived_idx = self.branches[sub_branch_idx].branch[0] as usize;
            if let ArchivedAiNode::Branch(sub_archived_branch) = &archived_branch.nodes[archived_idx] {
                self.init_recursive(db, task_map, sub_archived_branch, sub_branch_idx)?;
            }
            else {
                return xresf!(Unexpected; "tmpl_id={}, invalid branch", self.tmpl_id);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum InstAiNode<'t> {
    Task(&'t [()], &'t Rc<dyn InstAiTaskAny>),
    Branch(&'t [()]),
}

impl InstAiBrain {
    #[inline]
    pub fn travel_idle<F>(&self, visitor: F) -> XResult<()>
    where
        F: FnMut(&InstAiNode) -> XResult<()>,
    {
        self.travel(self.idle_pos, visitor)
    }

    fn travel<F>(&self, start_idx: u32, mut visitor: F) -> XResult<()>
    where
        F: FnMut(&InstAiNode) -> XResult<()>,
    {
        let task_range = self.branches[start_idx as usize].task;
        for task in self.task_slice(task_range) {
            let cond_range = task.condition;
            visitor(&InstAiNode::Task(&self.condition_slice(cond_range), &task.task))?;
        }

        let branch_range = self.branches[start_idx as usize].branch;
        for branch in self.branch_slice(branch_range) {
            let cond_range = branch.condition;
            visitor(&InstAiNode::Branch(&self.condition_slice(cond_range)))?;
        }

        Ok(())
    }

    #[inline]
    fn branch_slice(&self, range: [u32; 2]) -> &[NodeBranch] {
        &self.branches[range[0] as usize..range[1] as usize]
    }

    #[inline]
    fn task_slice(&self, range: [u32; 2]) -> &[NodeTask] {
        &self.tasks[range[0] as usize..range[1] as usize]
    }

    #[inline]
    fn condition_slice(&self, range: [u32; 2]) -> &[()] {
        &self.conditions[range[0] as usize..range[1] as usize]
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
        let tmpl = db.find_as::<TmplAiBrain>(id!("AiBrain.NpcInstance^1")).unwrap();

        let inst = InstAiBrain::new(&db, tmpl).unwrap();

        assert_eq!(inst.tmpl_id, id!("AiBrain.NpcInstance^1"));
        assert_eq!(inst.character, id!("NpcCharacter.NpcInstance^1"));
        assert_eq!(inst.alert_sphere.radius, 5.0);
        assert_eq!(inst.alert_cone.radius, 10.0);
        assert_eq!(inst.alert_cone.half_angle, 45.0f32.to_radians());
        assert_eq!(inst.attack_exit_delay, 30.0);

        let mut task_ids = vec![];
        inst.travel_idle(|node| {
            match node {
                InstAiNode::Task(_, task) => {
                    task_ids.push(task.tmpl_id);
                }
                InstAiNode::Branch(_) => {}
            };
            Ok(())
        })
        .unwrap();

        assert_eq!(task_ids.len(), 2);
        assert_eq!(task_ids[0], id!("AiTask.NpcInstance.Idle^1"));
        assert_eq!(task_ids[1], id!("AiTask.NpcInstance.Patrol^1"));
    }
}
