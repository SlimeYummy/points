use approx::assert_abs_diff_eq;
use critical_point_macros::{csharp_enum, csharp_out};
use glam::{Vec3, Vec3A, Vec3Swizzles};
use glam_ext::Vec2xz;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hint::likely;
use std::rc::Rc;
use std::sync::Arc;

use crate::consts::SPF;
use crate::instance::{InstActionMoveNpc, InstAiTaskMoveToCharacter, InstCharacter};
use crate::logic::ai_task::base::{
    AiBrainPurpose, AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny, StateAiTaskBase,
    impl_state_ai_task,
};
use crate::logic::character::LogicCharaPhysics;
use crate::logic::game::ContextUpdate;
use crate::loose_ge;
use crate::utils::{AiTaskType, Castable, NumID, TmplID, XResult, extend, loose_le, square, xresf};

const THRESHOLD_XZ_RATIO_MOVE: f32 = 0.25;
const THRESHOLD_Y_DISTANCE: f32 = 5.0;

#[csharp_enum]
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
)]
#[rkyv(derive(Debug))]
pub enum AiTaskMoveToCharacterMode {
    Move,
    Stop,
    Turn,
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateAiTaskMoveToCharacter {
    pub _base: StateAiTaskBase,
    pub mode: AiTaskMoveToCharacterMode,
    pub target_chara: NumID,
    pub path_idx: u32,
    pub path_refresh_timer: f32,
    #[csharp_hide(8, 8)]
    pub move_path: Option<Arc<Vec<Vec3>>>,
}

extend!(StateAiTaskMoveToCharacter, StateAiTaskBase);
impl_state_ai_task!(StateAiTaskMoveToCharacter, MoveToCharacter, "MoveToCharacter");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicAiTaskMoveToCharacter {
    _base: LogicAiTaskBase,
    inst: Rc<InstAiTaskMoveToCharacter>,
    inst_move: Rc<InstActionMoveNpc>,

    mode: AiTaskMoveToCharacterMode,
    target_chara: NumID,
    path_idx: u32,
    path_refresh_timer: f32,
    move_path: Option<Arc<Vec<Vec3>>>,
}

extend!(LogicAiTaskMoveToCharacter, LogicAiTaskBase);

impl LogicAiTaskMoveToCharacter {
    pub fn new(
        ctx: &mut ContextUpdate,
        inst_task: Rc<InstAiTaskMoveToCharacter>,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicAiTaskMoveToCharacter> {
        let inst_move = match inst_chara.actions.get(&inst_task.move_action) {
            Some(inst) => inst.clone().cast()?,
            None => return xresf!(InstNotFound; "id={}", inst_task.move_action),
        };

        Ok(LogicAiTaskMoveToCharacter {
            _base: LogicAiTaskBase::new(ctx.identity.gen_ai_task_id(), inst_task.clone()),
            inst: inst_task,
            inst_move,

            mode: AiTaskMoveToCharacterMode::Move,
            target_chara: NumID::INVALID,
            path_idx: 0,
            path_refresh_timer: 0.0,
            move_path: None,
        })
    }
}

unsafe impl LogicAiTaskAny for LogicAiTaskMoveToCharacter {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::MoveToCharacter
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskMoveToCharacter {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            target_chara: self.target_chara,
            path_idx: self.path_idx,
            path_refresh_timer: self.path_refresh_timer,
            move_path: self.move_path.clone(),
        })
    }

    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()> {
        if state.id() != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id(), self._base.id);
        }
        let state = state.cast::<StateAiTaskMoveToCharacter>()?;
        self._base.restore(&state._base);
        self.mode = state.mode;
        self.target_chara = state.target_chara;
        self.path_idx = state.path_idx;
        self.path_refresh_timer = state.path_refresh_timer;
        self.move_path = state.move_path.clone();
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;
        self.try_enter_move(ctx, ctxt)
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
            AiTaskMoveToCharacterMode::Move => self.update_move(ctx, ctxt)?,
            AiTaskMoveToCharacterMode::Stop => self.update_stop(ctx, ctxt)?,
            AiTaskMoveToCharacterMode::Turn => self.update_turn(ctx, ctxt)?,
        };

        match ret {
            Some(ret) => Ok(ret),
            None => match self.mode {
                AiTaskMoveToCharacterMode::Move => self.enter_stop(ctx, ctxt),
                AiTaskMoveToCharacterMode::Stop => self.enter_turn(ctx, ctxt),
                AiTaskMoveToCharacterMode::Turn => unreachable!(),
            },
        }
    }
}

