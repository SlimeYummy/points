use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule,
};
use crate::template::{ArchivedTmplActionMoveNpcStop, ArchivedTmplActionMoveNpcStopFrom, At, TmplActionMoveNpc};
use crate::utils::{ActionType, SmallVec, Symbol, VirtualKeyDir, extend, sb};

#[derive(Debug, Clone, PartialEq)]
pub struct InstActionMoveNpcStopFrom {
    pub anim: Symbol,
    pub ratio: f32,
}

impl InstActionMoveNpcStopFrom {
    #[inline]
    fn from_rkyv(archived: &ArchivedTmplActionMoveNpcStopFrom) -> InstActionMoveNpcStopFrom {
        InstActionMoveNpcStopFrom {
            anim: sb!(archived.anim.as_str()),
            ratio: archived.ratio.to_native(),
        }
    }
}

#[derive(Debug)]
pub struct InstActionMoveNpcStop {
    pub anim: InstAnimation,
    pub enter_from_table: SmallVec<[InstActionMoveNpcStopFrom; 3]>,
}

impl InstActionMoveNpcStop {
    #[inline]
    fn from_rkyv(archived: &ArchivedTmplActionMoveNpcStop) -> InstActionMoveNpcStop {
        InstActionMoveNpcStop {
            anim: InstAnimation::from_rkyv(&archived.anim),
            enter_from_table: archived
                .enter_from_table
                .iter()
                .map(InstActionMoveNpcStopFrom::from_rkyv)
                .collect(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InstActionMoveNpc {
    pub _base: InstActionBase,
    pub poise_level: u16,
    pub anim_move: InstAnimation,
    pub move_speed: f32,
    pub speed_ratio: f32,
    pub anim_start: InstAnimation,
    pub stops: Vec<InstActionMoveNpcStop>,
    pub turn_time: f32,
    pub min_distance: f32,
    pub step_length: f32,
}

extend!(InstActionMoveNpc, InstActionBase);

unsafe impl InstActionAny for InstActionMoveNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::MoveNpc
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<InstDeriveRule>) {}
}

impl InstActionMoveNpc {
    pub(crate) fn new_from_action(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionMoveNpc>,
    ) -> Option<InstActionMoveNpc> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        let stops = tmpl.stops.iter().map(InstActionMoveNpcStop::from_rkyv).collect();

        Some(InstActionMoveNpc {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: Some(VirtualKeyDir::new(tmpl.enter_key, None)),
                ..Default::default()
            },
            poise_level: tmpl.poise_level.to_native(),
            anim_move: InstAnimation::from_rkyv(&tmpl.anim_move),
            move_speed: tmpl.move_speed.to_native(),
            speed_ratio: tmpl.speed_ratio.to_native(),
            anim_start: InstAnimation::from_rkyv(&tmpl.anim_start),
            stops,
            turn_time: tmpl.turn_time.to_native(),
            min_distance: tmpl.min_distance.to_native(),
            step_length: tmpl.step_length.to_native(),
        })
    }

