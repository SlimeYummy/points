use cirtical_point_csgen::CsOut;
use educe::Educe;
use glam::{Quat, Vec3, Vec3A};
use jolt_physics_rs::{
    self as jolt, BodyID, BodySettings, CapsuleSettings, CharacterVirtual, CharacterVirtualSettings, MotionType,
    RotatedTranslatedSettings, PHY_LAYER_BODY_PLAYER,
};
use std::cell::Cell;
use std::rc::Rc;

use crate::instance::InstPlayer;
use crate::logic::game::{ContextRestore, ContextUpdate, LogicSystems};
use crate::logic::system;
use crate::template::TmplCharacter;
use crate::utils::{NumID, XError, XResult};

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
pub struct StateCharaPhysics {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Educe)]
#[educe(Debug)]
pub(crate) struct LogicCharaPhysics {
    player_id: NumID,
    inst_player: Rc<InstPlayer>,
    position: Vec3,
    rotation: Quat,
    pub(crate) idle: bool,

    #[educe(Debug(ignore))]
    phy_chara: CharacterVirtual,
    phy_bounding: BodyID,

    pub new_position: Vec3,
    pub new_rotation: Quat,
}

impl LogicCharaPhysics {
    #[cfg(test)]
    pub(crate) fn mock(player_id: NumID, inst_player: Rc<InstPlayer>) -> LogicCharaPhysics {
        LogicCharaPhysics {
            player_id,
            inst_player,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            idle: true,
            phy_chara: unsafe { std::mem::zeroed() },
            phy_bounding: BodyID::invalid(),
            new_position: Vec3::ZERO,
            new_rotation: Quat::IDENTITY,
        }
    }

    pub fn new(
        ctx: &mut ContextUpdate<'_>,
        player_id: NumID,
        inst_player: Rc<InstPlayer>,
        position: Vec3,
        rotation: Quat,
    ) -> XResult<LogicCharaPhysics> {
        let tmpl_chara = ctx.tmpl_db.find_as::<TmplCharacter>(&inst_player.tmpl_character)?;

        let bounding = tmpl_chara.bounding_capsule;
        let chara_shape_settings = CapsuleSettings::new(bounding.half_height, bounding.radius);
        let chara_shape = jolt::create_shape_rotated_translated(&RotatedTranslatedSettings::new(
            jolt::create_shape_capsule(&chara_shape_settings)?,
            Vec3A::new(0.0, bounding.half_height + bounding.radius, 0.0),
            Quat::IDENTITY,
        ))?;
        let phy_chara = CharacterVirtual::new(
            &mut ctx.physics,
            &CharacterVirtualSettings::new(chara_shape),
            position.into(),
            rotation,
        );

        let bounding_shape_settings = CapsuleSettings::new(bounding.half_height, bounding.radius * 0.9);
        let bounding_shape = jolt::create_shape_rotated_translated(&RotatedTranslatedSettings::new(
            jolt::create_shape_capsule(&bounding_shape_settings)?,
            Vec3A::new(0.0, bounding.half_height + bounding.radius, 0.0),
            Quat::IDENTITY,
        ))?;
        let phy_bounding = ctx.body_itf.create_add_body(
            &BodySettings::new(
                bounding_shape,
                PHY_LAYER_BODY_PLAYER,
                MotionType::Kinematic,
                position.into(),
                rotation,
            ),
            false,
        )?;

        Ok(LogicCharaPhysics {
            player_id,
            inst_player,
            position,
            rotation,
            idle: true,
            phy_chara,
            phy_bounding,
            new_position: position,
            new_rotation: rotation,
        })
    }

    pub fn init(&mut self, _ctx: &mut ContextUpdate<'_>) -> XResult<StateCharaPhysics> {
        Ok(StateCharaPhysics {
            position: self.position,
            rotation: self.rotation,
        })
    }

    pub fn update(&mut self, _ctx: &mut ContextUpdate<'_>) -> XResult<StateCharaPhysics> {
        self.position = self.new_position;
        self.rotation = self.new_rotation;
        self.idle = false;
        Ok(StateCharaPhysics {
            position: self.position,
            rotation: self.rotation,
        })
    }

    pub fn restore(&mut self, _ctx: &ContextRestore, state: &StateCharaPhysics) -> XResult<()> {
        self.position = state.position;
        self.rotation = state.rotation;
        self.new_position = state.position;
        self.new_rotation = state.rotation;
        Ok(())
    }
}

impl LogicCharaPhysics {
    #[inline]
    fn position(&self) -> Vec3 {
        self.position
    }

    #[inline]
    fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn is_idle(&self) -> bool {
        self.idle
    }
}
