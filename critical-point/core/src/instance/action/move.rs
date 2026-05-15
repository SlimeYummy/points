use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule,
};
use crate::template::{At, TmplActionMove, TmplActionMoveStopEnter, TmplActionMoveStopLeave};
use crate::utils::{ActionType, SmallVec, TmplID, VirtualKeyDir, extend, lerp, loose_ge, loose_le, sb};

pub type InstActionMoveStopEnter = TmplActionMoveStopEnter;
pub type InstActionMoveStopLeave = TmplActionMoveStopLeave;

#[derive(Debug)]
pub struct InstActionMoveStart {
    pub anim: InstAnimation,
    pub enter_angle: [f32; 2],
    pub turn_in_place_end: f32,
    pub quick_stop_end: f32,
}

#[derive(Debug)]
pub struct InstActionMoveTurn {
    pub anim: InstAnimation,
    pub enter_angle: [f32; 2],
    pub turn_in_place_end: f32,
}

#[derive(Debug)]
pub struct InstActionMoveStop {
    pub anim: InstAnimation,
    pub enter_phase_table: SmallVec<[InstActionMoveStopEnter; 3]>,
    pub leave_phase_table: SmallVec<[InstActionMoveStopLeave; 4]>,
}

#[derive(Debug)]
#[repr(C)]
pub struct InstActionMove {
    pub _base: InstActionBase,
    pub derive_level: u16,
    pub derive_level_special: u16,
    pub poise_level: u16,
    pub anim_move: InstAnimation,
    pub move_speed: f32,
    pub speed_ratio: f32,
    pub starts: Vec<InstActionMoveStart>,
    pub stops: Vec<InstActionMoveStop>,
    pub quick_stop_time: f32,
    pub turns: Vec<InstActionMoveTurn>,
    pub turn_time: f32,
    pub direct_turn_cos: [f32; 2],
    pub smooth_move_froms: Vec<TmplID>,
    pub smooth_move_duration: f32,
}

extend!(InstActionMove, InstActionBase);

unsafe impl InstActionAny for InstActionMove {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::Move
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<InstDeriveRule>) {}
}

