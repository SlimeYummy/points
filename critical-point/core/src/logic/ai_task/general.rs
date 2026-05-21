use critical_point_csgen::{CsEnum, CsOut};
use glam::{Vec3, Vec3A, Vec3Swizzles};
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use crate::consts::SPF;
use crate::instance::{InstActionAny, InstActionMoveNpc, InstAiTaskGeneral, InstCharacter};
use crate::logic::ai_task::base::{
    AiBrainPurpose, AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny, StateAiTaskBase,
    impl_state_ai_task, AiBrainThinking
};
use crate::logic::game::ContextUpdate;
use crate::loose_ge;
use crate::utils::{
    AiTaskType, Castable, NumID, SmallVec, TmplID, XResult, calc_dir_xz, extend, loose_le, ok_or, xres, xresf,
};

const THRESHOLD_XZ_RATIO_MOVE: f32 = 0.25;
const THRESHOLD_Y_DISTANCE: f32 = 1.0;

#[derive(Debug, Default)]
pub enum AiTaskGeneralMove {
    #[default]
    None,
    Move(Rc<InstActionMoveNpc>),
}

impl AiTaskGeneralMove {
    #[inline]
    fn unwrap_move(&self) -> Rc<InstActionMoveNpc> {
        if let AiTaskGeneralMove::Move(inst) = self {
            inst.clone()
        }
        else {
            panic!("AiTaskGeneralMove::unwrap_move()");
        }
    }
}

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
pub enum AiTaskGeneralMode {
    Move,
    MoveStop,
    Act,
}

#[derive(Debug)]
enum UpdateRes {
    Ret(AiTaskReturn),
    Goto(AiTaskGeneralMode),
    Stop,
}

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateAiTaskGeneral {
    pub _base: StateAiTaskBase,
    pub mode: AiTaskGeneralMode,
    pub tgt_chara_id: NumID,
    pub action_idx: u32,
    pub move_idx: u32,
    pub path_idx: u32,
    pub move_timer: f32,
    #[cs_hide(8, 8)]
    pub move_path: Option<Arc<Vec<Vec3>>>,
}

extend!(StateAiTaskGeneral, StateAiTaskBase);
impl_state_ai_task!(StateAiTaskGeneral, General, "General");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicAiTaskGeneral {
    _base: LogicAiTaskBase,
    inst: Rc<InstAiTaskGeneral>,
    inst_moves: Vec<AiTaskGeneralMove>,
    inst_actions: Vec<Rc<dyn InstActionAny>>,

    mode: AiTaskGeneralMode,
    tgt_chara_id: NumID,
    action_idx: u32,
    move_idx: u32,
    path_idx: u32,
    move_timer: f32,
    move_path: Option<Arc<Vec<Vec3>>>,
}

extend!(LogicAiTaskGeneral, LogicAiTaskBase);

impl LogicAiTaskGeneral {
    pub fn new(
        ctx: &mut ContextUpdate,
        inst_task: Rc<InstAiTaskGeneral>,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicAiTaskGeneral> {
        let mut inst_moves: Vec<AiTaskGeneralMove> = Vec::with_capacity(inst_task.moves.len());
        for mv in inst_task.moves.iter() {
            if mv.action.is_valid() {
                match inst_chara.actions.get(&mv.action) {
                    Some(inst) => inst_moves.push(AiTaskGeneralMove::Move(inst.clone().cast()?)),
                    None => return xres!(InstNotFound, mv.action),
                }
            }
            else {
                inst_moves.push(AiTaskGeneralMove::None);
            }
        }

        let mut inst_actions: Vec<Rc<dyn InstActionAny>> = Vec::with_capacity(inst_task.actions.len());
        for action_id in inst_task.actions.iter() {
            match inst_chara.actions.get(action_id) {
                Some(inst) => inst_actions.push(inst.clone()),
                None => return xres!(InstNotFound, *action_id),
            }
        }

        Ok(LogicAiTaskGeneral {
            _base: LogicAiTaskBase::new(ctx.gene.gen_ai_task_id(), inst_task.clone()),
            inst: inst_task,
            inst_actions,
            inst_moves,

            mode: AiTaskGeneralMode::Act,
            tgt_chara_id: NumID::INVALID,
            action_idx: u32::MAX,
            move_idx: u32::MAX,
            path_idx: 0,
            move_timer: 0.0,
            move_path: None,
        })
    }
}

unsafe impl LogicAiTaskAny for LogicAiTaskGeneral {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::General
    }

    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()> {
        if state.id() != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id(), self._base.id);
        }
        let state = state.cast::<StateAiTaskGeneral>()?;
        self._base.restore(&state._base);
        self.mode = state.mode;
        self.tgt_chara_id = state.tgt_chara_id;
        self.action_idx = state.action_idx;
        self.move_idx = state.action_idx;
        self.path_idx = state.path_idx;
        self.move_timer = state.move_timer;
        self.move_path = state.move_path.clone();
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;

        if let Some(ret) = self.try_enter_move(ctx, ctxt)? {
            return Ok(ret);
        }
        self.enter_act(ctx, ctxt)
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
            AiTaskGeneralMode::Move => self.update_move(ctx, ctxt)?,
            AiTaskGeneralMode::MoveStop => self.update_move_stop(ctx, ctxt)?,
            AiTaskGeneralMode::Act => self.update_act(ctx, ctxt)?,
        };
        match ret {
            Some(ret) => Ok(ret),
            None => self.enter_next(ctx, ctxt),
        }
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskGeneral {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            tgt_chara_id: self.tgt_chara_id,
            action_idx: self.action_idx,
            move_idx: self.move_idx,
            path_idx: self.path_idx,
            move_timer: self.move_timer,
            move_path: self.move_path.clone(),
        })
    }
}

