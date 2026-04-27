use critical_point_csgen::{CsEnum, CsOut};
use glam::{Vec3, Vec3A, Vec3Swizzles};
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::{InstActionIdle, InstActionMove, InstAiTaskPatrol, InstAiTaskPatrolStep, InstCharacter};
use crate::logic::ai_task::base::{
    AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny, StateAiTaskBase, impl_state_ai_task,
};
use crate::logic::game::ContextUpdate;
use crate::logic::system::input::{InputMoveSpeed, WorldMoveState};
use crate::utils::{AiTaskType, Castable, XResult, extend, loose_le, strict_lt, xres, xresf};

const DISTANCE_XZ_THRESHOLD: f32 = 0.1;
const DISTANCE_XZ_THRESHOLD_SQ: f32 = DISTANCE_XZ_THRESHOLD * DISTANCE_XZ_THRESHOLD;
const DISTANCE_Y_THRESHOLD: f32 = 1.0;

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
pub enum AiTaskPatrolMode {
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
pub struct StateAiTaskPatrol {
    pub _base: StateAiTaskBase,
    pub mode: AiTaskPatrolMode,
    pub step_idx: u32,
    pub idle_timer: f32,
    pub path_idx: u32,
}

extend!(StateAiTaskPatrol, StateAiTaskBase);
impl_state_ai_task!(StateAiTaskPatrol, Patrol, "Patrol");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicAiTaskPatrol {
    _base: LogicAiTaskBase,
    inst: Rc<InstAiTaskPatrol>,
    inst_idle: Rc<InstActionIdle>,
    inst_move: Rc<InstActionMove>,

    mode: AiTaskPatrolMode,
    step_idx: u32,
    idle_timer: f32,
    path_idx: u32,
    move_path: Vec<Vec3>,
}

extend!(LogicAiTaskPatrol, LogicAiTaskBase);

impl LogicAiTaskPatrol {
    pub fn new(
        ctx: &mut ContextUpdate,
        inst_task: Rc<InstAiTaskPatrol>,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicAiTaskPatrol> {
        let inst_idle: Rc<InstActionIdle> = match inst_chara.actions.get(&inst_task.action_idle) {
            Some(inst) => inst.clone().cast()?,
            None => return xres!(InstNotFound, inst_task.action_idle),
        };
        let inst_move: Rc<InstActionMove> = match inst_chara.actions.get(&inst_task.action_move) {
            Some(inst) => inst.clone().cast()?,
            None => return xres!(InstNotFound, inst_task.action_move),
        };
        Ok(LogicAiTaskPatrol {
            _base: LogicAiTaskBase::new(ctx.gene.gen_ai_task_id(), inst_task.clone()),
            inst: inst_task,
            inst_idle,
            inst_move,

            mode: AiTaskPatrolMode::Idle,
            step_idx: 0,
            idle_timer: 0.0,
            path_idx: 0,
            move_path: Vec::with_capacity(128),
        })
    }
}

unsafe impl LogicAiTaskAny for LogicAiTaskPatrol {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::Patrol
    }

    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()> {
        if state.id() != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id(), self._base.id);
        }
        let state = state.cast::<StateAiTaskPatrol>()?;
        self._base.restore(&state._base);
        self.mode = state.mode;
        self.step_idx = state.step_idx;
        self.idle_timer = state.idle_timer;
        self.path_idx = state.path_idx;
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;
        self.step_idx = self.inst.route.len() as u32 - 1;
        self.enter_next(ctx, ctxt)
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.update(ctx, ctxt)?;

        let ret = match self.mode {
            AiTaskPatrolMode::Idle => self.update_idle(ctx, ctxt)?,
            AiTaskPatrolMode::Move => self.update_move(ctx, ctxt)?,
            AiTaskPatrolMode::MoveStop => self.update_move_stop(ctx, ctxt)?,
        };
        match ret {
            Some(ret) => Ok(ret),
            None => self.enter_next(ctx, ctxt),
        }
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskPatrol {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            step_idx: self.step_idx,
            idle_timer: self.idle_timer,
            path_idx: self.path_idx,
        })
    }
}

impl LogicAiTaskPatrol {
    fn update_idle(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let inst_task = self.inst.clone();
        let current_act = ctxt.chara_ctrl.current_action();
        let current_act_id = current_act.map(|act| act.inst.tmpl_id).unwrap_or_default();

        let mut ret = AiTaskReturn::default();

        if current_act_id != self.inst_idle.tmpl_id {
            self.stop(ctx, ctxt)?;
            return Ok(Some(ret));
        }

        let duration = match inst_task.route[self.step_idx as usize] {
            InstAiTaskPatrolStep::Idle(duration) => duration,
            InstAiTaskPatrolStep::Move(_) => unreachable!(),
        };

        self.idle_timer += ctxt.time_step;
        if strict_lt!(self.idle_timer, duration) {
            if current_act_id != self.inst_idle.tmpl_id {
                ret.next_action = Some(self.inst_idle.clone());
            }
            Ok(Some(ret))
        }
        else {
            Ok(None)
        }
    }

