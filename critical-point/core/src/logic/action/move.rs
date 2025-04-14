use cirtical_point_csgen::{CsEnum, CsOut};
use glam::Vec2;
use std::f32::consts::{FRAC_PI_2, PI};
use std::fmt::Debug;
use std::rc::Rc;

use crate::consts::WEIGHT_THRESHOLD;
use crate::instance::{InstAction, InstActionMove};
use crate::logic::action::base::{
    continue_to, ArchivedStateAction, ContextAction, LogicAction, LogicActionBase, StateAction, StateActionAnimation,
    StateActionBase, StateActionType,
};
use crate::logic::game::ContextUpdate;
use crate::template::{TmplActionMove, TmplType};
use crate::utils::{calc_ratio, extend, xresf, ASymbol, CastRef, XResult};

const ANIME_MOVE_ID: u32 = 1;
const ANIME_TURN_LEFT_ID: u32 = 1;
const ANIME_TURN_RIGHT_ID: u32 = 1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsEnum)]
#[archive_attr(derive(Debug))]
pub enum ActionMoveMode {
    None,
    Start,
    Move,
    TurnLeft,
    TurnRight,
    Stop,
}

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionMove {
    pub _base: StateActionBase,
    pub mode: ActionMoveMode,
    pub switch_progress: u32,
    pub previous_progress: u32,
    pub current_progress: u32,
}

extend!(StateActionMove, StateActionBase);

unsafe impl StateAction for StateActionMove {
    #[inline]
    fn typ(&self) -> StateActionType {
        assert!(self.typ == StateActionType::Move);
        StateActionType::Move
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        assert!(self.tmpl_typ == TmplType::ActionMove);
        TmplType::ActionMove
    }
}

impl ArchivedStateAction for rkyv::Archived<StateActionMove> {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::Move
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionMove
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMove {
    _base: LogicActionBase,
    tmpl: Rc<TmplActionMove>,
    inst: Rc<InstActionMove>,

    yam_step_cos: f32,
    yam_step_vec: Vec2,
    turn_step_vec: Vec2,
    min_turn_cos: f32,

    mode: ActionMoveMode,
    switch_progress: u32,
    previous_progress: u32,
    current_progress: u32,
}

extend!(LogicActionMove, LogicActionBase);

unsafe impl LogicAction for LogicActionMove {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::Move
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionMove
    }

    #[inline]
    fn restore(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()> {
        self.restore_impl(state)
    }

    #[inline]
    fn update(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        ctxa: &mut ContextAction<'_>,
    ) -> XResult<Option<Box<dyn StateAction>>> {
        self.update_impl(ctx, ctxa)
    }
}

impl LogicActionMove {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst_act: Rc<InstActionMove>) -> XResult<LogicActionMove> {
        let yam_step_cos = libm::cosf(FRAC_PI_2 / (inst_act.tmpl.yam_time as f32));
        let yam_step_vec = Vec2::from_angle(FRAC_PI_2 / (inst_act.tmpl.yam_time as f32));
        let turn_step_vec = Vec2::from_angle(PI / (inst_act.tmpl.turn_time as f32));
        let min_turn_cos = match (&inst_act.tmpl.anime_turn_left, &inst_act.tmpl.anime_turn_right) {
            (Some(_), Some(_)) => 0.0,
            _ => -1.0,
        };

        Ok(LogicActionMove {
            _base: LogicActionBase {
                derive_level: inst_act.derive_level,
                antibreak_level: inst_act.antibreak_level,
                ..LogicActionBase::new(ctx.gene.gen_id(), inst_act.id.clone(), ctx.frame)
            },
            tmpl: inst_act.tmpl.clone(),
            inst: inst_act,

            yam_step_cos,
            yam_step_vec,
            turn_step_vec,
            min_turn_cos,

            mode: ActionMoveMode::None,
            switch_progress: 0,
            previous_progress: 0,
            current_progress: 0,
        })
    }

    fn restore_impl(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={} self.id={}", state.id, self._base.id);
        }
        let state = state.cast_ref::<StateActionMove>()?;

