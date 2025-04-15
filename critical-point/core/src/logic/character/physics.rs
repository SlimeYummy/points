use cirtical_point_csgen::CsOut;
use core::f32;
use educe::Educe;
use glam_ext::{Isometry3A, Quat, Vec2, Vec3, Vec3A};
use jolt_physics_rs::{
    self as jolt, vdata, Body, BodyCreationSettings, BodyID, BodyInterface, CapsuleShapeSettings,
    CharacterContactListener, CharacterContactListenerVTable, CharacterContactSettings, CharacterVirtual,
    CharacterVirtualSettings, ExtendedUpdateSettings, GroundState, JMut, JRef, JVec3, MotionType, MutableCompoundShape,
    MutableCompoundShapeSettings, PhysicsMaterial, RotatedTranslatedShapeSettings, SubShapeID, SubShapeSettings,
};
use ozz_animation_rs::SKELETON_NO_PARENT;
use std::cell::Cell;
use std::rc::Rc;

use super::LogicCharaAction;
use crate::animation::rest_poses_to_model_transforms;
use crate::consts::FPS;
use crate::instance::InstPlayer;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::physics::PHY_LAYER_PLAYER;
use crate::template::TmplCharacter;
use crate::utils::{near, to_euler_degree, xerrf, xfrom, CsQuat, CsVec3A, NumID, Symbol, XResult};

#[repr(C)]
#[derive(Debug, Default, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
pub struct StateCharaPhysics {
    pub velocity: CsVec3A,
    pub position: CsVec3A,
    pub rotation: CsQuat,
}

#[derive(Educe)]
#[educe(Debug)]
pub(crate) struct LogicCharaPhysics {
    player_id: NumID,
    inst_player: Rc<InstPlayer>,
    velocity: Vec3A,
    position: Vec3A,
    rotation: Quat,
    pub(crate) idle: Cell<bool>,

    #[educe(Debug(ignore))]
    character: JMut<CharacterVirtual<CharacterContactListenerImpl>>,
    target_body: BodyID,
    target_shape: JMut<MutableCompoundShape>,
    target_bindings: Vec<JointBinding>,

    cache_isometries: Vec<Isometry3A>,
}

#[derive(Debug, Clone)]
struct JointBinding {
    position: Vec3A,
    rotation: Quat,
    part: Symbol,
    joint: i16,
    ratio: f32,
    joint2: i16,
}

impl LogicCharaPhysics {
    #[cfg(test)]
    pub(crate) fn mock(player_id: NumID, inst_player: Rc<InstPlayer>) -> LogicCharaPhysics {
        use std::mem;
        LogicCharaPhysics {
            player_id,
            inst_player,
            velocity: Vec3A::ZERO,
            position: Vec3A::ZERO,
            rotation: Quat::IDENTITY,
            idle: Cell::new(true),
            character: unsafe { mem::zeroed() },
            target_body: BodyID::INVALID,
            target_shape: unsafe { mem::zeroed() },
            target_bindings: Vec::new(),
            cache_isometries: Vec::new(),
        }
    }

    pub fn new(
        ctx: &mut ContextUpdate<'_>,
        player_id: NumID,
        inst_player: Rc<InstPlayer>,
        position: Vec3A,
        rotation: Quat,
    ) -> XResult<LogicCharaPhysics> {
        let tmpl_chara = ctx.tmpl_db.find_as::<TmplCharacter>(&inst_player.tmpl_character)?;
        let character = LogicCharaPhysics::init_bounding(ctx, &tmpl_chara, position, rotation)?;
        let (target_body, target_shape, target_bindings) =
            LogicCharaPhysics::init_target(ctx, &tmpl_chara, position, rotation)?;

        let target_bindings_len = target_bindings.len();
        Ok(LogicCharaPhysics {
            player_id,
            inst_player,
            velocity: Vec3A::ZERO,
            position,
            rotation,
            idle: Cell::new(true),
            character,
            target_body,
            target_shape,
            target_bindings,
            cache_isometries: Vec::with_capacity(target_bindings_len),
        })
    }

