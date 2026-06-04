use crate::template::action::base::{TmplActionAttributes, TmplAnimation, TmplTimelineRange};
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::template::{TmplHit, TmplTimelinePoint};
use crate::utils::{F32Range, TmplID, XResult};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionGeneralNpc {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character_npcs: Vec<TmplID>,
    pub tags: Vec<String>,
    pub anim_main: TmplAnimation,
    #[serde(default)]
    pub adjust_movements: TmplTimelinePoint<TmplActionGeneralNpcMovement>,
    // pub attributes: TmplTimelineRange<TmplActionAttributes>,
    pub keep_levels: TmplTimelineRange<u16>,
    #[serde(default)]
    pub hits: Vec<TmplHit>,
    #[serde(default)]
    pub custom_events: TmplTimelinePoint<String>,
}

impl_tmpl!(TmplActionGeneralNpc, ActionGeneralNpc, "ActionGeneralNpc");

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
pub enum TmplActionGeneralNpcMovement {
    Translation(TmplActionGeneralNpcTranslation),
    Rotation(TmplActionGeneralNpcRotation),
}

impl TmplActionGeneralNpcMovement {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionGeneralNpcMovement) -> XResult<TmplActionGeneralNpcMovement> {
        let movement = match archived {
            ArchivedTmplActionGeneralNpcMovement::Translation(archived) => {
                TmplActionGeneralNpcMovement::Translation(TmplActionGeneralNpcTranslation::from_rkyv(archived)?)
            }
            ArchivedTmplActionGeneralNpcMovement::Rotation(archived) => {
                TmplActionGeneralNpcMovement::Rotation(TmplActionGeneralNpcRotation::from_rkyv(archived)?)
            }
        };
        Ok(movement)
    }
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
pub struct TmplActionGeneralNpcTranslation {
    /// Translation duration
    pub duration: f32,
    /// Fade (in/out) ratio in [0.0, 0.5]
    pub fade_ratio: f32,
    /// Distance range from NPC to it's target
    pub distance: F32Range,
    /// Speed ratio range
    pub speed_ratio: F32Range,
}

impl TmplActionGeneralNpcTranslation {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionGeneralNpcTranslation) -> XResult<TmplActionGeneralNpcTranslation> {
        Ok(TmplActionGeneralNpcTranslation {
            duration: archived.duration.to_native(),
            fade_ratio: archived.fade_ratio.to_native(),
            distance: archived.distance,
            speed_ratio: archived.speed_ratio,
        })
    }
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
pub struct TmplActionGeneralNpcRotation {
    /// Rotation duration
    pub duration: f32,
    /// Angle to rotate
    pub max_angle: f32,
}

impl TmplActionGeneralNpcRotation {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionGeneralNpcRotation) -> XResult<TmplActionGeneralNpcRotation> {
        Ok(TmplActionGeneralNpcRotation {
            duration: archived.duration.to_native(),
            max_angle: archived.max_angle.to_native(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{F32Range, LEVEL_ACTION, LEVEL_ATTACK, TimeRange, cf2s, id};

    #[test]
    fn test_load_action_general_npc() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionGeneralNpc>(id!("Action.Enemy.Attack")).unwrap();
        assert_eq!(act.id, id!("Action.Enemy.Attack"));
        assert_eq!(act.character_npcs.as_slice(), &[id!("CharacterNpc.Enemy")]);
        assert_eq!(act.tags.as_slice(), &["Attack"]);

        assert_eq!(act.anim_main.files, "Slime/Attack1A.*");
        assert_eq!(act.anim_main.duration, cf2s(206));
        assert_eq!(act.anim_main.fade_in, 0.1);
        assert!(act.anim_main.root_motion);
        assert!(!act.anim_main.weapon_motion);
        assert!(!act.anim_main.hit_motion);

        assert_eq!(act.keep_levels.fragments.len(), 2);
        assert_eq!(
            act.keep_levels.fragments[0].to_time_range(),
            TimeRange::new(0.0, cf2s(150))
        );
        assert_eq!(act.keep_levels.values[0], LEVEL_ACTION);
        assert_eq!(
            act.keep_levels.fragments[1].to_time_range(),
            TimeRange::new(cf2s(150), cf2s(206))
        );
        assert_eq!(act.keep_levels.values[1], LEVEL_ATTACK);

        assert_eq!(act.adjust_movements.pairs.len(), 2);

        assert_eq!(act.adjust_movements.pairs[0].0, 0.0);
        assert_eq!(
            TmplActionGeneralNpcMovement::from_rkyv(&act.adjust_movements.pairs[0].1).unwrap(),
            TmplActionGeneralNpcMovement::Rotation(TmplActionGeneralNpcRotation {
                duration: cf2s(8),
                max_angle: 45.0 * std::f32::consts::PI / 180.0,
            })
        );
        assert_eq!(act.adjust_movements.pairs[1].0, cf2s(20));
        assert_eq!(
            TmplActionGeneralNpcMovement::from_rkyv(&act.adjust_movements.pairs[1].1).unwrap(),
            TmplActionGeneralNpcMovement::Translation(TmplActionGeneralNpcTranslation {
                duration: cf2s(20),
                fade_ratio: 0.1,
                distance: F32Range::new(2.0, 5.0),
                speed_ratio: F32Range::new(0.8, 1.5),
            })
        );

        assert!(act.hits.is_empty());

        assert_eq!(act.custom_events.pairs.len(), 1);
        assert_eq!(act.custom_events.pairs[0].0, cf2s(60));
        assert_eq!(act.custom_events.pairs[0].1, "CustomEvent");
    }
}
