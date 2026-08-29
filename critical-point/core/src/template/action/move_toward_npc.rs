use crate::template::action::base::TmplAnimation;
use crate::template::action::move_free_npc::TmplActionMoveNpcStop;
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{TmplID, VirtualKey};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveTowardNpc {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character_npcs: Vec<TmplID>,
    pub tags: Vec<String>,
    pub enter_key: VirtualKey,
    pub poise_level: u16,
    pub directions: Vec<TmplActionMoveTowardNpcDir>,
    pub turn_time: f32,
}

impl_tmpl!(TmplActionMoveTowardNpc, ActionMoveTowardNpc, "ActionMoveTowardNpc");

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveTowardNpcDir {
    pub enter_angle: f32,
    pub anim_move: TmplAnimation,
    pub move_speed: f32,
    pub speed_ratio: f32,
    pub anim_start: TmplAnimation,
    pub stops: Vec<TmplActionMoveNpcStop>,
    pub turn_time: f32,
    pub min_distance: f32,
    pub step_length: f32,
}
