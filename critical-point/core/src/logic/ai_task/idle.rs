use critical_point_csgen::{CsEnum, CsOut};
use glam::{Vec3, Vec3A, Vec3Swizzles};
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use crate::instance::{InstActionIdle, InstActionMoveNpc, InstAiTaskIdle, InstAiTaskIdleStep, InstCharacter};
use crate::logic::ai_task::base::{
    AiBrainPurpose, AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny, StateAiTaskBase,
    impl_state_ai_task,
};
use crate::logic::game::ContextUpdate;
use crate::utils::{AiTaskType, Castable, TmplID, XResult, extend, loose_le, strict_lt, xres, xresf};

const THRESHOLD_XZ_RATIO_MOVE: f32 = 0.25;
const THRESHOLD_XZ_RATIO_IDLE: f32 = 1.0;
const THRESHOLD_Y_DISTANCE: f32 = 1.0;

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsEnum,
)]
#[rkyv(derive(Debug))]
pub enum AiTaskIdleMode {
    Idle,
    Move,
    MoveStop,
}

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateAiTaskIdle {
    pub _base: StateAiTaskBase,
    pub mode: AiTaskIdleMode,
    pub step_idx: u32,
    pub idle_timer: f32,
    pub path_idx: u32,
    #[cs_hide(8, 8)]
    pub move_path: Option<Arc<Vec<Vec3>>>, // TODO: Optimize
}

extend!(StateAiTaskIdle, StateAiTaskBase);
impl_state_ai_task!(StateAiTaskIdle, Idle, "Idle");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicAiTaskIdle {
    _base: LogicAiTaskBase,
    inst: Rc<InstAiTaskIdle>,
    inst_idle: Rc<InstActionIdle>,
    inst_move: Rc<InstActionMoveNpc>,

    mode: AiTaskIdleMode,
    step_idx: u32,
    idle_timer: f32,
    path_idx: u32,
    move_path: Option<Arc<Vec<Vec3>>>,
}

extend!(LogicAiTaskIdle, LogicAiTaskBase);

impl LogicAiTaskIdle {
    pub fn new(
        ctx: &mut ContextUpdate,
        inst_task: Rc<InstAiTaskIdle>,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicAiTaskIdle> {
        let inst_idle: Rc<InstActionIdle> = match inst_chara.actions.get(&inst_task.action_idle) {
            Some(inst) => inst.clone().cast()?,
            None => return xres!(InstNotFound, inst_task.action_idle),
        };
        let inst_move: Rc<InstActionMoveNpc> = match inst_chara.actions.get(&inst_task.action_move) {
            Some(inst) => inst.clone().cast()?,
            None => return xres!(InstNotFound, inst_task.action_move),
        };
        Ok(LogicAiTaskIdle {
            _base: LogicAiTaskBase::new(ctx.gene.gen_ai_task_id(), inst_task.clone()),
            inst: inst_task,
            inst_idle,
            inst_move,

            mode: AiTaskIdleMode::Idle,
            step_idx: u32::MAX,
            idle_timer: 0.0,
            path_idx: 0,
            move_path: None,
        })
    }
}

unsafe impl LogicAiTaskAny for LogicAiTaskIdle {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::Idle
    }

    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()> {
        if state.id() != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id(), self._base.id);
        }
        let state = state.cast::<StateAiTaskIdle>()?;
        self._base.restore(&state._base);
        self.mode = state.mode;
        self.step_idx = state.step_idx;
        self.idle_timer = state.idle_timer;
        self.path_idx = state.path_idx;
        self.move_path = state.move_path.clone();
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;
        self.step_idx = self.inst.route.len() as u32 - 1;
        self.enter_next(ctx, ctxt)
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.update(ctx, ctxt)?;

        // Action externally changed, stop task.
        let current_action = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.inst.tmpl_id,
            None => TmplID::INVALID,
        };
        if self.current_action != current_action {
            self.stop(ctx, ctxt)?;
            return Ok(AiTaskReturn::default());
        }

        let ret = match self.mode {
            AiTaskIdleMode::Idle => self.update_idle(ctx, ctxt)?,
            AiTaskIdleMode::Move => self.update_move(ctx, ctxt)?,
            AiTaskIdleMode::MoveStop => Some(self.update_move_stop(ctx, ctxt)?),
        };
        match ret {
            Some(ret) => Ok(ret),
            None => self.enter_next(ctx, ctxt),
        }
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskIdle {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            step_idx: self.step_idx,
            idle_timer: self.idle_timer,
            path_idx: self.path_idx,
            move_path: self.move_path.clone(),
        })
    }
}

