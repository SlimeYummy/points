use crate::template::action::base::TmplAnimation;
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{TmplID, VirtualKey};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMove {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character: TmplID,
    pub styles: Vec<TmplID>,
    pub anim_move: TmplAnimation,
    pub move_speed: f32,
    pub starts: Vec<TmplActionMoveStart>,
    pub start_time: f32,
    pub turns: Vec<TmplActionMoveTurn>,
    pub turn_time: f32,
    pub stops: Vec<TmplActionMoveStop>,
    pub stop_time: f32,
    pub quick_stop_time: f32,
    pub enter_key: VirtualKey,
    pub enter_level: u16,
    pub derive_level: u16,
    pub poise_level: u16,
}

impl_tmpl!(TmplActionMove, ActionMove, "ActionMove");

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveStart {
    #[serde(flatten)]
    pub anim: TmplAnimation,
    pub enter_angle: [f32; 2],
    pub turn_in_place_end: f32,
    pub quick_stop_end: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveTurn {
    #[serde(flatten)]
    pub anim: TmplAnimation,
    pub enter_angle: [f32; 2],
    pub turn_in_place_end: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveStop {
    #[serde(flatten)]
    pub anim: TmplAnimation,
    pub enter_phase_table: Vec<[f32; 3]>,
    pub speed_down_end: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::CFG_FPS;
    use crate::template::database::TmplDatabase;
    use crate::utils::{id, LEVEL_MOVE};

    #[test]
    fn test_load_action_move() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionMove>(id!("Action.One.Jog")).unwrap();
        assert_eq!(act.id, id!("Action.One.Jog"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One/1"), id!("Style.One/2")]);

        assert_eq!(act.anim_move.files, "girl_jog.*");
        assert_eq!(act.anim_move.duration, 0.93333334);
        assert_eq!(act.anim_move.fade_in, 4.0 / CFG_FPS);
        assert_eq!(act.anim_move.root_motion, true);
        assert_eq!(act.move_speed, 3.0);

        assert_eq!(act.starts.len(), 3);
        assert_eq!(act.start_time, 4.0 / CFG_FPS);
        assert_eq!(act.starts[0].anim.files, "girl_jog_start.*");
        assert_eq!(act.starts[0].anim.fade_in, 0.0);
        assert_eq!(act.starts[0].anim.root_motion, true);
        assert_eq!(act.starts[0].enter_angle, [15f32.to_radians(), -15f32.to_radians()]);
        assert_eq!(act.starts[0].turn_in_place_end, 2.0 / CFG_FPS);
        assert_eq!(act.starts[0].quick_stop_end, 20.0 / CFG_FPS);
        assert_eq!(act.starts[1].anim.files, "girl_jog_start_turn_l180.*");
        assert_eq!(act.starts[1].enter_angle, [15f32.to_radians(), 180f32.to_radians()]);
        assert_eq!(act.starts[1].turn_in_place_end, 8.0 / CFG_FPS);
        assert_eq!(act.starts[1].quick_stop_end, 26.0 / CFG_FPS);
        assert_eq!(act.starts[2].anim.files, "girl_jog_start_turn_r180.*");
        assert_eq!(act.starts[2].enter_angle, [-15f32.to_radians(), -180f32.to_radians()]);
        assert_eq!(act.starts[2].turn_in_place_end, 8.0 / CFG_FPS);
        assert_eq!(act.starts[2].quick_stop_end, 26.0 / CFG_FPS);

        assert_eq!(act.turns.len(), 0);
        assert_eq!(act.turn_time, 10.0 / CFG_FPS);

        assert_eq!(act.stops.len(), 2);
        assert_eq!(act.stop_time, 4.0 / CFG_FPS);
        assert_eq!(act.quick_stop_time, 0.0 / CFG_FPS);
        assert_eq!(act.stops[0].anim.files, "girl_jog_stop_l.*");
        assert_eq!(act.stops[0].anim.fade_in, 4.0 / CFG_FPS);
        assert_eq!(act.stops[0].anim.root_motion, true);
        assert_eq!(act.stops[0].enter_phase_table, vec![[0.75, 0.25, 2.0 / CFG_FPS]]);
        assert_eq!(act.stops[0].speed_down_end, 12.0 / CFG_FPS);
        assert_eq!(act.stops[1].anim.files, "girl_jog_stop_r.*");
        assert_eq!(act.stops[1].enter_phase_table, vec![[0.25, 0.75, 2.0 / CFG_FPS]]);
        assert_eq!(act.stops[1].speed_down_end, 12.0 / CFG_FPS);

        assert_eq!(act.enter_level, LEVEL_MOVE);
        assert_eq!(act.derive_level, LEVEL_MOVE);
        assert_eq!(act.poise_level, 0);
    }
}