impl LogicAiTaskMoveToCharacter {
    #[inline]
    fn init_mode(&mut self, mode: AiTaskMoveToCharacterMode) {
        self.mode = mode;
        self.path_idx = 0;
        self.path_refresh_timer = 0.0;
        if mode != AiTaskMoveToCharacterMode::Move {
            self.move_path = None;
        }
    }

    fn try_enter_move(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self.target_chara = ctxt.chara_ctrl.ai_thinking().target_chara;
        self.current_action = self.inst_move.tmpl_id;

        let mut ret = AiTaskReturn::default();
        ret.next_action = Some(self.inst_move.clone());
        ret.ai_purpose = AiBrainPurpose::ToCharacter;

        let Some(tgt_chara_pos) = ctxt.chara_ctrl.ai_thinking().target_chara_pos()
        else {
            // No target.
            self.init_mode(AiTaskMoveToCharacterMode::Stop);
            ret.ai_move_dst_pos = ctxt.chara_phy.position();
            ret.ai_move_dir = Vec2xz::ZERO;
            return Ok(ret);
        };

        let rel = self.calc_relative(&ctxt.chara_phy, tgt_chara_pos);
        if rel == Ordering::Equal {
            // Already in range, enter stop mode directly.
            self.init_mode(AiTaskMoveToCharacterMode::Stop);
            ret.ai_move_dst_pos = ctxt.chara_phy.position();
            ret.ai_move_dir = Vec2xz::ZERO;
            return Ok(ret);
        }

        let dst_pos = if rel == Ordering::Greater {
            tgt_chara_pos
        }
        else {
            self.calc_target_pos_move_away(&ctxt.chara_phy, tgt_chara_pos)
        };

        let src_pos = ctxt.chara_phy.position();
        if !self.update_move_path(ctxt, src_pos, dst_pos)? {
            // Path update failed, stop moving.
            self.init_mode(AiTaskMoveToCharacterMode::Stop);
            return Ok(ret);
        }

        self.init_mode(AiTaskMoveToCharacterMode::Move);
        ret.ai_move_dst_pos = dst_pos;
        ret.ai_move_dir = self.calc_move_dir(src_pos, ctxt.chara_phy.direction_xz());
        Ok(ret)
    }

    /// Some() => keep current mode.
    /// None => switch mode.
    fn update_move(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let ai_thinking = ctxt.chara_ctrl.ai_thinking();

        // No target or target changed.
        if ai_thinking.target_chara.is_invalid() || self.target_chara != ai_thinking.target_chara {
            return Ok(None);
        }

        let Some(tgt_chara_pos) = ctxt.chara_ctrl.ai_thinking().target_chara_pos()
        else {
            // No target.
            return Ok(None);
        };

        let rel = self.calc_relative(&ctxt.chara_phy, tgt_chara_pos);
        if rel == Ordering::Equal {
            // Already in range, enter stop mode directly.
            return Ok(None);
        }

        self.path_refresh_timer += ctxt.time_step;
        if loose_ge!(self.path_refresh_timer, 5.0 * SPF) {
            self.path_refresh_timer = 0.0;

            let dst_pos = if rel == Ordering::Greater {
                tgt_chara_pos
            }
            else {
                self.calc_target_pos_move_away(&ctxt.chara_phy, tgt_chara_pos)
            };

            let src_pos = ctxt.chara_phy.position();
            if !self.update_move_path(ctxt, src_pos, dst_pos)? {
                // Path update failed, stop moving.
                return Ok(None);
            }
            self.path_idx = 0;
        }

        let real_dst_pos = match self.move_path.as_ref().and_then(|p| p.last()) {
            Some(pos) => Vec3A::from(*pos),
            None => ctxt.chara_phy.position(), // Should not happen, move_path should be filled here.
        };

        let mut ret = AiTaskReturn::default();
        ret.ai_purpose = AiBrainPurpose::ToCharacter;
        ret.ai_move_dst_pos = real_dst_pos;
        ret.ai_move_dir = self.calc_move_dir(ctxt.chara_phy.position(), ctxt.chara_phy.direction_xz());
        Ok(Some(ret))
    }

