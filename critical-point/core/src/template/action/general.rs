use crate::template::action::base::{TmplActionAttributes, TmplAnimation, TmplDeriveRule, TmplTimelineRange};
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::template::TmplTimelinePoint;
use crate::utils::{Bitsetable, DeriveContinue, EnumBitset, TmplID, VirtualKeyDir, XResult};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionGeneral {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character: TmplID,
    pub styles: Vec<TmplID>,
    pub tags: Vec<String>,
    pub anim_main: TmplAnimation,
    pub enter_key: Option<VirtualKeyDir>,
    pub enter_level: u16,
    pub cool_down_time: TmplVar<f32>,
    pub cool_down_round: TmplVar<u16>,
    pub cool_down_init_round: TmplVar<u16>,
    #[serde(default)]
    pub input_root_motion: Option<TmplActionGeneralRootMotion>,
    #[serde(default)]
    pub input_movements: TmplTimelinePoint<TmplActionGeneralMovement>,
    pub attributes: TmplTimelineRange<TmplActionAttributes>,
    pub derive_levels: TmplTimelineRange<TmplVar<u16>>,
    #[serde(default)]
    pub derives: Vec<TmplDeriveRule>,
    #[serde(default)]
    pub derive_continues: EnumBitset<DeriveContinue, { DeriveContinue::LEN }>,
    #[serde(default)]
    pub custom_events: TmplTimelinePoint<String>,
}

impl_tmpl!(TmplActionGeneral, ActionGeneral, "ActionGeneral");

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
#[serde(tag = "T")]
pub enum TmplActionGeneralMovement {
    RootMotion(TmplActionGeneralRootMotion),
    Rotation(TmplActionGeneralRotation),
}

impl TmplActionGeneralMovement {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionGeneralMovement) -> XResult<TmplActionGeneralMovement> {
        let movement = match archived {
            ArchivedTmplActionGeneralMovement::RootMotion(archived) => {
                TmplActionGeneralMovement::RootMotion(TmplActionGeneralRootMotion::from_rkyv(archived)?)
            }
            ArchivedTmplActionGeneralMovement::Rotation(archived) => {
                TmplActionGeneralMovement::Rotation(TmplActionGeneralRotation::from_rkyv(archived)?)
            }
        };
        Ok(movement)
    }
}

#[derive(
    Default,
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
pub struct TmplActionGeneralRootMotion {
    #[serde(rename = "move")]
    pub mov: bool,
    #[serde(rename = "move_ex")]
    pub mov_ex: bool,
}

impl TmplActionGeneralRootMotion {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionGeneralRootMotion) -> XResult<TmplActionGeneralRootMotion> {
        Ok(TmplActionGeneralRootMotion {
            mov: archived.mov,
            mov_ex: archived.mov_ex,
        })
    }
}

#[derive(
    Default,
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
pub struct TmplActionGeneralRotation {
    pub duration: f32,
    pub angle: f32,
}

impl TmplActionGeneralRotation {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionGeneralRotation) -> XResult<TmplActionGeneralRotation> {
        Ok(TmplActionGeneralRotation {
            duration: archived.duration.into(),
            angle: archived.angle.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{cf2s, id, InputDir, VirtualKey, LEVEL_ACTION, LEVEL_ATTACK};

    #[test]
    fn test_load_action_general() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionGeneral>(id!("Action.One.Attack^1")).unwrap();
        assert_eq!(act.id, id!("Action.One.Attack^1"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One^1"), id!("Style.One^2")]);
        assert_eq!(act.tags.as_slice(), &["Attack"]);
        assert_eq!(act.anim_main.files, "Girl_Attack_01A.*");
        assert_eq!(act.anim_main.duration, 4.0);
        assert_eq!(act.anim_main.fade_in, 0.1);
        assert_eq!(act.anim_main.root_motion, true);
        assert_eq!(act.enter_key, Some(VirtualKeyDir::new(VirtualKey::Attack1, None)));
        assert_eq!(act.enter_level, LEVEL_ATTACK);
        assert_eq!(act.cool_down_time.value().unwrap(), 0.0);
        assert_eq!(act.cool_down_round.value().unwrap(), 1);
        assert_eq!(act.cool_down_init_round.value().unwrap(), 1);
        // assert_eq!(TmplActionGeneralRootMotion::from_rkyv(&act.input_root_motion), TmplActionGeneralRootMotion {
        //     in_place: 0.5,
        //     normal: 0.5,
        //     extended: 1.0,
        // });
        assert_eq!(act.input_movements.pairs.len(), 2);
        assert_eq!(act.input_movements.pairs[0].0, 0.0);
        assert_eq!(
            TmplActionGeneralMovement::from_rkyv(&act.input_movements.pairs[0].1).unwrap(),
            TmplActionGeneralMovement::Rotation(TmplActionGeneralRotation {
                duration: cf2s(8),
                angle: 45.0 * std::f32::consts::PI / 180.0,
            })
        );
        assert_eq!(act.input_movements.pairs[1].0, cf2s(20));
        assert_eq!(
            TmplActionGeneralMovement::from_rkyv(&act.input_movements.pairs[1].1).unwrap(),
            TmplActionGeneralMovement::RootMotion(TmplActionGeneralRootMotion {
                mov: true,
                mov_ex: false,
            })
        );
        assert_eq!(act.attributes.fragments.len(), 1);
        assert_eq!(act.attributes.values.len(), 1);
        assert_eq!(act.attributes.values[0].damage_rdc.value().unwrap(), 0.2);
        assert_eq!(act.attributes.values[0].shield_dmg_rdc.value().unwrap(), 0.0);
        assert_eq!(act.attributes.values[0].poise_level.value().unwrap(), 1);
        assert_eq!(act.derive_levels.fragments.len(), 2);
        assert_eq!(act.derive_levels.values.len(), 2);
        assert_eq!(act.derive_levels.values[0].value().unwrap(), LEVEL_ACTION);
        assert_eq!(act.derive_levels.values[1].value().unwrap(), LEVEL_ATTACK);
        assert_eq!(act.derives.len(), 2);
        assert_eq!(act.derives[0].key.key, VirtualKey::Attack1);
        assert!(act.derives[0].key.dir.is_none());
        assert_eq!(act.derives[0].level, LEVEL_ATTACK + 1);
        assert_eq!(act.derives[0].action.value().unwrap(), id!("Action.One.Attack^2"));
        assert_eq!(act.derives[1].key.key, VirtualKey::Attack2);
        assert_eq!(act.derives[1].key.dir.unwrap(), InputDir::Forward(0.5));
        assert_eq!(act.derives[1].level, LEVEL_ATTACK + 1);
        assert_eq!(act.derives[1].action.value().unwrap(), id!("Action.One.Attack^2"));
        assert!(act.derive_continues.is_empty());
        assert_eq!(act.custom_events.pairs.len(), 1);
        assert_eq!(act.custom_events.pairs[0].0, 1.0);
        assert_eq!(act.custom_events.pairs[0].1, "CustomEvent");
    }
}
