use critical_point_macros::{csharp_enum, csharp_out};
use glam::{Vec3, Vec3A, Vec3Swizzles};
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::rc::Rc;

use crate::consts::SPF;
use crate::instance::{InstActionAny, InstActionMoveNpc, InstAiTaskGeneral, InstCharacter};
use crate::logic::ai_task::base::{
    AiBrainPurpose, AiBrainThinking, AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny,
    StateAiTaskBase, impl_state_ai_task,
};
use crate::logic::game::ContextUpdate;
use crate::loose_ge;
use crate::utils::{AiTaskType, Castable, NumID, TmplID, XResult, calc_dir_xz, extend, loose_le, xresf};

const THRESHOLD_XZ_RATIO_MOVE: f32 = 0.25;
const THRESHOLD_Y_DISTANCE: f32 = 1.0;

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
pub enum AiTaskGeneralMode {
    Act,
}

#[derive(Debug)]
enum UpdateRes {
    Ret(AiTaskReturn),
    Stop,
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateAiTaskGeneral {
    pub _base: StateAiTaskBase,
    pub mode: AiTaskGeneralMode,
    pub tgt_chara_id: NumID,
    pub action_idx: u32,
}

extend!(StateAiTaskGeneral, StateAiTaskBase);
impl_state_ai_task!(StateAiTaskGeneral, General, "General");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicAiTaskGeneral {
    _base: LogicAiTaskBase,
    inst: Rc<InstAiTaskGeneral>,
    inst_actions: Vec<Rc<dyn InstActionAny>>,

    mode: AiTaskGeneralMode,
    tgt_chara_id: NumID,
    action_idx: u32,
}

extend!(LogicAiTaskGeneral, LogicAiTaskBase);

impl LogicAiTaskGeneral {
    pub fn new(
        ctx: &mut ContextUpdate,
        inst_task: Rc<InstAiTaskGeneral>,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicAiTaskGeneral> {
        let mut inst_actions: Vec<Rc<dyn InstActionAny>> = Vec::with_capacity(inst_task.actions.len());
        for action_id in inst_task.actions.iter() {
            match inst_chara.actions.get(action_id) {
                Some(inst) => inst_actions.push(inst.clone()),
                None => return xresf!(InstNotFound; "id={}", action_id),
            }
        }

        Ok(LogicAiTaskGeneral {
            _base: LogicAiTaskBase::new(ctx.identity.gen_ai_task_id(), inst_task.clone()),
            inst: inst_task,
            inst_actions,

            mode: AiTaskGeneralMode::Act,
            tgt_chara_id: NumID::INVALID,
            action_idx: u32::MAX,
        })
    }
}

unsafe impl LogicAiTaskAny for LogicAiTaskGeneral {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::General
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskGeneral {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            tgt_chara_id: self.tgt_chara_id,
            action_idx: self.action_idx,
        })
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
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;
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
            println!(">>>>>>>>>>>>>>>>>>>>>>>> {} {}", self.current_action, current_action);
            self.stop(ctx, ctxt)?;
            return Ok(AiTaskReturn::default());
        }

        let ret = match self.mode {
            AiTaskGeneralMode::Act => self.update_act(ctx, ctxt)?,
        };

        match ret {
            UpdateRes::Ret(ret) => Ok(ret),
            UpdateRes::Stop => {
                self.stop(ctx, ctxt)?;
                Ok(AiTaskReturn::default())
            }
        }
    }
}

impl LogicAiTaskGeneral {
    #[inline]
    fn init_mode(&mut self, mode: AiTaskGeneralMode) {
        self.mode = mode;
        self.action_idx = u32::MAX;
    }

    fn enter_act(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        debug_assert!(self.inst.actions.len() >= 1);
        debug_assert!(self.inst_actions.len() >= 1);

        self.init_mode(AiTaskGeneralMode::Act);
        self.action_idx = 0;
        self.current_action = self.inst_actions[0].tmpl_id;

        let mut ret = AiTaskReturn::default();
        ret.next_action = Some(self.inst_actions[0].clone());
        ret.ai_purpose = AiBrainPurpose::Attack;
        Ok(ret)
    }

    fn update_act(&mut self, _ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<UpdateRes> {
        debug_assert!((self.action_idx as usize) < self.inst_actions.len());

        let mut ret = AiTaskReturn::default();
        ret.ai_purpose = AiBrainPurpose::Attack;

        let is_inactive = match ctxt.chara_ctrl.current_action() {
            Some(act) => act.is_inactive(),
            None => true,
        };

        // Current action finished.
        if is_inactive {
            if (self.action_idx as usize) + 1 < self.inst_actions.len() {
                self.action_idx += 1;
                self.current_action = self.inst_actions[self.action_idx as usize].tmpl_id;
                ret.next_action = Some(self.inst_actions[self.action_idx as usize].clone());
            }
            else {
                return Ok(UpdateRes::Stop);
            }
        }

        Ok(UpdateRes::Ret(ret))
    }
}