        self._base.restore(&state._base);
        self.mode = state.mode;
        self.switch_progress = state.switch_progress;
        self.previous_progress = state.previous_progress;
        self.current_progress = state.current_progress;
        Ok(())
    }

    fn save(&self) -> Box<StateActionMove> {
        Box::new(StateActionMove {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            mode: self.mode,
            switch_progress: self.switch_progress,
            previous_progress: self.previous_progress,
            current_progress: self.current_progress,
        })
    }

    fn update_impl(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        ctxa: &mut ContextAction<'_>,
    ) -> XResult<Option<Box<dyn StateAction>>> {
        println!("---------------------------------------");

        let anim_weight = match self._base.handle_enter_leave(ctx, ctxa, self.tmpl.enter_time) {
            Some(ratio) => ratio,
            None => return Ok(None),
        };

        if !self.is_leaving {
            let chara_dir = ctxa.chara_physics.direction_2d();
            let move_dir = match ctxa.input_vars.optimized_world_move().move_dir() {
                Some(dir) => dir,
                None => {
                    if self.tmpl.anime_stop.is_none() {
                        self.is_leaving = true;
                        Vec2::ZERO
                    } else {
                        self.mode = ActionMoveMode::Stop;
                        chara_dir
                    }
                }
            };
            let diff_cos = chara_dir.dot(move_dir);
            println!(
                "diff_cos {} chara_dir {} move_dir {} self.yam_step_vec {}",
                diff_cos, chara_dir, move_dir, self.yam_step_vec
            );

            let mut new_chara_dir = chara_dir;
            let mut new_move_dir = Vec2::ZERO;
            loop {
                match self.mode {
                    ActionMoveMode::None => {
                        match diff_cos > self.yam_step_cos {
                            true => continue_to!(self.mode, ActionMoveMode::Move),
                            false => continue_to!(self.mode, ActionMoveMode::Start),
                        };
                    }
                    ActionMoveMode::Start => {
                        if diff_cos > self.yam_step_cos {
                            continue_to!(self.mode, ActionMoveMode::Move);
                        } else {
                            let mut rot = self.yam_step_vec;
                            rot.y *= chara_dir.perp_dot(move_dir).signum(); // sign
                            new_chara_dir = rot.rotate(chara_dir);
                            println!("chara_dir {} new_chara_dir {} rot {}", chara_dir, new_chara_dir, rot);
                        }
                    }
                    ActionMoveMode::Move => {
                        if diff_cos > self.yam_step_cos {
                            new_chara_dir = move_dir;
                        } else if diff_cos >= self.min_turn_cos {
                            let mut rot = self.yam_step_vec;
                            rot.y *= chara_dir.perp_dot(move_dir).signum(); // sign
                            new_chara_dir = rot.rotate(chara_dir);
                        } else {
                            let sign = chara_dir.perp_dot(move_dir).signum();
                            if sign < 0.0 {
                                continue_to!(self.mode, ActionMoveMode::TurnLeft);
                            } else if sign > 0.0 {
                                continue_to!(self.mode, ActionMoveMode::TurnRight);
                            }
                        }
                        new_move_dir = new_chara_dir * 1.0;
                    }
                    ActionMoveMode::TurnLeft => {}
                    ActionMoveMode::TurnRight => {}
                    ActionMoveMode::Stop => {}
                }
                break;
            }

            ctxa.set_new_velocity(new_move_dir * 5.0);
            ctxa.set_new_rotation(new_chara_dir);
        }

        let state = match self.mode {
            ActionMoveMode::Start => self.play_move(anim_weight),
            ActionMoveMode::Move => self.play_move(anim_weight),
            // ActionMoveMode::TurnLeft => {}
            // ActionMoveMode::TurnRight => {}
            ActionMoveMode::Stop => self.play_move(anim_weight),
            _ => unreachable!(),
        };
        Ok(Some(state))
    }

    fn play_move(&mut self, weight: f32) -> Box<StateActionMove> {
        let anime_move: &crate::template::TmplAnimation = &self.tmpl.anime_move;
        self.current_progress = (self.current_progress + 1) % anime_move.duration;
        let state_idle = StateActionAnimation {
            animation_id: ANIME_MOVE_ID,
            file: ASymbol::from(&anime_move.file),
            ratio: calc_ratio(self.current_progress, anime_move.duration),
            weight,
        };

        let mut state = self.save();
        state.animations[0] = state_idle;
        state
    }
}
