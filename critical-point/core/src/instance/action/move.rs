use crate::instance::action::base::{ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation};
use crate::template::{At, TmplActionMove, TmplType};
use crate::utils::{extend, TmplID, VirtualKey, VirtualKeyDir};
use crate::{loose_ge, loose_le};

#[derive(Debug)]
#[repr(C)]
pub struct InstActionMove {
    pub _base: InstActionBase,
    pub anim_move: InstAnimation,
    pub move_speed: f32,
    pub starts: Vec<InstActionMoveStart>,
    pub start_time: f32,
    pub turns: Vec<InstActionMoveTurn>,
    pub turn_time: f32,
    pub direct_turn_cos: [f32; 2],
    pub stops: Vec<InstActionMoveStop>,
    pub stop_time: f32,
    pub quick_stop_time: f32,
    pub derive_level: u16,
    pub poise_level: u16,
}

extend!(InstActionMove, InstActionBase);

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
    pub enter_phase_table: Vec<[f32; 3]>,
    pub speed_down_end: f32,
}

unsafe impl InstActionAny for InstActionMove {
    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::ActionMove
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<(VirtualKey, TmplID)>) {}
}

impl InstActionMove {
    pub(crate) fn try_assemble(ctx: &ContextActionAssemble<'_>, tmpl: At<TmplActionMove>) -> Option<InstActionMove> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        let mut starts = Vec::with_capacity(tmpl.starts.len());
        for t in tmpl.starts.iter() {
            let mut enter_angle = [t.enter_angle[0].into(), t.enter_angle[1].into()];
            if enter_angle[0] > enter_angle[1] {
                enter_angle.swap(0, 1);
            }

            starts.push(InstActionMoveStart {
                anim: InstAnimation::from_rkyv(&t.anim),
                enter_angle,
                turn_in_place_end: t.turn_in_place_end.into(),
                quick_stop_end: t.quick_stop_end.into(),
            });
        }

        let mut direct_turn_angle: [f32; 2] = [-std::f32::consts::PI, std::f32::consts::PI];
        let mut turns = Vec::with_capacity(tmpl.turns.len());
        for t in tmpl.turns.iter() {
            let mut enter_angle = [t.enter_angle[0].into(), t.enter_angle[1].into()];
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
                anim: InstAnimation::from_rkyv(&t.anim),
                enter_angle,
                turn_in_place_end: t.turn_in_place_end.into(),
            });
        }

        let mut stops = Vec::with_capacity(tmpl.stops.len());
        for t in tmpl.stops.iter() {
            stops.push(InstActionMoveStop {
                anim: InstAnimation::from_rkyv(&t.anim),
                enter_phase_table: t
                    .enter_phase_table
                    .iter()
                    .map(|x| [x[0].into(), x[1].into(), x[2].into()])
                    .collect(),
                speed_down_end: t.speed_down_end.into(),
            });
        }

        Some(InstActionMove {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                enter_key: Some(VirtualKeyDir::new(tmpl.enter_key, None)),
                enter_level: tmpl.enter_level.into(),
                ..Default::default()
            },
            anim_move: InstAnimation::from_rkyv(&tmpl.anim_move),
            move_speed: tmpl.move_speed.into(),
            starts,
            start_time: tmpl.start_time.into(),
            turns,
            turn_time: tmpl.turn_time.into(),
            direct_turn_cos: [direct_turn_angle[0].cos(), direct_turn_angle[1].cos()],
            stops,
            stop_time: tmpl.stop_time.into(),
            quick_stop_time: tmpl.quick_stop_time.into(),
            derive_level: tmpl.derive_level.into(),
            poise_level: tmpl.poise_level.into(),
        })
    }

    pub fn find_start_by_angle(&self, angle: f32) -> Option<(usize, &InstActionMoveStart)> {
        println!("{:?}", self.starts.iter().map(|x| x.enter_angle).collect::<Vec<_>>());
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
            for enter_phase in &stop.enter_phase_table {
                if enter_phase[0] <= enter_phase[1] {
                    if loose_ge!(pahse, enter_phase[0]) && loose_le!(pahse, enter_phase[1]) {
                        return Some((idx, stop, enter_phase[2]));
                    }
                }
                else {
                    if loose_le!(pahse, enter_phase[0]) || loose_ge!(pahse, enter_phase[1]) {
                        return Some((idx, stop, enter_phase[2]));
                    }
                }
            }
        }
        None
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
    pub fn animations_size(&self) -> usize {
        1 + self.starts.len() + self.turns.len() + self.stops.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::CFG_FPS;
    use crate::template::{TmplDatabase, TmplHashMap};
    use crate::utils::{id, LEVEL_MOVE};
    use ahash::HashMapExt;

    #[test]
    fn test_assemble() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = TmplHashMap::new();

        let tmpl_act = db.find_as::<TmplActionMove>(id!("Action.Instance.Jog/1A")).unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionMove::try_assemble(&ctx, tmpl_act).unwrap();
        assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Jog/1A"));
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Run, None));
        assert_eq!(inst_act.enter_level, LEVEL_MOVE);

        assert_eq!(inst_act.anim_move.files, "girl_jog.*");
        assert_eq!(inst_act.anim_move.duration, 0.93333334);
        assert_eq!(inst_act.anim_move.fade_in, 4.0 / CFG_FPS);
        assert_eq!(inst_act.anim_move.root_motion, true);
        assert_eq!(inst_act.move_speed, 3.0);

        assert_eq!(inst_act.starts.len(), 3);
        assert_eq!(inst_act.start_time, 4.0 / CFG_FPS);
        assert_eq!(inst_act.starts[0].anim.files, "girl_jog_start.*");
        assert_eq!(inst_act.starts[0].anim.fade_in, 0.0);
        assert_eq!(inst_act.starts[0].anim.root_motion, true);
        assert_eq!(inst_act.starts[0].enter_angle, [
            -15f32.to_radians(),
            15f32.to_radians()
        ]);
        assert_eq!(inst_act.starts[0].turn_in_place_end, 2.0 / CFG_FPS);
        assert_eq!(inst_act.starts[0].quick_stop_end, 20.0 / CFG_FPS);
        assert_eq!(inst_act.starts[1].anim.files, "girl_jog_start_turn_l180.*");
        assert_eq!(inst_act.starts[1].enter_angle, [
            15f32.to_radians(),
            180f32.to_radians()
        ]);
        assert_eq!(inst_act.starts[1].turn_in_place_end, 8.0 / CFG_FPS);
        assert_eq!(inst_act.starts[1].quick_stop_end, 26.0 / CFG_FPS);
        assert_eq!(inst_act.starts[2].anim.files, "girl_jog_start_turn_r180.*");
        assert_eq!(inst_act.starts[2].enter_angle, [
            -180f32.to_radians(),
            -15f32.to_radians()
        ]);
        assert_eq!(inst_act.starts[2].turn_in_place_end, 8.0 / CFG_FPS);
        assert_eq!(inst_act.starts[2].quick_stop_end, 26.0 / CFG_FPS);

        assert_eq!(inst_act.turns.len(), 0);
        assert_eq!(inst_act.turn_time, 10.0 / CFG_FPS);
        assert_eq!(inst_act.direct_turn_cos, [-1.0; 2]);

        assert_eq!(inst_act.stops.len(), 2);
        assert_eq!(inst_act.stop_time, 4.0 / CFG_FPS);
        assert_eq!(inst_act.quick_stop_time, 0.0 / CFG_FPS);
        assert_eq!(inst_act.stops[0].anim.files, "girl_jog_stop_l.*");
        assert_eq!(inst_act.stops[0].anim.fade_in, 4.0 / CFG_FPS);
        assert_eq!(inst_act.stops[0].anim.root_motion, true);
        assert_eq!(inst_act.stops[0].enter_phase_table, vec![[0.75, 0.25, 2.0 / CFG_FPS]]);
        assert_eq!(inst_act.stops[0].speed_down_end, 12.0 / CFG_FPS);
        assert_eq!(inst_act.stops[1].anim.files, "girl_jog_stop_r.*");
        assert_eq!(inst_act.stops[1].enter_phase_table, vec![[0.25, 0.75, 2.0 / CFG_FPS]]);
        assert_eq!(inst_act.stops[1].speed_down_end, 12.0 / CFG_FPS);

        assert_eq!(inst_act.enter_level, LEVEL_MOVE);
        assert_eq!(inst_act.derive_level, LEVEL_MOVE);
        assert_eq!(inst_act.poise_level, 0);
    }
}
