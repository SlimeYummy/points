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
    pub tags: Vec<String>,
    pub enter_key: VirtualKey,
    pub enter_level: u16,
    pub derive_level: u16,
    pub special_derive_level: u16,
    pub anim_move: TmplAnimation,
    pub move_speed: f32,
    pub starts: Vec<TmplActionMoveStart>,
    pub start_time: f32,
    pub turns: Vec<TmplActionMoveTurn>,
    pub turn_time: f32,
    pub stops: Vec<TmplActionMoveStop>,
    pub stop_time: f32,
    pub quick_stop_time: f32,
    pub poise_level: u16,
    pub smooth_move_froms: Vec<TmplID>,
    pub smooth_move_duration: f32,
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
    pub enter_phase_table: Vec<TmplActionMoveStopEnter>,
    pub leave_phase_table: Vec<TmplActionMoveStopLeave>,
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
pub struct TmplActionMoveStopEnter {
    pub phase: [f32; 2],
    pub offset: f32,
}

impl TmplActionMoveStopEnter {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionMoveStopEnter) -> TmplActionMoveStopEnter {
        TmplActionMoveStopEnter {
            phase: [archived.phase[0].into(), archived.phase[1].into()],
            offset: archived.offset.into(),
        }
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
pub struct TmplActionMoveStopLeave {
    pub time: f32,
    pub phase: f32,
}

impl TmplActionMoveStopLeave {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplActionMoveStopLeave) -> TmplActionMoveStopLeave {
        TmplActionMoveStopLeave {
            time: archived.time.into(),
            phase: archived.phase.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{cf2s, id, LEVEL_MOVE};

    #[test]
    fn test_load_action_move() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionMove>(id!("Action.One.Run")).unwrap();
        assert_eq!(act.id, id!("Action.One.Run"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One^1"), id!("Style.One^2")]);
        assert_eq!(act.tags.as_slice(), &["Run"]);
        assert_eq!(act.enter_key, VirtualKey::Run);
        assert_eq!(act.enter_level, LEVEL_MOVE);
        assert_eq!(act.derive_level, LEVEL_MOVE - 10);
        assert_eq!(act.special_derive_level, LEVEL_MOVE + 10);

        assert_eq!(act.anim_move.files, "Girl_Run_Empty.*");
        assert_eq!(act.anim_move.duration, 0.93333334);
        assert_eq!(act.anim_move.fade_in, cf2s(4));
        assert_eq!(act.anim_move.root_motion, true);
        assert_eq!(act.move_speed, 3.0);

        assert_eq!(act.starts.len(), 3);
        assert_eq!(act.start_time, cf2s(4));
        assert_eq!(act.starts[0].anim.files, "Girl_RunStart_Empty.*");
        assert_eq!(act.starts[0].anim.fade_in, 0.0);
        assert_eq!(act.starts[0].anim.root_motion, true);
        assert_eq!(act.starts[0].anim.weapon_motion, false);
        assert_eq!(act.starts[0].enter_angle, [15f32.to_radians(), -15f32.to_radians()]);
        assert_eq!(act.starts[0].turn_in_place_end, cf2s(2));
        assert_eq!(act.starts[0].quick_stop_end, cf2s(20));
        assert_eq!(act.starts[1].anim.files, "Girl_RunStart_L180_Empty.*");
        assert_eq!(act.starts[1].enter_angle, [15f32.to_radians(), 180f32.to_radians()]);
        assert_eq!(act.starts[1].turn_in_place_end, cf2s(8));
        assert_eq!(act.starts[1].quick_stop_end, cf2s(26));
        assert_eq!(act.starts[2].anim.files, "Girl_RunStart_R180_Empty.*");
        assert_eq!(act.starts[2].enter_angle, [-15f32.to_radians(), -180f32.to_radians()]);
        assert_eq!(act.starts[2].turn_in_place_end, cf2s(8));
        assert_eq!(act.starts[2].quick_stop_end, cf2s(26));

        assert_eq!(act.turns.len(), 0);
        assert_eq!(act.turn_time, cf2s(10));

        assert_eq!(act.stops.len(), 2);
        assert_eq!(act.stop_time, cf2s(6));
        assert_eq!(act.quick_stop_time, cf2s(0));
        assert_eq!(act.stops[0].anim.files, "Girl_RunStop_L_Empty.*");
        assert_eq!(act.stops[0].anim.fade_in, cf2s(4));
        assert_eq!(act.stops[0].anim.root_motion, true);
        assert_eq!(act.stops[0].anim.weapon_motion, false);
        assert_eq!(act.stops[0].enter_phase_table.len(), 1);
        assert_eq!(
            TmplActionMoveStopEnter::from_rkyv(&act.stops[0].enter_phase_table[0]),
            TmplActionMoveStopEnter {
                phase: [0.75, 0.25],
                offset: cf2s(2)
            }
        );
        assert_eq!(act.stops[0].leave_phase_table.len(), 2);
        assert_eq!(
            TmplActionMoveStopLeave::from_rkyv(&act.stops[0].leave_phase_table[0]),
            TmplActionMoveStopLeave { time: 0.0, phase: 0.0 }
        );
        assert_eq!(
            TmplActionMoveStopLeave::from_rkyv(&act.stops[0].leave_phase_table[1]),
            TmplActionMoveStopLeave {
                time: cf2s(14),
                phase: 0.5
            }
        );

        assert_eq!(act.stops[1].anim.files, "Girl_RunStop_R_Empty.*");
        assert_eq!(act.stops[0].enter_phase_table.len(), 1);
        assert_eq!(
            TmplActionMoveStopEnter::from_rkyv(&act.stops[1].enter_phase_table[0]),
            TmplActionMoveStopEnter {
                phase: [0.25, 0.75],
                offset: cf2s(2)
            }
        );
        assert_eq!(act.stops[1].leave_phase_table.len(), 2);
        assert_eq!(
            TmplActionMoveStopLeave::from_rkyv(&act.stops[1].leave_phase_table[0]),
            TmplActionMoveStopLeave { time: 0.0, phase: 0.5 }
        );
        assert_eq!(
            TmplActionMoveStopLeave::from_rkyv(&act.stops[1].leave_phase_table[1]),
            TmplActionMoveStopLeave {
                time: cf2s(14),
                phase: 0.0
            }
        );

        assert_eq!(act.poise_level, 0);
        assert_eq!(act.smooth_move_froms.as_slice(), &[id!("Action.One.Run")]);
        assert_eq!(act.smooth_move_duration, cf2s(10));
    }
}