    pub fn state(&self) -> StateCharaPhysics {
        StateCharaPhysics {
            velocity: self.velocity.into(),
            position: self.position.into(),
            rotation: self.rotation.into(),
        }
    }

    pub fn restore(&mut self, _ctx: &ContextRestore, state: &StateCharaPhysics) -> XResult<()> {
        self.position = state.position.into();
        self.rotation = state.rotation.into();
        Ok(())
    }

    fn init_bounding(
        ctx: &mut ContextUpdate<'_>,
        tmpl_chara: &TmplCharacter,
        position: Vec3A,
        rotation: Quat,
    ) -> XResult<JMut<CharacterVirtual<CharacterContactListenerImpl>>> {
        let bounding = tmpl_chara.bounding_capsule;
        let mut chara_shape =
            jolt::create_capsule_shape(&CapsuleShapeSettings::new(bounding.half_height, bounding.radius))
                .map_err(xfrom!())?;
        chara_shape = jolt::create_rotated_translated_shape(&RotatedTranslatedShapeSettings::new(
            chara_shape,
            Vec3A::new(0.0, bounding.half_height + bounding.radius, 0.0),
            Quat::IDENTITY,
        ))
        .map_err(xfrom!())?;

        let mut character = CharacterVirtual::new(
            &mut ctx.physics,
            &CharacterVirtualSettings::new(chara_shape),
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

    fn init_target(
        ctx: &mut ContextUpdate<'_>,
        tmpl_chara: &TmplCharacter,
        position: Vec3A,
        rotation: Quat,
    ) -> XResult<(BodyID, JMut<MutableCompoundShape>, Vec<JointBinding>)> {
        let skeleton = ctx.asset.load_skeleton(&tmpl_chara.skeleton)?;
        let mut loaded_target_box = ctx.asset.load_target_box(&tmpl_chara.target_box)?;

        let model_transforms = rest_poses_to_model_transforms(&skeleton)?;
        let mut target_bindings = Vec::with_capacity(loaded_target_box.bindings.len());
        let mut sub_shape_settings = Vec::with_capacity(loaded_target_box.bindings.len());

        for loaded_binding in loaded_target_box.bindings.drain(..) {
            let joint = skeleton.joint_by_name(loaded_binding.joint.as_str()).ok_or_else(
                || xerrf!(BadAsset; "target_box={} joint={}", &tmpl_chara.target_box, &loaded_binding.joint),
            )?;

            let mut joint2 = SKELETON_NO_PARENT as i16;
            if !loaded_binding.joint2.is_empty() {
                joint2 = skeleton.joint_by_name(loaded_binding.joint2.as_str()).ok_or_else(
                    || xerrf!(BadAsset; "target_box={} joint2={}", &tmpl_chara.target_box, &loaded_binding.joint2),
                )?
            };

            target_bindings.push(JointBinding {
                position: loaded_binding.position.into(),
                rotation: loaded_binding.rotation,
                part: loaded_binding.part,
                joint,
                ratio: loaded_binding.ratio,
                joint2,
            });

            let transform = model_transforms[joint as usize];
            let rotation = transform.rotation * loaded_binding.rotation;
            let position = if joint2 < 0 {
                transform.translation + transform.rotation * loaded_binding.position
            } else {
                Vec3A::lerp(
                    transform.translation,
                    model_transforms[joint2 as usize].translation,
                    loaded_binding.ratio,
                ) + transform.rotation * loaded_binding.position
            };

            println!(
                "{} >> pos:{:?}  rot:{:?}  ozz.pos:{:?}  ozz.rot:{:?}  joint.pos:{:?}",
                joint,
                loaded_binding.position,
                crate::utils::to_euler_degree(loaded_binding.rotation),
                position,
                crate::utils::to_euler_degree(rotation),
                transform.translation
            );

            sub_shape_settings.push(SubShapeSettings::new(loaded_binding.shape, position, rotation));
        }

        let target_box_settings = MutableCompoundShapeSettings::new(&sub_shape_settings);
        let mut target_shape = jolt::create_mutable_compound_shape_mut(&target_box_settings).map_err(xfrom!())?;
        let target_body = ctx
            .physics
            .body_itf()
            .create_add_body(
                &BodyCreationSettings::new(
                    unsafe { target_shape.steal_ref().into() },
                    PHY_LAYER_PLAYER,
                    MotionType::Kinematic,
                    position.into(),
                    rotation,
                ),
                false,
            )
            .map_err(xfrom!())?;

        Ok((target_body, target_shape, target_bindings))
    }

    pub fn update(&mut self, ctx: &mut ContextUpdate<'_>, action: &LogicCharaAction) -> XResult<()> {
        self.update_bounding(ctx, action)?;
        self.update_target_box(ctx, action)?;
        Ok(())
    }

    fn update_bounding(&mut self, ctx: &mut ContextUpdate<'_>, action: &LogicCharaAction) -> XResult<()> {
        // println!("action.new_rotation() {:?}", action.new_rotation());
        if action.new_rotation() != self.character.get_rotation() {
            self.character.set_rotation(action.new_rotation());
        }
        self.character.get_listener_mut().unwrap().allow_sliding = !near!(action.new_velocity(), Vec3A::ZERO);

        self.character.update_ground_velocity();

        let gravity: Vec3A = ctx.physics.get_gravity();
        let linear_velocity: Vec3A = self.character.get_linear_velocity();
        let ground_velocity: Vec3A = self.character.get_ground_velocity();
        let moving_towards_ground = (linear_velocity.y - ground_velocity.y) < 0.1;

        let mut new_velocity;
        if self.character.get_ground_state() == GroundState::OnGround && moving_towards_ground {
            new_velocity = ground_velocity;
        } else {
            new_velocity = Vec3A::new(0.0, linear_velocity.y, 0.0);
        }

        new_velocity += gravity * (1.0 / (FPS as f32)); // Gravity

        if self.character.is_supported() {
            new_velocity += action.new_velocity();
        } else {
            let horizontal_velocity = linear_velocity - Vec3A::new(0.0, linear_velocity.y, 0.0);
            new_velocity += horizontal_velocity;
        }
        self.character.set_linear_velocity(new_velocity.into());

        self.character.extended_update(
            PHY_LAYER_PLAYER,
            1.0 / (FPS as f32),
            gravity.into(),
            &ExtendedUpdateSettings::default(),
        );

        ctx.physics
            .body_itf()
            .set_position(self.target_body, self.character.get_position(), true);

        self.velocity = self.character.get_linear_velocity();
        self.position = self.character.get_position();
        self.rotation = self.character.get_rotation();
        self.idle.set(true);
        Ok(())
    }

    fn update_target_box(&mut self, ctx: &mut ContextUpdate<'_>, action: &LogicCharaAction) -> XResult<()> {
        self.cache_isometries.clear();

        let model_transforms = action.model_transforms();
        for binding in &self.target_bindings {
            let transform = model_transforms[binding.joint as usize];
            let rotation = transform.rotation * binding.rotation;
            let position = if binding.joint2 < 0 {
                transform.translation + transform.rotation * binding.position
            } else {
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

        ctx.physics
            .body_itf()
            .set_position_rotation(self.target_body, self.position, self.rotation, true);
        Ok(())
    }

    #[inline]
    pub fn position(&self) -> Vec3A {
        self.position
    }

    #[inline]
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    #[inline]
    pub fn direction(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    #[inline]
    pub fn direction_2d(&self) -> Vec2 {
        let rot_z = self.rotation * Vec3::Z;
        println!("rot_z {:?} rotation {:?}", rot_z, to_euler_degree(self.rotation));
        if rot_z.y.abs() < 0.99 {
            Vec2::new(rot_z.x, rot_z.z).normalize()
        } else {
            let rot_y = self.rotation * Vec3::Y;
            Vec2::new(rot_y.x, rot_y.z).normalize()
        }
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.idle.get()
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
        // println!("{} {} {}", self.allow_sliding, near!(contact_velocity, Vec3A::ZERO), !character.is_slope_too_steep(contact_normal));
        if !self.allow_sliding && near!(contact_velocity, Vec3A::ZERO) && !character.is_slope_too_steep(contact_normal)
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
