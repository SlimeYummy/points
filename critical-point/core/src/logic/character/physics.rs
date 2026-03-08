use approx::{abs_diff_eq, abs_diff_ne, assert_abs_diff_eq};
use core::f32;
use critical_point_csgen::CsOut;
use educe::Educe;
use glam::{Quat, Vec3A, Vec3Swizzles};
use glam_ext::{Isometry3A, Transform3A, Vec2xz};
use jolt_physics_rs::{
    self as jolt, vdata, Body, BodyCreationSettings, BodyID, BodyInterface, CharacterContactListener,
    CharacterContactListenerVTable, CharacterContactSettings, CharacterVirtual, CharacterVirtualSettings,
    ExtendedUpdateSettings, GroundState, JMut, JRef, JVec3, MotionType, MutableCompoundShape,
    MutableCompoundShapeSettings, PhysicsMaterial, SubShapeID, SubShapeSettings,
};
use ozz_animation_rs::SKELETON_NO_PARENT;
use std::cell::Cell;
use std::rc::Rc;

use crate::animation::{rest_poses_to_model_transforms, HitSampler, HitTrackBase};
use crate::consts::SPF;
use crate::instance::InstCharacter;
use crate::logic::character::LogicCharaAction;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::physics::phy_layer;
use crate::logic::{LogicActionAnimationID, PhyBodyUserData};
use crate::utils::{quat_from_dir_xz, xerrf, xfrom, NumID, Symbol, XResult};

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
pub struct StateCharaPhysics {
    pub velocity: Vec3A,
    pub position: Vec3A,
    pub direction: Vec2xz,
}

#[derive(Educe)]
#[educe(Debug)]
pub(crate) struct LogicCharaPhysics {
    chara_id: NumID,
    inst_chara: Rc<InstCharacter>,
    velocity: Vec3A,
    position: Vec3A,
    direction: Vec2xz,
    rotation: Quat,
    idle: Cell<bool>,

    #[educe(Debug(ignore))]
    character: JMut<CharacterVirtual<CharacterContactListenerImpl>>,
    target_body: BodyID,
    #[educe(Debug(ignore))]
    target_shape: JMut<MutableCompoundShape>,
    joint_bindings: Vec<JointBinding>,

    hit_current_id: LogicActionAnimationID,
    hit_bodies: Vec<BodyID>,

    cache_isometries: Vec<Isometry3A>,
}

#[derive(Debug, Clone, Copy)]
struct JointBinding {
    position: Vec3A,
    rotation: Quat,
    part: Symbol,
    joint: i16,
    ratio: f32,
    joint2: i16,
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

            hit_current_id: LogicActionAnimationID::INVALID,
            hit_bodies: Vec::with_capacity(16),