    #[inline]
    fn enter_stop(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self.mode = AiTaskMoveToCharacterMode::Stop;

        let mut ret = AiTaskReturn::default();
        ret.ai_purpose = AiBrainPurpose::ToCharacter;

        // During stop stage, we want character to face target character.
        Self::fill_face_character(&mut ret, ctxt);
        Ok(ret)
    }

    /// Some() => keep current mode.
    /// None => switch mode.
    fn update_stop(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        let is_inactive = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.is_inactive(),
            None => true,
        };

        // Move finished.
        if is_inactive {
            return Ok(None);
        }

        let mut ret = AiTaskReturn::default();
        ret.ai_purpose = AiBrainPurpose::ToCharacter;

        // During stop stage, we want character to face target character.
        Self::fill_face_character(&mut ret, ctxt);
        Ok(Some(ret))
    }

    fn enter_turn(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self.mode = AiTaskMoveToCharacterMode::Turn;

        let mut ret = AiTaskReturn::default();
        ret.ai_purpose = AiBrainPurpose::ToCharacter;

        // During turn stage, we want character to face target character.
        Self::fill_face_character(&mut ret, ctxt);
        Ok(ret)
    }

    fn update_turn(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<Option<AiTaskReturn>> {
        Ok(None)
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

    #[inline]
    fn calc_relative(&self, chara_phy: &LogicCharaPhysics, tgt_chara_pos: Vec3A) -> Ordering {
        let dir = tgt_chara_pos - chara_phy.position();
        let dist_sq = dir.length_squared();

        if dist_sq > square(self.inst.expected_distance.max) {
            Ordering::Greater // Move closer to target.
        }
        else if dist_sq < square(self.inst.expected_distance.min) {
            Ordering::Less // Move away from target.
        }
        else {
            Ordering::Equal // Already in distance.
        }
    }

    fn calc_target_pos_move_away(&self, chara_phy: &LogicCharaPhysics, tgt_chara_pos: Vec3A) -> Vec3A {
        // TODO: consider y-axis?
        let dir_xz = Vec2xz::from_vec3a(tgt_chara_pos - chara_phy.position());
        let dist_xz = dir_xz.length();

        let norm_dir_xz = if likely(dist_xz > 1e-3) {
            dir_xz.normalize()
        }
        else {
            assert_abs_diff_eq!(chara_phy.direction_xz().length_squared(), 1.0, epsilon = 1e-3);
            chara_phy.direction_xz()
        };

        let expected_dist = self.inst.expected_distance.middle();
        let dist_diff = dist_xz - expected_dist;
        let dst_pos_xz = chara_phy.position_xz() + norm_dir_xz * dist_diff;

        Vec3A::new(dst_pos_xz.x, chara_phy.position().y, dst_pos_xz.z)
    }

    fn calc_move_dir(&mut self, src_pos: Vec3A, def_val: Vec2xz) -> Vec2xz {
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
            let dir = Vec2xz::from_vec3a(waypoint) - Vec2xz::from_vec3a(src_pos);
            if likely(dir != Vec2xz::ZERO) {
                return dir.normalize();
            }
        }
        def_val
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
    fn fill_face_character(ret: &mut AiTaskReturn, ctxt: &ContextAiTask) {
        match ctxt.chara_ctrl.ai_thinking().target_chara_pos() {
            Some(tgt_chara_pos) => {
                ret.ai_move_dst_pos = tgt_chara_pos;
                // ret.ai_move_dir = Vec2xz::from_vec3a(tgt_chara_pos - ctxt.chara_phy.position()).normalize();
            }
            None => {
                ret.ai_move_dst_pos = ctxt.chara_phy.position();
                // ret.ai_move_dir = Vec2xz::ZERO;
            }
        };
    }
}
