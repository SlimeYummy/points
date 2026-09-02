use crate::template::action::base::{TmplAnimation, TmplTimelineRange};
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{F32Range, RotationReference, TmplID, XResult};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionDodgeNpc {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character_npcs: Vec<TmplID>,
    pub tags: Vec<String>,
    pub move_distance: F32Range,
    pub anim_dodges: Vec<TmplActionDodgeNpcDodge>,
}

impl_tmpl!(TmplActionDodgeNpc, ActionDodgeNpc, "ActionDodgeNpc");

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionDodgeNpcDodge {
    pub anim: TmplAnimation,
    /// Dodge direction in XZ plane (right-hand, character space)
    pub enter_angle: f32,
    pub rotation_reference: RotationReference,
    pub rotation_start: f32,
    pub rotation_duration: F32Range,
    pub rotation_max_angle: f32,
    pub keep_levels: TmplTimelineRange<u16>,
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
pub struct TmplActionDodgeNpcRotation {
    pub reference: RotationReference,
    /// Rotation duration range [min, max]
    pub duration: F32Range,
    /// Angle to rotate
    pub max_angle: f32,
}

impl TmplActionDodgeNpcRotation {
    #[inline]
    pub fn from_rkyv(archived: &rkyv::Archived<TmplActionDodgeNpcRotation>) -> XResult<TmplActionDodgeNpcRotation> {
        Ok(TmplActionDodgeNpcRotation {
            reference: archived.reference,
            duration: archived.duration,
            max_angle: archived.max_angle.to_native(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{F32Range, LEVEL_ACTION, TimeRange, cf2s, id};

    #[test]
    fn test_load_action_dodge_npc() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let act = db.find_as::<TmplActionDodgeNpc>(id!("Action.Enemy.Dodge")).unwrap();

        assert_eq!(act.id, id!("Action.Enemy.Dodge"));
        assert_eq!(act.character_npcs.as_slice(), &[id!("CharacterNpc.Enemy")]);
        assert_eq!(act.tags.as_slice(), &["Dodge"]);
        assert_eq!(act.move_distance, F32Range::new(2.0, 5.0));
        assert_eq!(act.anim_dodges.len(), 2);

        assert_eq!(act.anim_dodges[0].anim.files, "Slime/Dodge_F.*");
        assert_eq!(act.anim_dodges[0].anim.duration, cf2s(110));
        assert_eq!(act.anim_dodges[0].enter_angle, 0f32.to_radians());
        assert_eq!(
            act.anim_dodges[0].rotation_reference,
            RotationReference::TargetCharacter
        );
        assert_eq!(act.anim_dodges[0].rotation_start, cf2s(84));
        assert_eq!(act.anim_dodges[0].rotation_duration, F32Range::new(cf2s(16), cf2s(24)));
        assert_eq!(act.anim_dodges[0].rotation_max_angle, 180f32.to_radians());
        assert_eq!(act.anim_dodges[0].keep_levels.fragments.len(), 1);
        assert_eq!(act.anim_dodges[0].keep_levels.values[0], LEVEL_ACTION);
        assert_eq!(
            act.anim_dodges[0].keep_levels.fragments[0].to_time_range(),
            TimeRange::new(0.0, cf2s(110))
        );

        assert_eq!(act.anim_dodges[1].anim.files, "Slime/Dodge_B.*");
        assert_eq!(act.anim_dodges[1].anim.duration, cf2s(110));
        assert_eq!(act.anim_dodges[1].enter_angle, 180f32.to_radians());
        assert_eq!(
            act.anim_dodges[1].rotation_reference,
            RotationReference::TargetCharacter
        );
        assert_eq!(act.anim_dodges[1].rotation_start, cf2s(84));
        assert_eq!(act.anim_dodges[1].rotation_duration, F32Range::new(cf2s(16), cf2s(24)));
        assert_eq!(act.anim_dodges[1].rotation_max_angle, 180f32.to_radians());
        assert_eq!(act.anim_dodges[1].keep_levels.fragments.len(), 1);
        assert_eq!(act.anim_dodges[1].keep_levels.values[0], LEVEL_ACTION);
        assert_eq!(
            act.anim_dodges[1].keep_levels.fragments[0].to_time_range(),
            TimeRange::new(0.0, cf2s(110))
        );
    }
}