            cache_isometries: Vec::with_capacity(target_bindings_len),
        })
    }

    pub(crate) fn state(&self) -> StateCharaPhysics {
        StateCharaPhysics {
            velocity: self.velocity,
            position: self.position,
            direction: self.direction,
        }
    }

    pub(crate) fn restore(&mut self, _ctx: &ContextRestore, state: &StateCharaPhysics) -> XResult<()> {
        self.velocity = state.velocity;
        self.position = state.position;
        self.direction = state.direction;
        self.rotation = quat_from_dir_xz(self.direction);
        Ok(())
    }

    fn init_bounding(
        ctx: &mut ContextUpdate,
        inst_chara: &InstCharacter,
        position: Vec3A,
        rotation: Quat,
    ) -> XResult<JMut<CharacterVirtual<CharacterContactListenerImpl>>> {
        let charc_phy = ctx.asset.load_character_physics(inst_chara.skeleton_files)?;

        let mut character = CharacterVirtual::new(
            &mut ctx.physics,
            &CharacterVirtualSettings::new(charc_phy.bounding),
            position.into(),
            rotation,
        );
        character.set_listener(Some(CharacterContactListenerImpl::new_vbox(
            CharacterContactListenerImpl {
                allow_sliding: false,
                body_itf: unsafe { ctx.physics.steal_body_itf() },
            },
        )));
        Ok(character)
    }

    fn init_bodies(
        ctx: &mut ContextUpdate,
        chara_id: NumID,
        inst_chara: &InstCharacter,
        position: Vec3A,
        rotation: Quat,
    ) -> XResult<(BodyID, JMut<MutableCompoundShape>, Vec<JointBinding>)> {
        let skeleton = ctx.asset.load_skeleton(inst_chara.skeleton_files)?;
        let mut charc_phy = ctx.asset.load_character_physics(inst_chara.skeleton_files)?;

        let mut model_transforms = vec![Transform3A::ZERO; skeleton.num_joints()];
        rest_poses_to_model_transforms(&skeleton, &mut model_transforms)?;
        let mut joint_bindings = Vec::with_capacity(charc_phy.bodies.len());
        let mut sub_shape_settings = Vec::with_capacity(charc_phy.bodies.len());

        for body in charc_phy.bodies.drain(..) {
            let joint = skeleton
                .joint_by_name(body.joint.as_str())
                .ok_or_else(|| xerrf!(BadAsset; "target_box={}, joint={}", &inst_chara.skeleton_files, &body.joint))?;

            let mut joint2 = SKELETON_NO_PARENT as i16;
            if !body.joint2.is_empty() {
                joint2 = skeleton.joint_by_name(body.joint2.as_str()).ok_or_else(
                    || xerrf!(BadAsset; "target_box={}, joint2={}", &inst_chara.skeleton_files, &body.joint2),
                )?
            };

            joint_bindings.push(JointBinding {
                position: body.position,
                rotation: body.rotation,
                part: body.part,
                joint,
                ratio: body.ratio,
                joint2,
            });

            let transform = model_transforms[joint as usize];
            let rotation = transform.rotation * body.rotation;
            let position = if joint2 < 0 {
                transform.translation + transform.rotation * body.position
            }
            else {
                Vec3A::lerp(
                    transform.translation,
                    model_transforms[joint2 as usize].translation,
                    body.ratio,
                ) + transform.rotation * body.position
            };

            sub_shape_settings.push(SubShapeSettings::new(body.shape, position, rotation));
        }

        let target_box_settings = MutableCompoundShapeSettings::new(&sub_shape_settings);
        let mut target_shape = jolt::create_mutable_compound_shape_mut(&target_box_settings).map_err(xfrom!())?;

        let mut settings = BodyCreationSettings::new(
            unsafe { target_shape.steal_ref().into() },
            phy_layer!(Target, inst_chara.is_player => Player | Enemy),
            MotionType::Kinematic,
            position.into(),
            rotation * inst_chara.skeleton_rotation,
        );
        settings.user_data = PhyBodyUserData::new_character(chara_id).into();

        let target_body = ctx
            .physics
            .body_itf()
            .create_add_body(&settings, true)
            .map_err(xfrom!())?;

        Ok((target_body, target_shape, joint_bindings))
    }

    pub(crate) fn update(&mut self, ctx: &mut ContextUpdate, action: &LogicCharaAction) -> XResult<()> {
        self.cache_isometries.clear();
        self.update_bounding(ctx, action)?;
        self.update_bodies(ctx, action)?;
        self.update_hits(ctx, action)?;
        Ok(())
    }

    fn update_bounding(&mut self, ctx: &mut ContextUpdate, action: &LogicCharaAction) -> XResult<()> {
        let new_rotation = quat_from_dir_xz(action.new_direction());
        if new_rotation != self.character.get_rotation() {
            self.character.set_rotation(new_rotation);
        }
        self.character.get_listener_mut().unwrap().allow_sliding = abs_diff_ne!(action.new_velocity(), Vec3A::ZERO);

        self.character.update_ground_velocity();

        let gravity: Vec3A = ctx.physics.get_gravity();
        let linear_velocity: Vec3A = self.character.get_linear_velocity();
        let ground_velocity: Vec3A = self.character.get_ground_velocity();
        let moving_towards_ground = (linear_velocity.y - ground_velocity.y) < 0.1;

        if abs_diff_eq!(action.new_velocity().y, 0.0) {
            let mut new_velocity;
            if self.character.get_ground_state() == GroundState::OnGround && moving_towards_ground {
                new_velocity = ground_velocity;
            }
            else {
                new_velocity = Vec3A::new(0.0, linear_velocity.y, 0.0);
            }

            new_velocity += gravity * SPF; // Gravity

            if self.character.is_supported() {
                new_velocity += action.new_velocity();
            }
            else {
                let horizontal_velocity = linear_velocity - Vec3A::new(0.0, linear_velocity.y, 0.0);
                new_velocity += horizontal_velocity;
            }
            self.character.set_linear_velocity(new_velocity);
        }
        else {
            self.character.set_linear_velocity(action.new_velocity());
        }

        self.character.extended_update(
            phy_layer!(Bounding, self.inst_chara.is_player => Player | Enemy),
            SPF,
            gravity.into(),
            &ExtendedUpdateSettings::default(),
        );

        ctx.physics
            .body_itf()
            .set_position(self.target_body, self.character.get_position(), true);

        self.velocity = self.character.get_linear_velocity();
        self.position = self.character.get_position();

        assert_abs_diff_eq!(self.character.get_rotation().x, 0.0, epsilon = 0.01);
        assert_abs_diff_eq!(self.character.get_rotation().z, 0.0, epsilon = 0.01);
        self.direction = action.new_direction();
        self.rotation = quat_from_dir_xz(self.direction);
        self.idle.set(true);
        Ok(())
    }

    fn update_bodies(&mut self, ctx: &mut ContextUpdate, action: &LogicCharaAction) -> XResult<()> {
        let model_transforms = action.model_transforms();
        for binding in &self.joint_bindings {
            let transform = model_transforms[binding.joint as usize];
            let rotation = transform.rotation * binding.rotation;
            let position = if binding.joint2 < 0 {
                transform.translation + transform.rotation * binding.position
            }
            else {
                Vec3A::lerp(
                    transform.translation,
                    model_transforms[binding.joint2 as usize].translation,
                    binding.ratio,
                ) + transform.rotation * binding.position
            };
            self.cache_isometries.push(Isometry3A::new(position.into(), rotation));
        }

        let previous_center_of_mass = self.target_shape.get_center_of_mass();
        self.target_shape.modify_shapes_by_isometry(0, &self.cache_isometries);
        ctx.physics
            .body_itf()
            .notify_shape_changed(self.target_body, previous_center_of_mass, false, true);

        ctx.physics.body_itf().set_position_rotation(
            self.target_body,
            self.position,
            self.rotation * self.inst_chara.skeleton_rotation,
            true,
        );
        Ok(())
    }

    fn update_hits(&mut self, ctx: &mut ContextUpdate, action: &LogicCharaAction) -> XResult<()> {
        let (current_id, sampler) = match action.hit_motion_info() {
            Some(info) => (info.0, Some(info.1)),
            None => (LogicActionAnimationID::INVALID, None),
        };

        let body_itf = ctx.physics.body_itf();
        if current_id != self.hit_current_id {
            for body_id in self.hit_bodies.drain(..) {
                if body_id.is_valid() {
                    body_itf.remove_body(body_id);
                    body_itf.destroy_body(body_id);
                }
            }
            self.hit_current_id = current_id;

            if let Some(sampler) = sampler {
                self.hit_bodies.resize(sampler.tracks_count(), BodyID::INVALID);
            }
        }

        if let Some(sampler) = sampler {
            let mut body_idx = 0;
            for (track_idx, joint) in sampler.joints().iter().enumerate() {
                let track = &sampler.hit_motion.joint_tracks[track_idx];
                debug_assert_eq!(track.hit_id, joint.hit_id);
                self.update_hit_body(ctx, body_idx, track, joint)?;
                body_idx += 1;
            }

            for (track_idx, weapon) in sampler.weapons().iter().enumerate() {
                let track = &sampler.hit_motion.weapon_tracks[track_idx];
                debug_assert_eq!(track.hit_id, weapon.hit_id);
                self.update_hit_body(ctx, body_idx, track, weapon)?;
                body_idx += 1;
            }
        }

        Ok(())
    }

    fn update_hit_body<S>(
        &mut self,
        ctx: &mut ContextUpdate,
        body_idx: usize,
        track: &HitTrackBase,
        sampler: &HitSampler<S>,
    ) -> XResult<()> {
        let body_itf = ctx.physics.body_itf();

        if let Some(isometry) = sampler.isometry() {
            let hit_isometry =
                Isometry3A::new_3a(self.position, self.rotation * self.inst_chara.skeleton_rotation) * *isometry;

            if self.hit_bodies[body_idx] == BodyID::INVALID {
                let mut settings = BodyCreationSettings::new_sensor(
                    track.shape.clone(),
                    phy_layer!(Hit, self.inst_chara.is_player => Enemy | Player),
                    MotionType::Static,
                    hit_isometry.translation,
                    hit_isometry.rotation,
                );
                settings.user_data = PhyBodyUserData::new_hit(self.chara_id, track.hit_id).into();
                self.hit_bodies[body_idx] = body_itf.create_add_body(&settings, true).map_err(xfrom!())?;
            }
            else {
                body_itf.set_position_rotation(
                    self.hit_bodies[body_idx],
                    hit_isometry.translation,
                    hit_isometry.rotation,
                    true,
                );
            }
        }
        else {
            if self.hit_bodies[body_idx] != BodyID::INVALID {
                body_itf.remove_body(self.hit_bodies[body_idx]);
                body_itf.destroy_body(self.hit_bodies[body_idx]);
                self.hit_bodies[body_idx] = BodyID::INVALID;
            }
        }

        Ok(())
    }
}