impl LogicAiTaskIdle {
    #[inline]
    fn init_mode(&mut self, mode: AiTaskIdleMode) {
        self.mode = mode;
        self.idle_timer = 0.0;
        self.path_idx = 0;
        if mode != AiTaskIdleMode::Move {
            self.move_path = None;
        }
    }

    /// Some() => keep current mode.
    /// None => switch mode.
    fn update_idle(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let mut ret = AiTaskReturn::default();

        let duration = match self.inst.route[self.step_idx as usize] {
            InstAiTaskIdleStep::Idle(duration) => duration,
            InstAiTaskIdleStep::Move(_) => unreachable!(),
        };

        self.idle_timer += ctxt.time_step;
        if duration >= 0.0 && strict_lt!(self.idle_timer, duration) {
            if self.current_action != self.inst_idle.tmpl_id {
                ret.next_action = Some(self.inst_idle.clone());
            }
            Ok(Some(ret))
        }
        else {
            Ok(None)
        }
    }

    /// Some() => keep current mode.
    /// None => switch mode.
    fn update_move(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let mut ret = AiTaskReturn::default();

        let dst_pos = match self.inst.route[self.step_idx as usize] {
            InstAiTaskIdleStep::Move(point) => point,
            InstAiTaskIdleStep::Idle(_) => unreachable!(),
        };

        let real_dst_pos = match self.move_path.as_ref().and_then(|p| p.last()) {
            Some(pos) => Vec3A::from(*pos),
            None => dst_pos,
        };

        let threshold_xz = self.guess_threshold_xz();
        if !Self::is_reached(ctxt.chara_phy.position(), real_dst_pos, threshold_xz) {
            ret.thinking.purpose = AiBrainPurpose::ToLocation;
            ret.thinking.dst_point = real_dst_pos;
            ret.thinking.move_dir = self.calc_move_dir(ctxt.chara_phy.position());
            if self.current_action != self.inst_move.tmpl_id {
                ret.next_action = Some(self.inst_move.clone());
            }
            Ok(Some(ret))
        }
        else {
            println!("reached");
            Ok(None)
        }
    }

