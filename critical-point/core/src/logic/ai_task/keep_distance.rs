use critical_point_macros::csharp_out;
use glam::Vec3;
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::hint::unlikely;
use std::rc::Rc;

use crate::instance::{InstActionDodgeNpc, InstAiTaskKeepDistance, InstCharacter};
use crate::logic::ai_task::base::{
    AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny, StateAiTaskBase, impl_state_ai_task,
};
use crate::logic::game::ContextUpdateEx;
use crate::utils::{AiTaskType, Castable, TmplID, XResult, circle_circle_intersection, extend, xres, xresf};

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateAiTaskKeepDistance {
    pub _base: StateAiTaskBase,
    pub dodge_dir: Vec2xz,
    pub dodge_pt: Vec3,
}

extend!(StateAiTaskKeepDistance, StateAiTaskBase);
impl_state_ai_task!(StateAiTaskKeepDistance, KeepDistance, "KeepDistance");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicAiTaskKeepDistance {
    _base: LogicAiTaskBase,
    inst: Rc<InstAiTaskKeepDistance>,
    inst_dodge: Rc<InstActionDodgeNpc>,
    dodge_dir: Vec2xz,
    dodge_pt: Vec3,
}

extend!(LogicAiTaskKeepDistance, LogicAiTaskBase);

impl LogicAiTaskKeepDistance {
    pub fn new(
        ctx: &mut ContextUpdateEx,
        inst_task: Rc<InstAiTaskKeepDistance>,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicAiTaskKeepDistance> {
        let inst_dodge = match inst_chara.actions.get(&inst_task.dodge_action) {
            Some(inst) => inst.clone().cast()?,
            None => return xresf!(InstNotFound; "id={}", inst_task.dodge_action),
        };
        Ok(LogicAiTaskKeepDistance {
            _base: LogicAiTaskBase::new(ctx.identity.gen_ai_task_id(), inst_task.clone()),
            inst: inst_task,
            inst_dodge,
            dodge_dir: Vec2xz::ZERO,
            dodge_pt: Vec3::ZERO,
        })
    }
}

unsafe impl LogicAiTaskAny for LogicAiTaskKeepDistance {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::KeepDistance
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskKeepDistance {
            _base: self._base.save(self.typ()),
            dodge_dir: self.dodge_dir,
            dodge_pt: self.dodge_pt,
        })
    }

    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()> {
        if state.id() != self._base.id {
            return xres!(LogicIDMismatch);
        }
        let state = state.cast::<StateAiTaskKeepDistance>()?;
        self._base.restore(&state._base);
        self.dodge_dir = state.dodge_dir;
        self.dodge_pt = state.dodge_pt;
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdateEx, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;
        self.intention = self.inst.intention;
        self.current_action = self.inst_dodge.tmpl_id;

        // Calculate dodge direction and point.
        let (dir, dist) = self.calc_move_dir(ctx, ctxt)?;
        self.dodge_dir = dir;
        let chara_pos = ctxt.chara_phy.position();
        let offset = glam::Vec3A::new(dir.x, 0.0, dir.z) * dist;
        self.dodge_pt = (chara_pos + offset).into();

        debug_assert!(self.dodge_dir.is_normalized());

        let mut ret = AiTaskReturn::default();
        ret.next_action = Some(self.inst_dodge.clone());
        ret.ai_move_dir = dir;
        ret.ai_move_dst_pos = chara_pos + offset;
        Ok(ret)
    }

    fn update(&mut self, ctx: &mut ContextUpdateEx, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.update(ctx, ctxt)?;

        // Action externally changed.
        let current_action = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.inst.tmpl_id,
            None => TmplID::INVALID,
        };
        if self.current_action != current_action {
            self.stop(ctx, ctxt)?;
            return Ok(AiTaskReturn::default());
        }

        // Action finished.
        let is_inactive = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.is_inactive(),
            None => true,
        };
        if is_inactive {
            self.stop(ctx, ctxt)?;
            return Ok(AiTaskReturn::default());
        }

        let mut ret = AiTaskReturn::default();
        ret.ai_move_dir = self.dodge_dir;
        ret.ai_move_dst_pos = self.dodge_pt.into();
        Ok(ret)
    }
}

impl LogicAiTaskKeepDistance {
    fn calc_move_dir(&self, ctx: &mut ContextUpdateEx, ctxt: &mut ContextAiTask) -> XResult<(Vec2xz, f32)> {
        let expected_dist = self.inst.expected_distance;
        let dodge_range = self.inst_dodge.move_distance;

        let tgt_pos = match ctxt.ai_thinking.target_chara_pos() {
            Some(pos) => Vec2xz::from_vec3a(pos),
            None => {
                // No target, move backward.
                let dir = -ctxt.chara_phy.direction_xz();
                return Ok((dir, self.inst_dodge.move_distance.min));
            }
        };

        let chara_pos = ctxt.chara_phy.position_xz();
        let current_dist = (tgt_pos - chara_pos).length();
        let dist_diff = (current_dist - expected_dist).abs();

        if unlikely(dist_diff < 1e-4) {
            return match current_dist <= expected_dist {
                true => Ok((-ctxt.chara_phy.direction_xz(), dodge_range.min)),
                false => Ok((ctxt.chara_phy.direction_xz(), dodge_range.min)),
            };
        }

        // Too close: move away directly.
        if current_dist <= expected_dist && dist_diff >= dodge_range.min {
            let dir = chara_pos - tgt_pos;
            // println!("Too close: dir={:?}, dist_diff={}", dir, dist_diff);
            Ok((dir.normalize(), dist_diff.min(dodge_range.max)))
        }
        // Too far: move toward directly.
        else if current_dist >= expected_dist && dist_diff >= dodge_range.min {
            let dir = tgt_pos - chara_pos;
            // println!("Too far: dir={:?}, dist_diff={}", dir, dist_diff);
            Ok((dir.normalize(), dist_diff.min(dodge_range.max)))
        }
        // dist_diff < dodge_range.min
        else {
            let (p1, p2) = match circle_circle_intersection(chara_pos, dodge_range.min, tgt_pos, expected_dist) {
                // When expected_dist > dodge_range.min (the typical case), the constraints
                // expected_dist - dodge_range.min < current_dist < expected_dist + dodge_range.min
                // guarantee that circle_circle_intersection always returns Some here.
                Some((p1, p2)) => (p1, p2),
                // Only reachable when expected_dist <= dodge_range.min (unusual config).
                // Use Pythagorean Theorem here.
                None => {
                    let hypotenuse = dodge_range.min * dodge_range.min;
                    let leg_a = (tgt_pos - chara_pos).length_squared();
                    let leg_b = (hypotenuse - leg_a).sqrt();
                    let perp = (tgt_pos - chara_pos).normalize().perp();
                    let p1 = tgt_pos + perp * leg_b;
                    let p2 = tgt_pos - perp * leg_b;
                    (p1, p2)
                }
            };
            let p = if ctx.rand.rand_f32() < 0.5 { p1 } else { p2 };
            let dir = (p - chara_pos).normalize();
            // println!(
            //     "Circle intersection: dir={:?}, dist_diff={} p1={:?}, p2={:?}",
            //     dir, dist_diff, p1, p2
            // );
            // println!("len = {}", (p1 - chara_pos).length());
            Ok((dir, dodge_range.min))
        }
    }
}