impl InstActionMove {
    pub(crate) fn new_from_action(ctx: &ContextActionAssemble<'_>, tmpl: At<TmplActionMove>) -> Option<InstActionMove> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        let mut starts = Vec::with_capacity(tmpl.starts.len());
        for a in tmpl.starts.iter() {
            let mut enter_angle = [a.enter_angle[0].to_native(), a.enter_angle[1].to_native()];
            if enter_angle[0] > enter_angle[1] {
                enter_angle.swap(0, 1);
            }

            starts.push(InstActionMoveStart {
                anim: InstAnimation::from_rkyv(&a.anim),
                enter_angle,
                turn_in_place_end: a.turn_in_place_end.to_native(),
                quick_stop_end: a.quick_stop_end.to_native(),
            });
        }

        let mut direct_turn_angle: [f32; 2] = [-std::f32::consts::PI, std::f32::consts::PI];
        let mut turns = Vec::with_capacity(tmpl.turns.len());
        for a in tmpl.turns.iter() {
            let mut enter_angle = [a.enter_angle[0].to_native(), a.enter_angle[1].to_native()];
            if enter_angle[0] > enter_angle[1] {
                enter_angle.swap(0, 1);
            }

            if enter_angle[0] < 0.0 && enter_angle[1] < 0.0 {
                direct_turn_angle[0] = direct_turn_angle[0].max(enter_angle[0]);
            }
            else if enter_angle[0] > 0.0 && enter_angle[1] > 0.0 {
                direct_turn_angle[1] = direct_turn_angle[1].min(enter_angle[1]);
            }
            else {
                direct_turn_angle = [0.0; 2];
            }

            turns.push(InstActionMoveTurn {
                anim: InstAnimation::from_rkyv(&a.anim),
                enter_angle,
                turn_in_place_end: a.turn_in_place_end.to_native(),
            });
        }

        let mut stops = Vec::with_capacity(tmpl.stops.len());
        for a in tmpl.stops.iter() {
            stops.push(InstActionMoveStop {
                anim: InstAnimation::from_rkyv(&a.anim),
                enter_phase_table: a
                    .enter_phase_table
                    .iter()
                    .map(InstActionMoveStopEnter::from_rkyv)
                    .collect(),
                leave_phase_table: a
                    .leave_phase_table
                    .iter()
                    .map(InstActionMoveStopLeave::from_rkyv)
                    .collect(),
            });
        }

        Some(InstActionMove {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: Some(VirtualKeyDir::new(tmpl.enter_key, None)),
                enter_level: tmpl.enter_level.to_native(),
                ..Default::default()
            },
            derive_level: tmpl.derive_level.to_native(),
            derive_level_special: tmpl.derive_level_special.to_native(),
            poise_level: tmpl.poise_level.to_native(),
            anim_move: InstAnimation::from_rkyv(&tmpl.anim_move),
            move_speed: tmpl.move_speed.to_native(),
            speed_ratio: tmpl.speed_ratio.to_native(),
            starts,
            stops,
            quick_stop_time: tmpl.quick_stop_time.to_native(),
            turns,
            turn_time: tmpl.turn_time.to_native(),
            direct_turn_cos: [direct_turn_angle[0].cos(), direct_turn_angle[1].cos()],
            smooth_move_froms: tmpl.smooth_move_froms.iter().map(|x| *x).collect(),
            smooth_move_duration: tmpl.smooth_move_duration.to_native(),
        })
    }

    #[inline]
    pub fn animations(&self) -> impl Iterator<Item = &InstAnimation> {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                yield &self.anim_move;
                for start in &self.starts {
                    yield &start.anim;
                }
                for turn in &self.turns {
                    yield &turn.anim;
                }
                for stop in &self.stops {
                    yield &stop.anim;
                }
            },
        )
    }

    #[inline]
    pub fn animations_count(&self) -> usize {
        1 + self.starts.len() + self.turns.len() + self.stops.len()
    }

    pub fn find_start_by_angle(&self, angle: f32) -> Option<(usize, &InstActionMoveStart)> {
        for (idx, start) in self.starts.iter().enumerate() {
            if loose_ge!(angle, start.enter_angle[0]) && loose_le!(angle, start.enter_angle[1]) {
                return Some((idx, start));
            }
        }
        None
    }

    #[inline]
    pub fn check_direct_turn_by_cos(&self, cos: f32, sign: f32) -> bool {
        match sign < 0.0 {
            true => cos >= self.direct_turn_cos[0],
            false => cos <= self.direct_turn_cos[1],
        }
    }

    pub fn find_turn_by_angle(&self, angle: f32) -> Option<(usize, &InstActionMoveTurn)> {
        for (idx, turn) in self.turns.iter().enumerate() {
            if loose_ge!(angle, turn.enter_angle[0]) && loose_le!(angle, turn.enter_angle[1]) {
                return Some((idx, turn));
            }
        }
        None
    }

    pub fn find_stop_by_phase(&self, phase: f32) -> Option<(usize, &InstActionMoveStop, f32)> {
        let pahse = phase.rem_euclid(1.0);
        for (idx, stop) in self.stops.iter().enumerate() {
            for item in &stop.enter_phase_table {
                if item.phase[0] <= item.phase[1] {
                    if loose_ge!(pahse, item.phase[0]) && loose_le!(pahse, item.phase[1]) {
                        return Some((idx, stop, item.offset));
                    }
                }
                else {
                    if loose_ge!(pahse, item.phase[0]) || loose_le!(pahse, item.phase[1]) {
                        return Some((idx, stop, item.offset));
                    }
                }
            }
        }
        None
    }

    pub fn calc_stop_phase(&self, stop_anim_idx: usize, time: f32) -> Option<f32> {
        let stop = self.stops.get(stop_anim_idx)?;
        let time = time % stop.anim.duration;

        let idx = stop.leave_phase_table.iter().position(|x| x.time > time)?;
        debug_assert!(idx > 0);
        let a = stop.leave_phase_table[idx - 1];
        let b = stop.leave_phase_table[idx];

        let phase = if a.phase < b.phase {
            lerp(a.phase, b.phase, (time - a.time) / (b.time - a.time))
        }
        else {
            lerp(a.phase, b.phase + 1.0, (time - a.time) / (b.time - a.time)) % 1.0
        };
        Some(phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{DtHashMap, LEVEL_MOVE, VirtualKey, cf2s, id};

    #[test]
    fn test_new_move() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = DtHashMap::default();

        let tmpl_act = db.find_as::<TmplActionMove>(id!("Action.Instance.Run^1A")).unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionMove::new_from_action(&ctx, tmpl_act).unwrap();
        assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Run^1A"));
        assert_eq!(inst_act.tags, vec![sb!("Run")]);
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Run, None));
        assert_eq!(inst_act.enter_level, LEVEL_MOVE);
        assert_eq!(inst_act.derive_level, LEVEL_MOVE - 10);
        assert_eq!(inst_act.derive_level_special, LEVEL_MOVE + 10);
        assert_eq!(inst_act.poise_level, 0);

        assert_eq!(inst_act.anim_move.files, "Girl/Run_Empty.*");
        assert_eq!(inst_act.anim_move.duration, 0.93333334);
        assert_eq!(inst_act.anim_move.fade_in, cf2s(4));
        assert_eq!(inst_act.anim_move.root_motion, true);
        assert_eq!(inst_act.move_speed, 3.0);
        assert_eq!(inst_act.speed_ratio, 1.0);

        assert_eq!(inst_act.starts.len(), 3);
        assert_eq!(inst_act.starts[0].anim.files, "Girl/RunStart_Empty.*");
        assert_eq!(inst_act.starts[0].anim.fade_in, 0.0);
        assert_eq!(inst_act.starts[0].anim.root_motion, true);
        assert_eq!(inst_act.starts[0].enter_angle, [
            -15f32.to_radians(),
            15f32.to_radians()
        ]);
        assert_eq!(inst_act.starts[0].turn_in_place_end, cf2s(2));
        assert_eq!(inst_act.starts[0].quick_stop_end, cf2s(20));
        assert_eq!(inst_act.starts[1].anim.files, "Girl/RunStart_L180_Empty.*");
        assert_eq!(inst_act.starts[1].enter_angle, [
            15f32.to_radians(),
            180f32.to_radians()
        ]);
        assert_eq!(inst_act.starts[1].turn_in_place_end, cf2s(8));
        assert_eq!(inst_act.starts[1].quick_stop_end, cf2s(26));
        assert_eq!(inst_act.starts[2].anim.files, "Girl/RunStart_R180_Empty.*");
        assert_eq!(inst_act.starts[2].enter_angle, [
            -180f32.to_radians(),
            -15f32.to_radians()
        ]);
        assert_eq!(inst_act.starts[2].turn_in_place_end, cf2s(8));
        assert_eq!(inst_act.starts[2].quick_stop_end, cf2s(26));

        assert_eq!(inst_act.stops.len(), 2);
        assert_eq!(inst_act.quick_stop_time, cf2s(0));
        assert_eq!(inst_act.stops[0].anim.files, "Girl/RunStop_L_Empty.*");
        assert_eq!(inst_act.stops[0].anim.fade_in, cf2s(4));
        assert_eq!(inst_act.stops[0].anim.root_motion, true);
        assert_eq!(inst_act.stops[0].enter_phase_table.as_slice(), &[
            InstActionMoveStopEnter {
                phase: [0.75, 0.25],
                offset: cf2s(2)
            }
        ]);
        assert_eq!(inst_act.stops[0].leave_phase_table.as_slice(), &[
            InstActionMoveStopLeave { time: 0.0, phase: 0.0 },
            InstActionMoveStopLeave {
                time: cf2s(14),
                phase: 0.5
            }
        ]);
        assert_eq!(inst_act.stops[1].anim.files, "Girl/RunStop_R_Empty.*");
        assert_eq!(inst_act.stops[1].enter_phase_table.as_slice(), &[
            InstActionMoveStopEnter {
                phase: [0.25, 0.75],
                offset: cf2s(2)
            }
        ]);
        assert_eq!(inst_act.stops[1].leave_phase_table.as_slice(), &[
            InstActionMoveStopLeave { time: 0.0, phase: 0.5 },
            InstActionMoveStopLeave {
                time: cf2s(14),
                phase: 0.0
            }
        ]);

        assert_eq!(inst_act.turns.len(), 0);
        assert_eq!(inst_act.turn_time, cf2s(10));
        assert_eq!(inst_act.direct_turn_cos, [-1.0; 2]);

        assert_eq!(inst_act.smooth_move_froms.as_slice(), &[]);
        assert_eq!(inst_act.smooth_move_duration, cf2s(10));
    }
}