    fn update_move_stop(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        let mut ret = AiTaskReturn::default();

        let is_inactive = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.is_inactive(),
            None => true,
        };
        if is_inactive {
            self.init_mode(AiTaskIdleMode::Idle);
            if self.current_action != self.inst_idle.tmpl_id {
                self.current_action = self.inst_idle.tmpl_id;
                ret.next_action = Some(self.inst_idle.clone());
            }
        }
        Ok(ret)
    }

    fn enter_next(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        let mut ret = AiTaskReturn::default();

        for _ in 0..self.inst.route.len() {
            self.step_idx = (self.step_idx + 1) % (self.inst.route.len() as u32);
            match self.inst.route[self.step_idx as usize] {
                InstAiTaskIdleStep::Move(point) => {
                    if Self::is_reached(ctxt.chara_phy.position(), point, self.inst_move.min_distance) {
                        continue;
                    }
                    else {
                        if !self.update_move_path(ctxt, ctxt.chara_phy.position(), point)? {
                            continue;
                        }
                        self.init_mode(AiTaskIdleMode::Move);
                        if self.current_action != self.inst_move.tmpl_id {
                            self.current_action = self.inst_move.tmpl_id;
                            ret.next_action = Some(self.inst_move.clone());
                        }
                        ret.thinking.purpose = AiBrainPurpose::ToLocation;
                        ret.thinking.dst_point = point;
                        ret.thinking.move_dir = self.calc_move_dir(ctxt.chara_phy.position());
                        return Ok(ret);
                    }
                }
                InstAiTaskIdleStep::Idle(duration) => {
                    if loose_le!(duration, 0.0) {
                        continue;
                    }
                    if self.mode == AiTaskIdleMode::Move {
                        self.init_mode(AiTaskIdleMode::MoveStop);
                        if self.current_action != self.inst_move.tmpl_id {
                            self.current_action = self.inst_move.tmpl_id;
                            ret.next_action = Some(self.inst_move.clone());
                        }
                    }
                    else {
                        self.init_mode(AiTaskIdleMode::Idle);
                        if self.current_action != self.inst_idle.tmpl_id {
                            self.current_action = self.inst_idle.tmpl_id;
                            ret.next_action = Some(self.inst_idle.clone());
                        }
                    }
                    return Ok(ret);
                }
            }
        }

        self.stop(ctx, ctxt)?;
        return Ok(AiTaskReturn::default());
    }

    #[inline]
    fn update_move_path(&mut self, ctxt: &ContextAiTask, src_pos: Vec3A, dst_pos: Vec3A) -> XResult<bool> {
        let mut move_path = Vec::new();
        ctxt.zone.find_path(src_pos, dst_pos, &mut move_path)?;
        if !move_path.is_empty() {
            self.move_path = Some(Arc::new(move_path));
            Ok(true)
        }
        else {
            self.move_path = None;
            Ok(false)
        }
    }

    fn calc_move_dir(&mut self, src_pos: Vec3A) -> Vec2xz {
        let move_path = self.move_path.as_ref().map(|p| p.as_slice()).unwrap_or(&[]);
        let mut waypoint = Vec3A::ZERO;
        while self.path_idx < move_path.len() as u32 {
            waypoint = Vec3A::from(move_path[self.path_idx as usize]);
            if !Self::is_reached(src_pos, waypoint, self.inst_move.step_length * THRESHOLD_XZ_RATIO_MOVE) {
                break;
            }
            self.path_idx += 1;
        }

        if self.path_idx < move_path.len() as u32 {
            return Self::calc_dir_xz(src_pos, waypoint);
        }
        Vec2xz::ZERO
    }

    #[inline]
    fn is_reached(src_pos: Vec3A, dst_pos: Vec3A, threshold_xz: f32) -> bool {
        // TODO: Arrival judgment based on pathfinding routes.

        let dxz = src_pos.xz() - dst_pos.xz();
        let dy = src_pos.y - dst_pos.y;
        let reached_xz = loose_le!(dxz.length_squared(), threshold_xz * threshold_xz);
        let reached_y = loose_le!(dy.abs(), THRESHOLD_Y_DISTANCE);
        reached_xz && reached_y
    }

    #[inline]
    fn guess_threshold_xz(&self) -> f32 {
        let next_idx = (self.step_idx + 1) % (self.inst.route.len() as u32);
        match self.inst.route[next_idx as usize] {
            InstAiTaskIdleStep::Move(_) => self.inst_move.step_length * THRESHOLD_XZ_RATIO_MOVE,
            InstAiTaskIdleStep::Idle(_) => self.inst_move.step_length * THRESHOLD_XZ_RATIO_IDLE,
        }
    }

    #[inline]
    fn calc_dir_xz(src_pos: Vec3A, dst_pos: Vec3A) -> Vec2xz {
        (Vec2xz::from_vec3a(dst_pos) - Vec2xz::from_vec3a(src_pos)).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::test_utils::TestEnv;
    use crate::template::TmplAiTaskIdle;
    use crate::utils::id;

    // #[test]
    // fn test_logic_idle_restore() {
    //     let mut tenv = TestEnv::new().unwrap();
    //     let tmpl_task = tenv
    //         .systems
    //         .tmpl_db
    //         .find_as::<TmplAiTaskIdle>(id!("AiTask.InstanceNpc.Idle^1"))
    //         .unwrap();
    //     let inst_task = Rc::new(InstAiTaskIdle::new(tmpl_task));

    //     let param_npc = crate::parameter::ParamNpc {
    //         character: id!("Character.Human"),
    //         level: 1,
    //         ..Default::default()
    //     };
    //     let mut ctx = tenv.context_update();
    //     let inst_chara = InstCharacter::new_npc(&mut ctx.context_assemble(), &param_npc).unwrap();

    //     let mut logic = LogicAiTaskIdle::new(&mut ctx, inst_task.clone(), inst_chara.clone()).unwrap();

    //     let path = vec![
    //         Vec3::new(1.0, 0.0, 1.0),
    //         Vec3::new(2.0, 0.0, 2.0),
    //         Vec3::new(3.0, 0.0, 3.0),
    //     ];
    //     logic.move_path = path.clone();
    //     logic.path_idx = 1;
    //     logic.mode = AiTaskIdleMode::Move;

    //     let state = logic.save();
    //     let mut logic2 = LogicAiTaskIdle::new(&mut ctx, inst_task, inst_chara).unwrap();

    //     logic2.restore(state.as_ref()).unwrap();

    //     assert_eq!(logic2.move_path, path);
    //     assert_eq!(logic2.path_idx, 1);
    //     assert_eq!(logic2.mode, AiTaskIdleMode::Move);
    // }
}
