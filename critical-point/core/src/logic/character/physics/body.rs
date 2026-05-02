use approx::{abs_diff_eq, abs_diff_ne, assert_abs_diff_eq};
use core::f32;
use glam::{Quat, Vec3, Vec3A};
use glam_ext::{Isometry3A, Transform3A};
use jolt_physics_rs::{
    self as jolt, Body, BodyCreationSettings, BodyID, BodyInterface, Character, CharacterContactListener,
    CharacterContactListenerVTable, CharacterContactSettings, CharacterSettings, CharacterVirtual,
    CharacterVirtualSettings, ExtendedUpdateSettings, GroundState, JMut, JRef, JVec3, MotionType, MutableCompoundShape,
    MutableCompoundShapeSettings, PhysicsMaterial, Plane, SubShapeID, SubShapeSettings, vdata,
};
use ozz_animation_rs::SKELETON_NO_PARENT;

use crate::animation::rest_poses_to_model_transforms;
use crate::consts::SPF;
use crate::instance::InstCharacter;
use crate::logic::character::control::LogicCharaControl;
use crate::logic::character::physics::physics::{CharacterHandle, CharacterLocation, JointBinding, LogicCharaPhysics};
use crate::logic::game::ContextUpdate;
use crate::logic::physics::{PhyBodyUserData, phy_layer};
use crate::utils::{NumID, XResult, quat_from_dir_xz, xerrf, xfrom};

const CHARACTER_RADIUS_STANDING: f32 = -0.3;

impl LogicCharaPhysics {
    pub(super) fn init_bounding(
        ctx: &mut ContextUpdate,
        inst_chara: &InstCharacter,
        position: Vec3A,
        rotation: Quat,
    ) -> XResult<CharacterHandle> {
        let charc_phy = ctx.asset.load_character_physics(inst_chara.skeleton_files)?;

        if inst_chara.is_player {
            // Use VirtualCharacter for player

            let mut settings = CharacterVirtualSettings::new(charc_phy.bounding);
            settings.max_slope_angle = 45f32.to_radians();
            settings.supporting_volume = Plane::new(Vec3::Y, CHARACTER_RADIUS_STANDING);

            let mut character = CharacterVirtual::new(&mut ctx.physics, &settings, position, rotation);
            character.set_listener(Some(CharacterContactListenerImpl::new_vbox(
                CharacterContactListenerImpl {
                    allow_sliding: false,
                    body_itf: unsafe { ctx.physics.steal_body_itf() },
                },
            )));
            Ok(CharacterHandle::Player(character))
        }
        else {
            // Use Character for NPC

            let mut settings = CharacterSettings::new(charc_phy.bounding, phy_layer!(Bounding, All));
            settings.max_slope_angle = 45f32.to_radians();
            settings.friction = 0.5;
            settings.supporting_volume = Plane::new(Vec3::Y, CHARACTER_RADIUS_STANDING);

            let character = Character::new_add(&mut ctx.physics, &settings, position, rotation, 0, true, false);
            Ok(CharacterHandle::Npc(character))
        }
    }

    pub(super) fn update_bounding(&mut self, ctx: &mut ContextUpdate, chara_ctrl: &LogicCharaControl) -> XResult<()> {
        let location = match &mut self.character {
            CharacterHandle::Player(character) => Self::update_player_character(character, ctx, chara_ctrl)?,
            CharacterHandle::Npc(character) => Self::update_npc_character(character, ctx, chara_ctrl)?,
        };

        ctx.physics
            .body_itf()
            .set_position(self.target_body, location.position, true);

        self.velocity = location.velocity;

        self.position = location.position;
        assert_abs_diff_eq!(location.rotation.x, 0.0, epsilon = 0.01);
        assert_abs_diff_eq!(location.rotation.z, 0.0, epsilon = 0.01);

        self.direction = chara_ctrl.new_direction();
        self.rotation = quat_from_dir_xz(self.direction);
        self.idle.set(true);
        Ok(())
    }

    pub(super) fn update_player_character(
        character: &mut JMut<CharacterVirtual<CharacterContactListenerImpl>>,
        ctx: &mut ContextUpdate,
        chara_ctrl: &LogicCharaControl,
    ) -> XResult<CharacterLocation> {
        let new_rotation = quat_from_dir_xz(chara_ctrl.new_direction());
        if new_rotation != character.get_rotation() {
            character.set_rotation(new_rotation);
        }
        character.get_listener_mut().unwrap().allow_sliding = abs_diff_ne!(chara_ctrl.new_velocity(), Vec3A::ZERO);

        character.update_ground_velocity();

        let gravity: Vec3A = ctx.physics.get_gravity();
        let linear_velocity: Vec3A = character.get_linear_velocity();
        let ground_velocity: Vec3A = character.get_ground_velocity();
        let moving_towards_ground = (linear_velocity.y - ground_velocity.y) < 0.1;

        if abs_diff_eq!(chara_ctrl.new_velocity().y, 0.0) {
            let mut new_velocity;
            if character.get_ground_state() == GroundState::OnGround && moving_towards_ground {
                new_velocity = ground_velocity;
            }
            else {
                new_velocity = Vec3A::new(0.0, linear_velocity.y, 0.0);
            }

            new_velocity += gravity * SPF; // Gravity

            if character.is_supported() {
                new_velocity += chara_ctrl.new_velocity();
            }
            else {
                let horizontal_velocity = linear_velocity - Vec3A::new(0.0, linear_velocity.y, 0.0);
                new_velocity += horizontal_velocity;
            }
            character.set_linear_velocity(new_velocity);
        }
        else {
            character.set_linear_velocity(chara_ctrl.new_velocity());
        }

        character.extended_update(
            phy_layer!(Bounding, Player),
            SPF,
            gravity.into(),
            &ExtendedUpdateSettings::default(),
        );

        Ok(CharacterLocation {
            position: character.get_position(),
            rotation: character.get_rotation(),
            velocity: character.get_linear_velocity(),
        })
    }

    pub(super) fn update_npc_character(
        character: &mut JMut<Character>,
        ctx: &mut ContextUpdate,
        chara_ctrl: &LogicCharaControl,
    ) -> XResult<CharacterLocation> {
        character.set_linear_velocity(chara_ctrl.new_velocity(), false);

        Ok(CharacterLocation {
            position: character.get_position(false),
            rotation: character.get_rotation(false),
            velocity: character.get_linear_velocity(false),
        })
    }

    pub(super) fn init_bodies(
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

    pub(super) fn update_bodies(&mut self, ctx: &mut ContextUpdate, chara_ctrl: &LogicCharaControl) -> XResult<()> {
        let model_transforms = chara_ctrl.model_transforms();
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
}

#[vdata(CharacterContactListenerVTable)]
pub(super) struct CharacterContactListenerImpl {
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
