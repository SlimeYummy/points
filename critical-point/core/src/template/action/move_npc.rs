use crate::template::action::base::TmplAnimation;
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{TmplID, VirtualKey};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveNpc {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character_npcs: Vec<TmplID>,
    pub tags: Vec<String>,
    pub enter_key: VirtualKey,
    pub poise_level: u16,
    pub anim_move: TmplAnimation,
    pub move_speed: f32,
    pub speed_ratio: f32,
    pub anim_start: TmplAnimation,
    pub stops: Vec<TmplActionMoveNpcStop>,
    pub turn_time: f32,
    pub min_distance: f32,
    pub step_length: f32,
}

impl_tmpl!(TmplActionMoveNpc, ActionMoveNpc, "ActionMoveNpc");

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveNpcStop {
    pub anim: TmplAnimation,
    pub enter_from_table: Vec<TmplActionMoveNpcStopFrom>,
}

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveNpcStopFrom {
    /// From which animation to enter stop.
    pub anim: String,

    /// The ratio of the animation to enter stop.
    pub ratio: f32,
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{cf2s, id};

    #[test]
    fn test_load_action_move_npc() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionMoveNpc>(id!("Action.Enemy.Walk")).unwrap();
        assert_eq!(act.id, id!("Action.Enemy.Walk"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character_npcs.as_slice(), &[id!("CharacterNpc.Enemy")]);
        assert_eq!(act.tags.as_slice(), &["Walk"]);
        assert_eq!(act.enter_key, VirtualKey::Walk);
        assert_eq!(act.poise_level, 0);

        assert_eq!(act.anim_move.files, "Slime/WalkLoop.*");
        assert_eq!(act.anim_move.duration, cf2s(80));
        assert_eq!(act.anim_move.fade_in, 0.1);
        assert_eq!(act.anim_move.root_motion, true);
        assert_eq!(act.anim_move.weapon_motion, false);
        assert_eq!(act.anim_move.hit_motion, false);
        assert_eq!(act.move_speed, 1.5);

        assert_eq!(act.anim_start.files, "Slime/WalkStart.*");
        assert_eq!(act.anim_start.duration, cf2s(40));
        assert_eq!(act.anim_start.fade_in, 0.1);
        assert_eq!(act.anim_start.root_motion, true);
        assert_eq!(act.anim_start.weapon_motion, false);
        assert_eq!(act.anim_start.hit_motion, false);

        assert_eq!(act.stops.len(), 1);
        assert_eq!(act.stops[0].anim.files, "Slime/WalkStop.*");
        assert_eq!(act.stops[0].anim.duration, cf2s(40));
        assert_eq!(act.stops[0].anim.fade_in, 0.1);
        assert_eq!(act.stops[0].anim.root_motion, true);
        assert_eq!(act.stops[0].anim.weapon_motion, false);
        assert_eq!(act.stops[0].anim.hit_motion, false);
        assert_eq!(act.stops[0].enter_from_table.len(), 3);
        assert_eq!(act.stops[0].enter_from_table[0].anim.as_str(), "Slime/WalkStart.*");
        assert_eq!(act.stops[0].enter_from_table[0].ratio.to_native(), 1.0);
        assert_eq!(act.stops[0].enter_from_table[1].anim.as_str(), "Slime/WalkLoop.*");
        assert_eq!(act.stops[0].enter_from_table[1].ratio.to_native(), 0.5);
        assert_eq!(act.stops[0].enter_from_table[2].anim.as_str(), "Slime/WalkLoop.*");
        assert_eq!(act.stops[0].enter_from_table[2].ratio.to_native(), 1.0);

        assert_eq!(act.turn_time, cf2s(12));
        assert_abs_diff_eq!(act.min_distance.to_native(), 1.8, epsilon = 1e-3);
        assert_abs_diff_eq!(act.step_length.to_native(), 1.0, epsilon = 1e-3);
    }
}
