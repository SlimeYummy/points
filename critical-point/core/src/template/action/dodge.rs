use crate::template::action::base::{TmplAnimation, TmplDeriveRule};
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{F32Range, TmplID, VirtualKey};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionDodgeTimes {
    pub quick: f32,
    pub normal: f32,
}

impl TmplActionDodgeTimes {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionDodgeTimes) -> TmplActionDodgeTimes {
        TmplActionDodgeTimes {
            quick: archived.quick.into(),
            normal: archived.normal.into(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionDodgeKeepLevels {
    pub dodge: u16,
    pub quick: u16,
    pub normal: u16,
}

impl TmplActionDodgeKeepLevels {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionDodgeKeepLevels) -> TmplActionDodgeKeepLevels {
        TmplActionDodgeKeepLevels {
            dodge: archived.dodge.into(),
            quick: archived.quick.into(),
            normal: archived.normal.into(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionDodge {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    #[serde(default)]
    pub character: TmplID,
    #[serde(default)]
    pub styles: Vec<TmplID>,
    #[serde(default)]
    pub character_npcs: Vec<TmplID>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub enter_key: Option<VirtualKey>,
    pub enter_level: u16,
    pub anim_dodge: TmplAnimation,
    #[serde(default)]
    pub smooth_move_froms: Vec<TmplID>,
    #[serde(default)]
    pub smooth_no_leg_fade_in: f32,
    #[serde(default)]
    pub smooth_speed_duration: f32,
    pub dodge_time: F32Range,
    pub derive_times: TmplActionDodgeTimes,
    pub keep_levels: TmplActionDodgeKeepLevels,
    #[serde(default)]
    pub derives: Vec<TmplDeriveRule>,
}

impl_tmpl!(TmplActionDodge, ActionDodge, "ActionDodge");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{LEVEL_ACTION, LEVEL_DERIVE, LEVEL_MOVE, cf2s, id};

    #[test]
    fn test_load_action_dodge() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let act = db.find_as::<TmplActionDodge>(id!("Action.One.Dodge")).unwrap();

        assert_eq!(act.id, id!("Action.One.Dodge"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One^1")]);
        assert_eq!(act.tags.as_slice(), &["Dodge"]);
        assert_eq!(act.enter_key, Some(VirtualKey::Dodge));
        assert_eq!(act.enter_level, LEVEL_DERIVE);

        assert_eq!(act.anim_dodge.files, "Girl/Dodge_F_Empty.*");
        assert_eq!(act.anim_dodge.duration, cf2s(72));
        assert_eq!(act.anim_dodge.fade_in, 0.1);
        assert_eq!(act.anim_dodge.root_motion, true);
        assert_eq!(act.anim_dodge.weapon_control, false);
        assert_eq!(act.anim_dodge.hit_motion, false);
        assert_eq!(act.anim_dodge.shape_key, false);
        assert_eq!(act.anim_dodge.additive_blending, false);

        assert_eq!(act.smooth_move_froms.as_slice(), &[id!("Action.One.Run")]);
        assert_eq!(act.smooth_no_leg_fade_in, cf2s(8));
        assert_eq!(act.smooth_speed_duration, cf2s(10));
        assert_eq!(act.dodge_time, F32Range::new(cf2s(6), cf2s(24)));
        assert_eq!(act.derive_times.quick, cf2s(34));
        assert_eq!(act.derive_times.normal, cf2s(48));
        assert_eq!(act.keep_levels.dodge, LEVEL_ACTION);
        assert_eq!(act.keep_levels.quick, LEVEL_DERIVE);
        assert_eq!(act.keep_levels.normal, LEVEL_MOVE);
    }
}