impl LogicAiTaskGeneral {
    #[inline]
    fn init_mode(&mut self, mode: AiTaskGeneralMode) {
        self.mode = mode;
        self.action_idx = u32::MAX;
        self.move_idx = u32::MAX;
        self.path_idx = 0;
        self.move_timer = 0.0;
        if mode != AiTaskGeneralMode::Move {
            self.move_path = None;
        }
    }

    /// Some() => enter move.
    /// None => enter actions directly.
    fn try_enter_move(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let tgt_pos = match ctxt.tgt_chara_phy {
            Some(tgt_chara_phy) => tgt_chara_phy.position(),
            None => return Ok(None), // No target, skip Move mode.
        };

        let distance_xz = Self::distance_xz(ctxt.chara_phy.position(), tgt_pos);
        let distance_y = (ctxt.chara_phy.position().y - tgt_pos.y).abs();
        if distance_y < THRESHOLD_Y_DISTANCE && self.inst.expected_distance.contains(distance_xz) {
            return Ok(None); // Already in range, skip Move mode.
        }

        let Some((idx, _)) = self
            .inst
            .find_move_by_target(distance_xz, ctxt.chara_phy.direction_xz())
        else {
            // In theory, moves should be checked before, so we should not enter this branch.
            // We can only skip Move mode to prevent a crash.
            log::warn!(
                "InstAiTaskGeneral::find_move_by_target(), task_id={}, tmpl_task={}, chara_id={}, tmpl_character={}, distance_xz={}, direction={}",
                self.id,
                self.inst.tmpl_id,
                ctxt.chara_id(),
                ctxt.inst_chara.tmpl_character,
                distance_xz,
                ctxt.chara_phy.direction()
            );
            return Ok(None);
        };

        let mut ret = AiTaskReturn::default();

        match &self.inst_moves[idx] {
            AiTaskGeneralMove::Move(inst) => {
                ret.next_action = Some(inst.clone());

                self.init_mode(AiTaskGeneralMode::Move);
                self.move_idx = idx as u32;

                let src_pos = ctxt.chara_phy.position();
                if !self.update_move_path(ctxt, src_pos, tgt_pos)? {
                    return Ok(None);
                }

                ret.thinking.purpose = AiBrainPurpose::ToCharacter;
                ret.thinking.dst_point = tgt_pos;
                ret.thinking.move_dir = self.calc_move_dir(src_pos);
            }
            AiTaskGeneralMove::None => {
                // Invalid action, skip Move mode directly.
                return Ok(None);
            }
        }
        Ok(Some(ret))
    }

