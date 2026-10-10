use crate::template::action::base::TmplAnimation;
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{TmplID, VirtualKey};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveFree {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character: TmplID,
    pub styles: Vec<TmplID>,
    pub tags: Vec<String>,
    pub enter_key: VirtualKey,
    pub enter_level: u16,
    pub keep_level: u16,
    pub keep_level_special: u16,
    pub poise_level: u16,
    pub anim_move: TmplAnimation,
    pub move_speed: f32,
    pub speed_ratio: f32,
    pub starts: Vec<TmplActionMoveFreeStart>,
    pub stops: Vec<TmplActionMoveFreeStop>,
    pub quick_stop_time: f32,
    pub turns: Vec<TmplActionMoveFreeTurn>,
    pub turn_time: f32,
    pub smooth_move_froms: Vec<TmplID>,
    pub smooth_move_duration: f32,
}

impl_tmpl!(TmplActionMoveFree, ActionMoveFree, "ActionMoveFree");

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveFreeStart {
    pub anim: TmplAnimation,
    pub enter_angle: [f32; 2],
    pub turn_in_place_end: f32,
    pub quick_stop_end: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveFreeTurn {
    pub anim: TmplAnimation,
    pub enter_angle: [f32; 2],
    pub turn_in_place_end: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
#[serde(tag = "T")]
pub enum TmplActionMoveFreeStop {
    Stop1(TmplActionMoveFreeStop1),
    Stop2(TmplActionMoveFreeStop2),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveFreeStop1 {
    pub anim: TmplAnimation,
    #[serde(default)]
    pub fade_in: f32,
    pub enter_phase_table: Vec<[f32; 2]>,
    pub leave_phase_table: Vec<TmplActionMoveFreeStopLeave>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMoveFreeStop2 {
    pub prev_anim_time: [f32; 2],
    pub anim_stop: TmplAnimation,
    pub no_arm_fade_in: f32,
    pub arm_fade_out: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anim_arm_additive: Option<TmplAnimation>,
    #[serde(default)]
    pub arm_additive_fade_inout: [f32; 2],
    pub enter_phase_table: Vec<[f32; 2]>,
    pub leave_phase_table: Vec<TmplActionMoveFreeStopLeave>,
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
pub struct TmplActionMoveFreeStopLeave {
    pub time: f32,
    pub phase: f32,
}

impl TmplActionMoveFreeStopLeave {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionMoveFreeStopLeave) -> TmplActionMoveFreeStopLeave {
        TmplActionMoveFreeStopLeave {
            time: archived.time.into(),
            phase: archived.phase.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{RootMotionMeta, load_root_motion_meta};
    use crate::consts::TEST_ASSET_PATH;
    use crate::template::database::TmplDatabase;
    use crate::utils::{LEVEL_MOVE, cf2s, id};

    #[test]
    fn test_load_action_move() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionMoveFree>(id!("Action.One.Run")).unwrap();
        assert_eq!(act.id, id!("Action.One.Run"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One^1"), id!("Style.One^2")]);
        assert_eq!(act.tags.as_slice(), &["Run"]);
        assert_eq!(act.enter_key, VirtualKey::Run);
        assert_eq!(act.enter_level, LEVEL_MOVE);
        assert_eq!(act.keep_level, LEVEL_MOVE - 10);
        assert_eq!(act.keep_level_special, LEVEL_MOVE + 10);
        assert_eq!(act.poise_level, 0);

        assert_eq!(act.anim_move.files, "Girl/Run_Empty.*");
        assert_eq!(act.anim_move.duration, 0.93333334);
        assert_eq!(act.anim_move.fade_in, cf2s(4));
        assert_eq!(act.anim_move.root_motion, true);
        assert_eq!(act.anim_move.weapon_control, false);
        assert_eq!(act.anim_move.hit_motion, false);
        assert_eq!(act.anim_move.shape_key, false);
        assert_eq!(act.anim_move.additive_blending, false);
        assert_eq!(act.move_speed, 3.0);
        let meta = load_root_motion_meta(&format!("{}/Girl/Run_Empty.rm-ozz", TEST_ASSET_PATH)).unwrap();
        assert_eq!(
            act.speed_ratio,
            act.move_speed / (meta.position_default.whole_distance_xz / act.anim_move.duration)
        );

        assert_eq!(act.starts.len(), 3);
        assert_eq!(act.starts[0].anim.files, "Girl/Run_Start_Empty.*");
        assert_eq!(act.starts[0].anim.fade_in, 0.0);
        assert_eq!(act.starts[0].anim.root_motion, true);
        assert_eq!(act.starts[0].anim.weapon_control, false);
        assert_eq!(act.starts[0].anim.hit_motion, false);
        assert_eq!(act.starts[0].anim.shape_key, false);
        assert_eq!(act.starts[0].anim.additive_blending, false);
        assert_eq!(act.starts[0].enter_angle, [15f32.to_radians(), -15f32.to_radians()]);
        assert_eq!(act.starts[0].turn_in_place_end, cf2s(2));
        assert_eq!(act.starts[0].quick_stop_end, cf2s(20));
        assert_eq!(act.starts[1].anim.files, "Girl/Run_Start_L180_Empty.*");
        assert_eq!(act.starts[1].enter_angle, [15f32.to_radians(), 180f32.to_radians()]);
        assert_eq!(act.starts[1].turn_in_place_end, cf2s(8));
        assert_eq!(act.starts[1].quick_stop_end, cf2s(26));
        assert_eq!(act.starts[2].anim.files, "Girl/Run_Start_R180_Empty.*");
        assert_eq!(act.starts[2].enter_angle, [-15f32.to_radians(), -180f32.to_radians()]);
        assert_eq!(act.starts[2].turn_in_place_end, cf2s(8));
        assert_eq!(act.starts[2].quick_stop_end, cf2s(26));

        assert_eq!(act.stops.len(), 2);
        assert_eq!(act.quick_stop_time, cf2s(0));

        let stop0 = match &act.stops[0] {
            ArchivedTmplActionMoveFreeStop::Stop1(s) => s,
            _ => panic!("Expected Stop1"),
        };
        assert_eq!(stop0.anim.files, "Girl/Run_Stop_L1_Empty.*");
        assert_eq!(stop0.anim.fade_in, cf2s(4));
        assert_eq!(stop0.anim.root_motion, true);
        assert_eq!(stop0.anim.weapon_control, false);
        assert_eq!(stop0.anim.hit_motion, false);
        assert_eq!(stop0.anim.shape_key, false);
        assert_eq!(stop0.anim.additive_blending, false);
        assert_eq!(stop0.fade_in, cf2s(4));
        assert_eq!(stop0.enter_phase_table.len(), 1);
        assert_eq!(stop0.enter_phase_table[0][0], 0.75);
        assert_eq!(stop0.enter_phase_table[0][1], 0.25);
        assert_eq!(stop0.leave_phase_table.len(), 2);
        assert_eq!(
            TmplActionMoveFreeStopLeave::from_rkyv(&stop0.leave_phase_table[0]),
            TmplActionMoveFreeStopLeave { time: 0.0, phase: 0.0 }
        );
        assert_eq!(
            TmplActionMoveFreeStopLeave::from_rkyv(&stop0.leave_phase_table[1]),
            TmplActionMoveFreeStopLeave {
                time: cf2s(14),
                phase: 0.5
            }
        );

        let stop1 = match &act.stops[1] {
            ArchivedTmplActionMoveFreeStop::Stop2(s) => s,
            _ => panic!("Expected Stop2"),
        };
        assert_eq!(stop1.prev_anim_time, [cf2s(6), cf2s(12)]);
        assert_eq!(stop1.anim_stop.files, "Girl/Run_Stop_R1_Empty.*");
        assert_eq!(stop1.anim_stop.fade_in, cf2s(4));
        assert_eq!(stop1.anim_stop.root_motion, true);
        assert_eq!(stop1.anim_stop.weapon_control, false);
        assert_eq!(stop1.anim_stop.hit_motion, false);
        assert_eq!(stop1.anim_stop.shape_key, false);
        assert_eq!(stop1.anim_stop.additive_blending, false);
        assert_eq!(stop1.no_arm_fade_in, cf2s(6));
        assert_eq!(stop1.arm_fade_out, cf2s(14));
        assert_eq!(
            stop1.anim_arm_additive.as_ref().unwrap().files,
            "Girl/Run_Stop_Add_Empty.*"
        );
        assert_eq!(stop1.anim_arm_additive.as_ref().unwrap().additive_blending, true);
        assert_eq!(stop1.arm_additive_fade_inout, [cf2s(12), cf2s(20)]);
        assert_eq!(stop1.enter_phase_table.len(), 1);
        assert_eq!(stop1.enter_phase_table[0][0], 0.25);
        assert_eq!(stop1.enter_phase_table[0][1], 0.75);
        assert_eq!(stop1.leave_phase_table.len(), 3);
        assert_eq!(
            TmplActionMoveFreeStopLeave::from_rkyv(&stop1.leave_phase_table[0]),
            TmplActionMoveFreeStopLeave { time: 0.0, phase: 0.0 }
        );
        assert_eq!(
            TmplActionMoveFreeStopLeave::from_rkyv(&stop1.leave_phase_table[1]),
            TmplActionMoveFreeStopLeave {
                time: cf2s(14),
                phase: 0.5
            }
        );
        assert_eq!(
            TmplActionMoveFreeStopLeave::from_rkyv(&stop1.leave_phase_table[2]),
            TmplActionMoveFreeStopLeave {
                time: cf2s(34),
                phase: 0.8
            }
        );

        assert_eq!(act.turns.len(), 0);
        assert_eq!(act.turn_time, cf2s(10));

        assert_eq!(act.smooth_move_froms.as_slice(), &[id!("Action.One.Run")]);
        assert_eq!(act.smooth_move_duration, cf2s(10));
    }
}