impl LogicCharaPhysics {
    #[inline]
    pub fn position(&self) -> Vec3A {
        self.position
    }

    #[inline]
    pub fn position_xz(&self) -> Vec3A {
        Vec3A::new(self.position.x, 0.0, self.position.z)
    }

    #[inline]
    pub fn position_2d(&self) -> Vec2xz {
        Vec2xz::from_vec2(self.position.xz())
    }

    #[cfg(test)]
    pub fn set_position(&mut self, position: Vec3A) {
        self.position = position;
    }

    #[inline]
    pub fn direction(&self) -> Vec2xz {
        self.direction
    }

    #[inline]
    pub fn direction_3d(&self) -> Vec3A {
        self.direction.as_vec3a()
    }

    #[inline]
    pub fn direction_xz(&self) -> Vec3A {
        self.direction_3d()
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

#[vdata(CharacterContactListenerVTable)]
struct CharacterContactListenerImpl {
    allow_sliding: bool,
    body_itf: JRef<BodyInterface>,
}

impl CharacterContactListener for CharacterContactListenerImpl {
    fn on_adjust_body_velocity(
        &mut self,
        _character: &CharacterVirtual,
        _body2: &Body,
        _linear_velocity: &mut Vec3A,
        _angular_velocity: &mut Vec3A,
    ) {
    }

    fn on_contact_validate(&mut self, _character: &CharacterVirtual, _body2: &BodyID, _subshape2: &SubShapeID) -> bool {
        true
    }

    fn on_character_contact_validate(
        &mut self,
        _character: &CharacterVirtual,
        _other_character: &CharacterVirtual,
        _subshape2: &SubShapeID,
    ) -> bool {
        true
    }

    fn on_contact_added(
        &mut self,
        _character: &CharacterVirtual,
        body2: &BodyID,
        _subshape2: &SubShapeID,
        _contact_position: JVec3,
        _contact_normal: JVec3,
        settings: &mut CharacterContactSettings,
    ) {
        if settings.can_push_character && self.body_itf.get_motion_type(*body2) != MotionType::Static {
            self.allow_sliding = true;
        }
    }

    fn on_contact_persisted(
        &mut self,
        _character: &CharacterVirtual,
        body2: &BodyID,
        _subshape2: &SubShapeID,
        _contact_position: JVec3,
        _contact_normal: JVec3,
        settings: &mut CharacterContactSettings,
    ) {
        if settings.can_push_character && self.body_itf.get_motion_type(*body2) != MotionType::Static {
            self.allow_sliding = true;
        }
    }

    fn on_contact_removed(&mut self, _character: &CharacterVirtual, _body2: &BodyID, _subshape2: &SubShapeID) {}

    fn on_character_contact_added(
        &mut self,
        _character: &CharacterVirtual,
        _other_character: &CharacterVirtual,
        _subshape2: &SubShapeID,
        _contact_position: JVec3,
        _contact_normal: JVec3,
        settings: &mut CharacterContactSettings,
    ) {
        if settings.can_push_character {
            self.allow_sliding = true;
        }
    }

    fn on_character_contact_persisted(
        &mut self,
        _character: &CharacterVirtual,
        _other_character: &CharacterVirtual,
        _subshape2: &SubShapeID,
        _contact_position: JVec3,
        _contact_normal: JVec3,
        settings: &mut CharacterContactSettings,
    ) {
        if settings.can_push_character {
            self.allow_sliding = true;
        }
    }

    fn on_character_contact_removed(
        &mut self,
        _character: &CharacterVirtual,
        _other_character: &CharacterVirtual,
        _subshape2: &SubShapeID,
    ) {
    }

    fn on_contact_solve(
        &mut self,
        character: &CharacterVirtual,
        _body2: &BodyID,
        _subshape2: &SubShapeID,
        _contact_position: JVec3,
        contact_normal: JVec3,
        contact_velocity: JVec3,
        _material: &PhysicsMaterial,
        _character_velocity: JVec3,
        new_character_velocity: &mut Vec3A,
    ) {
        let contact_normal: Vec3A = contact_normal.into();
        let contact_velocity: Vec3A = contact_velocity.into();
        if !self.allow_sliding
            && abs_diff_eq!(contact_velocity, Vec3A::ZERO)
            && !character.is_slope_too_steep(contact_normal)
        {
            *new_character_velocity = Vec3A::ZERO;
        }
    }

    fn on_character_contact_solve(
        &mut self,
        _character: &CharacterVirtual,
        _other_character: &CharacterVirtual,
        _subshape2: &SubShapeID,
        _contact_position: JVec3,
        _contact_normal: JVec3,
        _contact_velocity: JVec3,
        _material: &PhysicsMaterial,
        _character_velocity: JVec3,
        _new_character_velocity: &mut Vec3A,
    ) {
    }
}
