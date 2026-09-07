use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule,
};
use crate::instance::action::move_free_npc::InstActionMoveFreeNpcStop;
use crate::template::{At, TmplActionMoveTowardNpc};
use crate::utils::{ActionType, VirtualKeyDir, extend, sb};

#[derive(Debug)]
pub struct InstActionMoveTowardNpcDir {
    pub enter_angle: f32,
    pub anim_move: InstAnimation,
    pub move_speed: f32,
    pub speed_ratio: f32,
    pub anim_start: InstAnimation,
    pub anim_stops: Vec<InstActionMoveFreeNpcStop>,
    pub min_distance: f32,
    pub step_length: f32,
}

#[derive(Debug)]
#[repr(C)]
pub struct InstActionMoveTowardNpc {
    pub _base: InstActionBase,
    pub poise_level: u16,
    pub directions: Vec<InstActionMoveTowardNpcDir>,
    pub turn_time: f32,
}

extend!(InstActionMoveTowardNpc, InstActionBase);

unsafe impl InstActionAny for InstActionMoveTowardNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::MoveTowardNpc
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        for dir in &self.directions {
            animations.push(&dir.anim_move);
            animations.push(&dir.anim_start);
            for stop in &dir.anim_stops {
                animations.push(&stop.anim);
            }
        }
    }

    fn derives(&self, _derives: &mut Vec<InstDeriveRule>) {}
}

impl InstActionMoveTowardNpc {
    pub(crate) fn new_from_action(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionMoveTowardNpc>,
    ) -> Option<InstActionMoveTowardNpc> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        let mut directions = Vec::new();
        for tmpl_dir in tmpl.directions.iter() {
            let anim_move = InstAnimation::from_rkyv(&tmpl_dir.anim_move);
            let anim_start = InstAnimation::from_rkyv(&tmpl_dir.anim_start);

            // Calculate speed_ratio
            let speed_ratio = anim_move.duration / tmpl_dir.move_speed.to_native().max(0.001);

            // Placeholder for min_distance and step_length calculation
            // These would normally be calculated from animation root motion data
            let min_distance = 0.0;
            let step_length = 0.0;

            let anim_stops = tmpl_dir
                .stops
                .iter()
                .map(|stop| InstActionMoveFreeNpcStop {
                    anim: InstAnimation::from_rkyv(&stop.anim),
                    enter_from_table: stop
                        .enter_from_table
                        .iter()
                        .map(|f| crate::instance::action::move_free_npc::InstActionMoveNpcStopFrom {
                            anim: sb!(f.anim.as_str()),
                            ratio: f.ratio.to_native(),
                        })
                        .collect(),
                })
                .collect();

            directions.push(InstActionMoveTowardNpcDir {
                enter_angle: tmpl_dir.enter_angle.to_native(),
                anim_move,
                move_speed: tmpl_dir.move_speed.to_native(),
                speed_ratio,
                anim_start,
                anim_stops,
                min_distance,
                step_length,
            });
        }

        Some(InstActionMoveTowardNpc {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: Some(VirtualKeyDir::new(tmpl.enter_key, None)),
                ..Default::default()
            },
            poise_level: tmpl.poise_level.to_native(),
            directions,
            turn_time: tmpl.turn_time.to_native(),
        })
    }
}