    /// Some() => keep current mode.
    /// None => switch mode.
    fn update_move(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<UpdateRes> {
        // No target or target changed.
        if ctxt.tgt_chara_phy.is_none() || self.tgt_chara_id != ctxt.tgt_chara_id() {
            return Ok(UpdateRes::Goto(AiTaskGeneralMode::MoveStop));
        }

        let tgt_pos = ctxt.tgt_chara_phy.unwrap().position();
        let distance_xz = Self::distance_xz(ctxt.chara_phy.position(), tgt_pos);
        let distance_y = (ctxt.chara_phy.position().y - tgt_pos.y).abs();
        if distance_y < THRESHOLD_Y_DISTANCE && self.inst.expected_distance.contains(distance_xz) {
            // Already in range.
            return Ok(UpdateRes::Goto(AiTaskGeneralMode::MoveStop));
        }

        self.move_timer += ctxt.time_step;
        if loose_ge!(self.move_timer, 5.0 * SPF) {
            let src_pos = ctxt.chara_phy.position();
            if !self.update_move_path(ctxt, src_pos, tgt_pos)? {
                // Path update failed, stop moving.
                return Ok(UpdateRes::Goto(AiTaskGeneralMode::MoveStop));
            }
            self.path_idx = 0;
        }

        let real_dst_pos = match self.move_path.as_ref().and_then(|p| p.last()) {
            Some(pos) => Vec3A::from(*pos),
            None => tgt_pos,
        };

        let mut ret = AiTaskReturn::default();
        ret.thinking.purpose = AiBrainPurpose::ToCharacter;
        ret.thinking.dst_point = real_dst_pos;
        ret.thinking.move_dir = self.calc_move_dir(ctxt.chara_phy.position());
        Ok(UpdateRes::Ret(ret))
    }

    #[inline]
    fn enter_move_stop(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self.mode = AiTaskGeneralMode::MoveStop;
        
        let mut ret = AiTaskReturn::default();
        ret.thinking = Self::make_non_move_thinking(ctxt, AiBrainPurpose::ToCharacter);
        Ok(ret)
    }

    fn update_move_stop(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<UpdateRes> {
        let is_inactive = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.is_inactive(),
            None => true,
        };

        // Move finished.
        if is_inactive {
            if ctxt.tgt_chara_phy.is_some() {
                return Ok(UpdateRes::Goto(AiTaskGeneralMode::Act))
            } else {
                // We do not support enter Act from Move without a target.
                return Ok(UpdateRes::Stop);
            }
        }
        
        let mut ret = AiTaskReturn::default();
        ret.thinking = Self::make_non_move_thinking(ctxt, AiBrainPurpose::ToCharacter);
        Ok(UpdateRes::Ret(ret))
    }

    fn enter_act(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        debug_assert!(self.inst.actions.len() >= 1);
        debug_assert!(self.inst_actions.len() >= 1);

        self.init_mode(AiTaskGeneralMode::Act);
        self.action_idx = 0;

        let mut ret = AiTaskReturn::default();
        ret.next_action = Some(self.inst_actions[0].clone());
        ret.thinking = Self::make_non_move_thinking(ctxt, AiBrainPurpose::Attack);
        Ok(ret)
    }

    fn update_act(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<UpdateRes> {
        debug_assert!((self.action_idx as usize) <= self.inst_actions.len());
        
        let mut ret = AiTaskReturn::default();

        let is_inactive = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.is_inactive(),
            None => true,
        };

        // Current action finished.
        if is_inactive {
            if (self.action_idx as usize) < self.inst_actions.len() {
                self.action_idx += 1;
                ret.next_action = Some(self.inst_actions[self.action_idx as usize].clone());
            } else {
                return Ok(UpdateRes::Stop);
            }
        }

        ret.thinking = Self::make_non_move_thinking(ctxt, AiBrainPurpose::Attack);
        Ok(UpdateRes::Ret(ret))
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
        let inst_move = self.inst_moves[self.move_idx as usize].as_ref().unwrap(); // verified
        let move_path = self.move_path.as_ref().map(|p| p.as_slice()).unwrap_or(&[]);
        let mut waypoint = Vec3A::ZERO;
        while self.path_idx < move_path.len() as u32 {
            waypoint = Vec3A::from(move_path[self.path_idx as usize]);
            if !Self::is_reached(src_pos, waypoint, inst_move.step_length * THRESHOLD_XZ_RATIO_MOVE) {
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
    fn distance_xz(a: Vec3A, b: Vec3A) -> f32 {
        (a.xz() - b.xz()).length()
    }

    #[inline]
    fn calc_dir_xz(src_pos: Vec3A, dst_pos: Vec3A) -> Vec2xz {
        (Vec2xz::from_vec3a(dst_pos) - Vec2xz::from_vec3a(src_pos)).normalize()
    }

    fn make_non_move_thinking(ctxt: &ContextAiTask, purpose: AiBrainPurpose) -> AiBrainThinking {
        let mut thinking = AiBrainThinking::default();
        thinking.purpose = purpose;
        if let Some(tgt_chara_phy) = ctxt.tgt_chara_phy {
            thinking.dst_point = tgt_chara_phy.position();
            thinking.move_dir = calc_dir_xz(
                ctxt.chara_phy.position_xz(),
                tgt_chara_phy.position_xz(),
                ctxt.chara_phy.direction_xz(),
            );
        }
        else {
            thinking.dst_point = ctxt.chara_phy.position();
            thinking.move_dir = ctxt.chara_phy.direction_xz();
        }
        thinking
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