    #[inline]
    pub fn animations(&self) -> impl Iterator<Item = &InstAnimation> {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                yield &self.anim_start;
                yield &self.anim_move;
                for stop in &self.stops {
                    yield &stop.anim;
                }
            },
        )
    }

    #[inline]
    pub fn animations_count(&self) -> usize {
        2 + self.stops.len()
    }

    pub fn find_stop_by_anim_ratio(&self, anim_files: Symbol, anim_ratio: f32, wrapping: bool) -> Option<(usize, f32)> {
        let mut best_stop = None;
        let mut min_delta_ratio = f32::MAX;

        for (stop_idx, stop) in self.stops.iter().enumerate() {
            for from in &stop.enter_from_table {
                if from.anim != anim_files {
                    continue;
                }

                let delta_ratio = if from.ratio >= anim_ratio {
                    from.ratio - anim_ratio
                }
                else if wrapping {
                    1.0 - anim_ratio + from.ratio
                }
                else {
                    continue;
                };

                if delta_ratio < min_delta_ratio {
                    min_delta_ratio = delta_ratio;
                    best_stop = Some((stop_idx, from.ratio));
                }
            }
        }

        best_stop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{DtHashMap, VirtualKey, cf2s, id, sb};

    #[test]
    fn test_new() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = DtHashMap::default();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };

        let tmpl_act = db
            .find_as::<TmplActionMoveNpc>(id!("Action.InstanceNpc.Walk^1A"))
            .unwrap();
        let inst_act = InstActionMoveNpc::new_from_action(&ctx, tmpl_act).unwrap();

        assert_eq!(inst_act.tmpl_id, id!("Action.InstanceNpc.Walk^1A"));
        assert_eq!(inst_act.tags, vec![sb!("Walk")]);
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Walk, None));
        assert_eq!(inst_act.enter_level, 0);
        assert_eq!(inst_act.poise_level, 0);

        assert_eq!(inst_act.anim_move.files, sb!("Slime/WalkLoop.*"));
        assert_eq!(inst_act.anim_move.local_id, 1);
        assert_eq!(inst_act.anim_move.duration, cf2s(80));
        assert_eq!(inst_act.anim_move.fade_in, 0.1);
        assert_eq!(inst_act.anim_move.root_motion, true);
        assert_eq!(inst_act.anim_move.weapon_motion, false);
        assert_eq!(inst_act.anim_move.hit_motion, false);
        assert_eq!(inst_act.move_speed, 1.5);
        assert_eq!(inst_act.speed_ratio, 1.0);

        assert_eq!(inst_act.anim_start.files, sb!("Slime/WalkStart.*"));
        assert_eq!(inst_act.anim_start.local_id, 0);
        assert_eq!(inst_act.anim_start.duration, cf2s(40));
        assert_eq!(inst_act.anim_start.fade_in, 0.1);
        assert_eq!(inst_act.anim_start.root_motion, true);
        assert_eq!(inst_act.anim_start.weapon_motion, false);
        assert_eq!(inst_act.anim_start.hit_motion, false);

        assert_eq!(inst_act.stops.len(), 1);
        assert_eq!(inst_act.stops[0].anim.files, sb!("Slime/WalkStop.*"));
        assert_eq!(inst_act.stops[0].anim.local_id, 2);
        assert_eq!(inst_act.stops[0].anim.duration, cf2s(40));
        assert_eq!(inst_act.stops[0].anim.fade_in, 0.1);
        assert_eq!(inst_act.stops[0].anim.root_motion, true);
        assert_eq!(inst_act.stops[0].anim.weapon_motion, false);
        assert_eq!(inst_act.stops[0].anim.hit_motion, false);
        assert_eq!(inst_act.stops[0].enter_from_table.as_slice(), &[
            InstActionMoveNpcStopFrom {
                anim: sb!("Slime/WalkStart.*"),
                ratio: 1.0
            },
            InstActionMoveNpcStopFrom {
                anim: sb!("Slime/WalkLoop.*"),
                ratio: 0.5
            },
            InstActionMoveNpcStopFrom {
                anim: sb!("Slime/WalkLoop.*"),
                ratio: 1.0
            }
        ]);

        assert_eq!(inst_act.turn_time, cf2s(12));
        assert!((inst_act.min_distance - 1.8001196).abs() < 1e-6);
        assert!((inst_act.step_length - 1.0001197).abs() < 1e-6);
        assert_eq!(inst_act.animations_count(), 3);
    }

    #[test]
    fn test_find_stop_by_anim_ratio() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = DtHashMap::default();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };

        let tmpl_act = db
            .find_as::<TmplActionMoveNpc>(id!("Action.InstanceNpc.Walk^1A"))
            .unwrap();
        let inst_act = InstActionMoveNpc::new_from_action(&ctx, tmpl_act).unwrap();

        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkStart.*"), 0.25, false),
            Some((0, 1.0))
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkLoop.*"), 0.25, false),
            Some((0, 0.5))
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkLoop.*"), 0.75, false),
            Some((0, 1.0))
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkLoop.*"), 0.75, true),
            Some((0, 1.0))
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkLoop.*"), 1.0, true),
            Some((0, 1.0))
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkLoop.*"), 1.0, false),
            Some((0, 1.0))
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkLoop.*"), 1.1, false),
            None
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("Slime/WalkLoop.*"), 1.1, true),
            Some((0, 0.5))
        );
        assert_eq!(
            inst_act.find_stop_by_anim_ratio(sb!("TrainingDummy/Idle.*"), 0.0, true),
            None
        );
    }
}
