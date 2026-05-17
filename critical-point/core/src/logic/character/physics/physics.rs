use critical_point_csgen::CsOut;
use educe::Educe;
use glam::{Quat, Vec3A, Vec3Swizzles};
use glam_ext::{Isometry3A, Vec2xz};
use jolt_physics_rs::{BodyID, Character, CharacterVirtual, JMut, MutableCompoundShape};
use std::cell::Cell;
use std::rc::Rc;

use crate::instance::InstCharacter;
use crate::logic::character::control::LogicCharaControl;
use crate::logic::character::physics::body::CharacterContactListenerImpl;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{NumID, SmallVec, Symbol, XResult, quat_from_dir_xz};

#[repr(C)]
#[derive(
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Value)]
pub struct StateCharaPhysics {
    pub velocity: Vec3A,
    pub position: Vec3A,
    pub direction: Vec2xz,

    #[cs_hide(32, 8)]
    pub body_ids: SmallVec<[BodyID; 4]>,
    #[cs_hide(64, 8)]
    pub box_pairs: SmallVec<[StateCharaHitBoxPair; 4]>,
    #[cs_hide(64, 8)]
    pub group_pairs: SmallVec<[StateCharaHitGroupPair; 3]>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct StateCharaHitBoxPair {
    pub box_index: u16,
    pub dst_chara_id: NumID,
    pub last_hit_time: f32,
    pub hit_times: u16,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct StateCharaHitGroupPair {
    pub group: Symbol,
    pub dst_chara_id: NumID,
    pub hit_times: u16,
}

#[derive(Educe)]
#[educe(Debug)]
pub(crate) struct LogicCharaPhysics {
    pub(super) chara_id: NumID,
    pub(super) inst_chara: Rc<InstCharacter>,
    pub(super) velocity: Vec3A,
    pub(super) position: Vec3A,
    pub(super) direction: Vec2xz,
    pub(super) rotation: Quat,
    pub(super) idle: Cell<bool>,

    #[educe(Debug(ignore))]
    pub(super) character: CharacterHandle,
    pub(super) target_body: BodyID,
    #[educe(Debug(ignore))]
    pub(super) target_shape: JMut<MutableCompoundShape>,
    pub(super) joint_bindings: Vec<JointBinding>,

    pub(super) body_ids: Vec<BodyID>,
    pub(super) box_pairs: Vec<StateCharaHitBoxPair>,
    pub(super) group_pairs: Vec<StateCharaHitGroupPair>,

    pub(super) cache_isometries: Vec<Isometry3A>,
    pub(super) hit_events: Vec<usize>,
    pub(super) be_hit_events: Vec<usize>,
}

pub(super) enum CharacterHandle {
    Npc(JMut<Character>),
    Player(JMut<CharacterVirtual<CharacterContactListenerImpl>>),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct JointBinding {
    pub(super) position: Vec3A,
    pub(super) rotation: Quat,
    pub(super) part: Symbol,
    pub(super) joint: i16,
    pub(super) ratio: f32,
    pub(super) joint2: i16,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CharacterLocation {
    pub position: Vec3A,
    pub rotation: Quat,
    pub velocity: Vec3A,
}

impl LogicCharaPhysics {
    pub(crate) fn new(
        ctx: &mut ContextUpdate,
        chara_id: NumID,
        inst_chara: Rc<InstCharacter>,
        position: Vec3A,
        direction: Vec2xz,
    ) -> XResult<LogicCharaPhysics> {
        let rotation = quat_from_dir_xz(direction);
        let character = LogicCharaPhysics::init_bounding(ctx, &inst_chara, position, rotation)?;
        let (target_body, target_shape, joint_bindings) =
            LogicCharaPhysics::init_bodies(ctx, chara_id, &inst_chara, position, rotation)?;

        let target_bindings_len = joint_bindings.len();
        Ok(LogicCharaPhysics {
            chara_id,
            inst_chara,
            velocity: Vec3A::ZERO,
            position,
            direction,
            rotation,
            idle: Cell::new(true),

            character,
            target_body,
            target_shape,
            joint_bindings,

            body_ids: Vec::with_capacity(32),
            box_pairs: Vec::with_capacity(32),
            group_pairs: Vec::with_capacity(16),

            cache_isometries: Vec::with_capacity(target_bindings_len),
            hit_events: Vec::with_capacity(32),
            be_hit_events: Vec::with_capacity(32),
        })
    }

    pub(crate) fn init(&mut self, ctx: &mut ContextUpdate, chara_ctrl: &LogicCharaControl) -> XResult<()> {
        self.update(ctx, chara_ctrl)
    }

    pub(crate) fn update(&mut self, ctx: &mut ContextUpdate, chara_ctrl: &LogicCharaControl) -> XResult<()> {
        self.cache_isometries.clear();
        self.update_bounding(ctx, chara_ctrl)?;
        self.update_bodies(ctx, chara_ctrl)?;

        if chara_ctrl.animation_changed() {
            self.handle_action_changed(ctx, chara_ctrl)?
        }
        self.update_boxes_and_groups(ctx, chara_ctrl)?;

        self.hit_events.clear();
        self.be_hit_events.clear();
        Ok(())
    }

    pub(crate) fn state(&self) -> StateCharaPhysics {
        StateCharaPhysics {
            velocity: self.velocity,
            position: self.position,
            direction: self.direction,

            body_ids: SmallVec::from_slice(&self.body_ids),
            box_pairs: SmallVec::from_slice(&self.box_pairs),
            group_pairs: SmallVec::from_slice(&self.group_pairs),
        }
    }

    pub(crate) fn restore(&mut self, _ctx: &ContextRestore, state: &StateCharaPhysics) -> XResult<()> {
        self.velocity = state.velocity;
        self.position = state.position;
        self.direction = state.direction;
        self.rotation = quat_from_dir_xz(self.direction);

        self.body_ids.clear();
        self.body_ids.extend_from_slice(&state.body_ids);
        self.box_pairs.clear();
        self.box_pairs.extend_from_slice(&state.box_pairs);
        self.group_pairs.clear();
        self.group_pairs.extend_from_slice(&state.group_pairs);
        Ok(())
    }

    pub(crate) fn clean_up(&mut self) {
        self.hit_events.clear();
        self.be_hit_events.clear();
        self.cache_isometries.clear();
    }

    #[inline]
    pub fn id(&self) -> NumID {
        self.chara_id
    }

    #[inline]
    pub fn position(&self) -> Vec3A {
        self.position
    }

    #[inline]
    pub fn position_xz_3d(&self) -> Vec3A {
        Vec3A::new(self.position.x, 0.0, self.position.z)
    }

    #[inline]
    pub fn position_xz(&self) -> Vec2xz {
        Vec2xz::from_vec2(self.position.xz())
    }

    #[cfg(test)]
    pub fn set_position(&mut self, position: Vec3A) {
        self.position = position;
    }

    #[inline]
    pub fn direction(&self) -> Vec3A {
        self.direction.as_vec3a()
    }

    #[inline]
    pub fn direction_xz(&self) -> Vec2xz {
        self.direction
    }

    #[cfg(test)]
    pub fn set_direction(&mut self, direction: Vec2xz) {
        self.direction = direction;
        self.rotation = quat_from_dir_xz(self.direction);
    }

    #[inline]
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    #[inline]
    pub fn rotation_y(&self) -> Quat {
        self.rotation
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.idle.get()
    }

    #[cfg(test)]
    pub(crate) fn set_idle(&self, idle: bool) {
        self.idle.set(idle);
    }
}