    fn update_move(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let inst_task = self.inst.clone();
        let current_act = ctxt.chara_ctrl.current_action();
        let current_act_id = current_act.map(|act| act.inst.tmpl_id).unwrap_or_default();

        let mut ret = AiTaskReturn::default();

        if current_act_id != self.inst_move.tmpl_id {
            self.stop(ctx, ctxt)?;
            return Ok(Some(ret));
        }

        let dst_pos = match inst_task.route[self.step_idx as usize] {
            InstAiTaskPatrolStep::Move(point) => point,
            InstAiTaskPatrolStep::Idle(_) => unreachable!(),
        };

        let real_dst_pos = match self.move_path.last() {
            Some(pos) => Vec3A::from(*pos),
            None => dst_pos,
        };

        if !Self::is_reached(ctxt.chara_phy.position(), real_dst_pos) {
            ret.world_move = self.calc_world_move(ctxt.chara_phy.position());
            if current_act_id != self.inst_move.tmpl_id {
                ret.next_action = Some(self.inst_move.clone());
            }
            Ok(Some(ret))
        }
        else {
            Ok(None)
        }
    }

    fn update_move_stop(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let current_act = ctxt.chara_ctrl.current_action();
        let current_act_id = current_act.map(|act| act.inst.tmpl_id).unwrap_or_default();

        let mut ret = AiTaskReturn::default();

        if current_act_id != self.inst_move.tmpl_id {
            self.stop(ctx, ctxt)?;
            return Ok(Some(ret));
        }

        if current_act.unwrap().is_inactive() {
            self.switch_mode(AiTaskPatrolMode::Idle);
            if current_act_id != self.inst_idle.tmpl_id {
                ret.next_action = Some(self.inst_idle.clone());
            }
        }
        Ok(Some(ret))
    }

    fn enter_next(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        let current_act = ctxt.chara_ctrl.current_action();
        let current_act_id = current_act.map(|act| act.inst.tmpl_id).unwrap_or_default();

        let mut ret = AiTaskReturn::default();

        for _ in 0..self.inst.route.len() {
            self.step_idx = (self.step_idx + 1) % (self.inst.route.len() as u32);
            match self.inst.route[self.step_idx as usize] {
                InstAiTaskPatrolStep::Move(point) => {
                    if Self::is_reached(ctxt.chara_phy.position(), point) {
                        continue;
                    }
                    else {
                        if !self.update_move_path(ctxt, ctxt.chara_phy.position(), point)? {
                            continue;
                        }
                        self.switch_mode(AiTaskPatrolMode::Move);
                        if current_act_id != self.inst_move.tmpl_id {
                            ret.next_action = Some(self.inst_move.clone());
                            ret.quick_switch = true;
                        }
                        ret.world_move = self.calc_world_move(ctxt.chara_phy.position());
                        return Ok(ret);
                    }
                }
                InstAiTaskPatrolStep::Idle(duration) => {
                    if loose_le!(duration, 0.0) {
                        continue;
                    }
                    if self.mode == AiTaskPatrolMode::Move {
                        self.switch_mode(AiTaskPatrolMode::MoveStop);
                        if current_act_id != self.inst_move.tmpl_id {
                            ret.next_action = Some(self.inst_move.clone());
                            ret.quick_switch = true;
                        }
                    }
                    else {
                        self.switch_mode(AiTaskPatrolMode::Idle);
                        if current_act_id != self.inst_idle.tmpl_id {
                            ret.next_action = Some(self.inst_idle.clone());
                            ret.quick_switch = true;
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
    fn switch_mode(&mut self, mode: AiTaskPatrolMode) {
        self.mode = mode;
        self.idle_timer = 0.0;
        self.path_idx = 0;
    }

    #[inline]
    fn update_move_path(&mut self, ctxt: &ContextAiTask, src_pos: Vec3A, dst_pos: Vec3A) -> XResult<bool> {
        ctxt.zone.find_path(src_pos, dst_pos, &mut self.move_path)?;
        Ok(self.move_path.len() > 0)
    }

    fn calc_world_move(&mut self, src_pos: Vec3A) -> WorldMoveState {
        let mut waypoint = Vec3A::ZERO;
        while self.path_idx < self.move_path.len() as u32 {
            waypoint = Vec3A::from(self.move_path[self.path_idx as usize]);
            if !Self::is_reached(src_pos, waypoint) {
                break;
            }
            self.path_idx += 1;
        }

        if self.path_idx < self.move_path.len() as u32 {
            let dir = Self::calc_dir_xz(src_pos, waypoint);
            WorldMoveState::new_move(InputMoveSpeed::Normal, dir)
        }
        else {
            WorldMoveState::new_stop()
        }
    }

    #[inline]
    fn is_reached(src_pos: Vec3A, dst_pos: Vec3A) -> bool {
        let dxz = src_pos.xz() - dst_pos.xz();
        let dy = src_pos.y - dst_pos.y;
        dxz.length_squared() < DISTANCE_XZ_THRESHOLD_SQ && dy.abs() < DISTANCE_Y_THRESHOLD
    }

    #[inline]
    fn calc_dir_xz(src_pos: Vec3A, dst_pos: Vec3A) -> Vec2xz {
        (Vec2xz::from_vec3a(dst_pos) - Vec2xz::from_vec3a(src_pos)).normalize()
    }
}
