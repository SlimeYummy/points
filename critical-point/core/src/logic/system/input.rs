use approx::abs_diff_ne;
use cirtical_point_csgen::CsIn;
use glam::{Vec2, Vec2Swizzles, Vec3A};
use std::cell::RefCell;
use std::collections::{vec_deque, VecDeque};
use std::f32::consts::{FRAC_PI_2, PI};
use std::rc::Rc;

use crate::consts::{DEFAULT_VIEW_DIR_2D, DEFAULT_VIEW_DIR_3D, FPS_USIZE, MAX_PLAYER};
use crate::utils::{xerrf, xres, xresf, NumID, RawEvent, RawKey, VirtualEvent, XResult, MIN_PLAYER_ID};

const FIRST_EVENT_ID: u64 = 1;

//
// Input events collections
//

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    CsIn,
)]
#[cs_attr(Class)]
pub struct InputPlayerEvents {
    pub player_id: NumID,
    pub frame: u32,
    pub events: Vec<RawEvent>,
}

impl InputPlayerEvents {
    #[inline]
    pub fn new(player_id: NumID, frame: u32, events: Vec<RawEvent>) -> InputPlayerEvents {
        InputPlayerEvents {
            player_id,
            frame,
            events,
        }
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct InputFrameEvents {
    pub frame: u32,
    pub player_events: Vec<InputPlayerEvents>,
}

impl InputFrameEvents {
    #[inline]
    pub fn new(frame: u32, player_events: &[InputPlayerEvents]) -> InputFrameEvents {
        InputFrameEvents {
            frame,
            player_events: player_events.to_vec(),
        }
    }
}

//
// Input system
//

#[derive(Debug)]
pub struct SystemInput {
    // Input event queue for all players, including local player and network remote players.
    queues: Vec<Rc<RefCell<InputEventQueue>>>,

    // Pre-input window in frame.
    input_window: u32,

    // The frame that have been synchronized with other network remote players.
    // This means:
    //   1. The all players' current_frame >= synced_frame.
    //   2. The local player's game has been running for at least synced_frame.
    //   3. Input events for all remote players that occurred prior to synced_frame have been successfully received.
    synced_frame: u32,

    current_frames: Vec<u32>,
}

impl SystemInput {
    pub fn new(input_window: u32) -> SystemInput {
        SystemInput {
            queues: vec![],
            input_window,
            synced_frame: 0,
            current_frames: vec![],
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn player_count(&self) -> u32 {
        self.queues.len() as u32
    }

    #[inline]
    #[allow(dead_code)]
    pub fn input_window(&self) -> u32 {
        self.input_window
    }

    #[inline]
    #[allow(dead_code)]
    pub fn synced_frame(&self) -> u32 {
        self.synced_frame
    }

    #[inline]
    #[allow(dead_code)]
    pub fn unsynced_frame(&self) -> u32 {
        self.synced_frame + 1
    }

    pub fn init(&mut self, player_count: usize) -> XResult<()> {
        if !self.queues.is_empty() {
            return xres!(BadOperation; "queue empty");
        }
        if player_count <= 0 || player_count > MAX_PLAYER {
            return xres!(BadArgument; "player count");
        }

        self.queues = Vec::with_capacity(player_count);
        for player_idx in 0..player_count {
            let player_id = (player_idx as u64) + MIN_PLAYER_ID;
            let queue = Rc::new(RefCell::new(InputEventQueue::new(player_id, self.input_window)));
            self.queues.push(queue);
        }
        self.current_frames = vec![0; player_count];
        Ok(())
    }

    // Returns the frame which the game should restore to.
    pub fn produce(&mut self, player_events: &[InputPlayerEvents]) -> XResult<u32> {
        let base_frame = player_events.iter().map(|e| e.frame.wrapping_sub(1)).min().unwrap_or(0);

        for events in player_events {
            let player_idx = events.player_id.wrapping_sub(MIN_PLAYER_ID);
            match self.queues.get(player_idx as usize) {
                Some(queue) => {
                    let mut queue = queue.borrow_mut();
                    queue.produce(events.frame, &events.events)?;
                    self.current_frames[player_idx as usize] = queue.current_frame;
                }
                None => {
                    return xresf!(NotFound; "player_id={}", events.player_id);
                }
            };
        }

        self.synced_frame = *self.current_frames[0..self.queues.len()].iter().min().unwrap_or(&0);
        Ok(base_frame)
    }

    pub fn confirm(&mut self) -> XResult<()> {
        for queue in &mut self.queues {
            queue.borrow_mut().confirm(self.synced_frame)?;
        }
        Ok(())
    }

    #[inline]
    pub fn player_events(&mut self, player_id: NumID) -> XResult<Rc<RefCell<InputEventQueue>>> {
        let player_idx = player_id.wrapping_sub(MIN_PLAYER_ID);
        match self.queues.get(player_idx as usize) {
            Some(queue) => Ok(queue.clone()),
            None => xresf!(NotFound; "player_id={}", player_id),
        }
    }
}

//
// Input variables
//

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct InputMoveState {
    pub moving: bool,
    pub slow: bool,
    pub direction: Vec2, // device direction
}

impl InputMoveState {
    const EMPTY: InputMoveState = InputMoveState {
        moving: false,
        slow: false,
        direction: Vec2::ZERO,
    };

    #[inline]
    pub fn new(direction: Vec2) -> InputMoveState {
        if abs_diff_ne!(direction, Vec2::ZERO) {
            InputMoveState {
                moving: true,
                slow: direction.length_squared() < 0.25,
                direction: direction.normalize(),
            }
        } else {
            InputMoveState {
                moving: false,
                slow: false,
                direction: Vec2::ZERO,
            }
        }
    }

    #[inline]
    pub fn move_dir(&self) -> Option<Vec2> {
        match self.moving {
            true => Some(self.direction),
            false => None,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct InputVariables {
    pub view_rads: Vec2,
    pub view_dir_2d: Vec2,
    pub view_dir_3d: Vec3A,
    pub device_move: InputMoveState,
    pub optimized_device_move: InputMoveState,
}

impl Default for InputVariables {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl InputVariables {
    const EMPTY: InputVariables = InputVariables {
        view_rads: Vec2::ZERO,
        view_dir_2d: DEFAULT_VIEW_DIR_2D,
        view_dir_3d: DEFAULT_VIEW_DIR_3D,
        device_move: InputMoveState::EMPTY,
        optimized_device_move: InputMoveState::EMPTY,
    };

    #[inline]
    pub fn device_move(&self) -> InputMoveState {
        self.device_move
    }

    #[inline]
    pub fn optimized_device_move(&self) -> InputMoveState {
        self.optimized_device_move
    }

    pub fn world_move(&self) -> InputMoveState {
        if !self.device_move.moving {
            InputMoveState::EMPTY
        } else {
            let angle = self.device_move.direction.yx(); // Adjust angle dir, +Y -> 0°
            let direction = angle.rotate(self.view_dir_2d);
            InputMoveState {
                moving: true,
                slow: self.device_move.slow,
                direction,
            }
        }
    }

    pub fn optimized_world_move(&self) -> InputMoveState {
        if !self.optimized_device_move.moving {
            self.optimized_device_move
        } else {
            let angle = self.optimized_device_move.direction.yx(); // Adjust angle dir, +Y -> 0°
            let direction = angle.rotate(self.view_dir_2d);
            InputMoveState {
                moving: true,
                slow: self.optimized_device_move.slow,
                direction,
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct InputFrameMeta {
    frame: u32,
    start_event_id: u64,
    end_next_event_id: u64, // end_event_idx + 1
    variables: InputVariables,
}

impl InputFrameMeta {
    const EMPTY: InputFrameMeta = InputFrameMeta {
        frame: 0,
        start_event_id: FIRST_EVENT_ID,
        end_next_event_id: FIRST_EVENT_ID,
        variables: InputVariables::EMPTY,
    };

    #[inline]
    #[allow(dead_code)]
    fn events_count(&self) -> usize {
        (self.end_next_event_id - self.start_event_id) as usize
    }
}

#[derive(Debug)]
pub struct InputEventQueue {
    player_id: NumID,
    input_window: u32,
    id_counter: u64, // Event id counter, starts from 1
    events: VecDeque<VirtualEvent>,
    metas: VecDeque<InputFrameMeta>,
    current_frame: u32, // The lastest frame which events has been produced (reached) currently.
    synced_frame: u32,  // The frame that have been synchronized with other players.
    base_frame: u32,    // The smallest frame saved in self.events. Previous frames have been discarded.
    base_id: u64,       // The smallest id saved in self.events. Previous events have been discarded.
    variables: InputVariables,
}

impl InputEventQueue {
    fn new(player_id: NumID, input_window: u32) -> InputEventQueue {
        InputEventQueue {
            player_id,
            input_window,
            id_counter: FIRST_EVENT_ID,
            events: VecDeque::with_capacity(256),
            metas: VecDeque::with_capacity((2 * FPS_USIZE) + (input_window as usize)),
            current_frame: 0,
            synced_frame: 0,
            base_frame: 1,
            base_id: FIRST_EVENT_ID,
            variables: InputVariables::default(),
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn player_id(&self) -> NumID {
        self.player_id
    }

    // The next unproduced future event id.
    #[inline]
    pub fn future_id(&self) -> u64 {
        self.id_counter
    }

    #[inline]
    #[allow(dead_code)]
    pub fn current_frame(&self) -> u32 {
        self.current_frame
    }

    #[inline]
    #[allow(dead_code)]
    pub fn synced_frame(&self) -> u32 {
        self.synced_frame
    }

    #[inline]
    #[allow(dead_code)]
    pub fn unsynced_frame(&self) -> u32 {
        self.synced_frame() + 1
    }

    #[inline]
    #[allow(dead_code)]
    pub fn base_frame(&self) -> u32 {
        self.base_frame
    }

    #[inline]
    #[allow(dead_code)]
    pub fn base_id(&self) -> u64 {
        self.base_id
    }

    fn produce(&mut self, current_frame: u32, events: &[RawEvent]) -> XResult<()> {
        if current_frame != self.current_frame + 1 {
            return xresf!(BadArgument; "current_frame={}, self.current_frame={}", current_frame, self.current_frame);
        }
        self.current_frame += 1;

        self.variables.optimized_device_move = self.variables.device_move;
        let start_event_id = self.id_counter;

        for (n, event) in events.iter().enumerate() {
            // View
            if event.key == RawKey::View {
                self.variables.view_rads = Vec2::new(event.motion.x % (2.0 * PI), event.motion.y % FRAC_PI_2);
                self.variables.view_dir_2d = Vec2::from_angle(event.motion.x);
                let cos_x = self.variables.view_dir_2d.x;
                let sin_x = self.variables.view_dir_2d.y;
                let (sin_y, cos_y) = libm::sincosf(self.variables.view_rads.y);
                self.variables.view_dir_3d = Vec3A::new(cos_y * cos_x, sin_y, cos_y * sin_x);

            // Move
            } else if event.key == RawKey::Move {
                if event.pressed {
                    self.variables.device_move = InputMoveState::new(event.motion);
                    self.variables.optimized_device_move = self.variables.device_move;
                } else {
                    self.variables.device_move = InputMoveState::default();
                    // Ignore Up event, if a Down-Up pair is the last operation.
                    let ignore = (n == events.len() - 1)
                        && (n >= 1)
                        && events[n - 1].key == RawKey::Move
                        && events[n - 1].pressed;
                    if !ignore {
                        self.variables.optimized_device_move = InputMoveState::default();
                    }
                }

            // Other events
            } else {
                self.events.push_back(VirtualEvent::new_ex(
                    self.id_counter,
                    current_frame,
                    event.key.into(),
                    event.pressed,
                    self.variables.view_dir_2d,
                    self.variables.view_dir_3d,
                    self.variables.world_move().direction,
                ));
                self.id_counter += 1;
            }
        }

        self.metas.push_back(InputFrameMeta {
            frame: current_frame,
            start_event_id,
            end_next_event_id: self.id_counter,
            variables: self.variables,
        });
        Ok(())
    }

    fn confirm(&mut self, synced_frame: u32) -> XResult<()> {
        if synced_frame > self.current_frame {
            return xresf!(BadArgument; "synced_frame={}, self.current_frame={}", synced_frame, self.current_frame);
        }
        if synced_frame <= self.synced_frame {
            return Ok(());
        }

        self.synced_frame = synced_frame;

        let base_frame = synced_frame.saturating_sub(self.input_window).max(self.base_frame);
        if base_frame <= self.base_frame {
            return Ok(());
        }

        for frame in self.base_frame..base_frame {
            let meta = self.metas.pop_front();
            assert!(meta.map(|m| m.frame == frame) == Some(true));
        }

        let new_base_id = match self.metas.front() {
            Some(meta) => meta.start_event_id,
            None => return xres!(Unexpected; "metas empty"),
        };
        while let Some(event) = self.events.front() {
            if event.id < new_base_id {
                assert!(event.frame >= self.base_frame && event.frame < base_frame);
                self.events.pop_front();
            } else {
                break;
            }
        }

        self.base_frame = base_frame;
        self.base_id = new_base_id;
        Ok(())
    }

    pub fn iter_current(&self, frame: u32) -> XResult<vec_deque::Iter<'_, VirtualEvent>> {
        if frame > self.current_frame {
            return Ok(self.events.range(0..0));
        }
        let meta = self
            .index_meta(frame) // Checked frame < self.base_frame
            .ok_or_else(
                || xerrf!(Overflow; "frame={}, base_frame={}, metas.len={}", frame, self.base_frame, self.metas.len()),
            )?;
        let start = (meta.start_event_id - self.base_id) as usize;
        let end = (meta.end_next_event_id - self.base_id) as usize;
        Ok(self.events.range(start..end))
    }

    pub fn iter_preinput(&self, frame: u32, cursor_id: u64) -> XResult<vec_deque::Iter<'_, VirtualEvent>> {
        if cursor_id > self.future_id() {
            return xresf!(BadArgument; "cursor_id={}, self.future_id={}", cursor_id, self.future_id());
        }
        let current_meta = self.index_or_last_meta(frame)?; // Checked frame < self.base_frame

        let end_id = if frame > self.current_frame {
            current_meta.end_next_event_id
        } else {
            current_meta.start_event_id
        };

        let mut start_id;
        if let Some(window_start_meta) = self.index_meta(frame.wrapping_sub(self.input_window)) {
            start_id = window_start_meta.start_event_id
        } else if frame < self.base_frame + self.input_window {
            // frame - input_window < base_frame
            start_id = match self.metas.front() {
                Some(meta) => meta.start_event_id,
                None => FIRST_EVENT_ID,
            };
        } else {
            // frame - input_window > current_frame
            return Ok(self.events.range(0..0));
        };
        start_id = start_id.max(cursor_id).min(end_id);

        let start = (start_id - self.base_id) as usize;
        let end = (end_id - self.base_id) as usize;
        Ok(self.events.range(start..end))
    }

    #[inline]
    pub fn variables(&self, frame: u32) -> XResult<InputVariables> {
        self.index_or_last_meta(frame).map(|meta| meta.variables)
    }

    #[inline]
    fn index_meta(&self, frame: u32) -> Option<&InputFrameMeta> {
        self.metas.get((frame.wrapping_sub(self.base_frame)) as usize)
    }

    #[inline]
    fn index_or_last_meta(&self, frame: u32) -> XResult<&InputFrameMeta> {
        if frame > self.current_frame {
            return Ok(self.metas.back().unwrap_or(&InputFrameMeta::EMPTY));
        }
        self.index_meta(frame).ok_or_else(
            || xerrf!(Overflow; "frame={}, base_frame={}, metas.len={}", frame, self.base_frame, self.metas.len()),
        )
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;
    use crate::utils::VirtualKey;
    use std::f32::consts::FRAC_1_SQRT_2;

    fn collect_frame(iq: &InputEventQueue) -> Vec<u32> {
        iq.metas.iter().map(|m| m.frame).collect()
    }

    fn collect_start_event_id(iq: &InputEventQueue) -> Vec<u64> {
        iq.metas.iter().map(|m| m.start_event_id).collect()
    }

    fn collect_xend_event_id(iq: &InputEventQueue) -> Vec<u64> {
        iq.metas.iter().map(|m| m.end_next_event_id).collect()
    }

    fn evt(id: u64, frame: u32, raw: RawEvent) -> VirtualEvent {
        VirtualEvent::new(id, frame, raw.key.into(), raw.pressed)
    }

    #[test]
    fn test_input_queue_empty() {
        let mut iq: InputEventQueue = InputEventQueue::new(100, 3);
        assert_eq!(iq.current_frame, 0);
        assert_eq!(iq.synced_frame, 0);
        assert_eq!(iq.base_frame, 1);
        assert_eq!(iq.base_id, 1);

        assert!(iq.produce(0, &[]).is_err());
        assert!(iq.produce(5, &[]).is_err());

        assert!(iq.confirm(0).is_ok());
        assert!(iq.confirm(1).is_err());
        assert!(iq.confirm(7).is_err());
        assert_eq!(iq.current_frame, 0);
        assert_eq!(iq.synced_frame, 0);
        assert_eq!(iq.base_frame, 1);
        assert_eq!(iq.base_id, 1);

        assert!(iq.iter_current(0).is_err());
        assert_eq!(iq.iter_current(1).unwrap().count(), 0);
        assert_eq!(iq.iter_current(100).unwrap().count(), 0);
        assert!(iq.iter_preinput(0, 0).is_err());
        assert_eq!(iq.iter_preinput(1, 0).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(100, 0).unwrap().count(), 0);

        iq.produce(1, &[]).unwrap();
        assert_eq!(iq.current_frame, 1);
        assert_eq!(
            iq.metas,
            vec![InputFrameMeta {
                frame: 1,
                start_event_id: 1,
                end_next_event_id: 1,
                variables: InputVariables::EMPTY
            }]
        );
        assert!(iq.iter_current(0).is_err());
        assert_eq!(iq.iter_current(1).unwrap().count(), 0);
        assert_eq!(iq.iter_current(100).unwrap().count(), 0);
        assert!(iq.iter_preinput(0, 100).is_err());
        assert!(iq.iter_preinput(1, 100).is_err());
        assert!(iq.iter_preinput(100, 100).is_err());

        iq.produce(2, &[]).unwrap();
        iq.produce(3, &[]).unwrap();
        iq.produce(4, &[]).unwrap();
        assert_eq!(iq.current_frame, 4);
        assert_eq!(collect_frame(&iq), vec![1, 2, 3, 4]);
        assert_eq!(collect_start_event_id(&iq), vec![1, 1, 1, 1]);
        assert_eq!(collect_xend_event_id(&iq), vec![1, 1, 1, 1]);
        assert!(iq.iter_current(0).is_err());
        assert_eq!(iq.iter_current(4).unwrap().count(), 0);
        assert_eq!(iq.iter_current(6).unwrap().count(), 0);
        assert!(iq.iter_preinput(0, 0).is_err());
        assert_eq!(iq.iter_preinput(4, 0).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(6, 0).unwrap().count(), 0);
    }

    #[test]
    fn test_input_queue_produce() {
        let a1_down = RawEvent::new_button(RawKey::Attack1, true);
        let a1_up = RawEvent::new_button(RawKey::Attack1, false);
        let a2_down = RawEvent::new_button(RawKey::Attack2, true);
        let a2_up = RawEvent::new_button(RawKey::Attack2, false);
        let s1_down = RawEvent::new_button(RawKey::Skill1, true);
        let s1_up = RawEvent::new_button(RawKey::Skill1, false);

        let mut iq = InputEventQueue::new(0, 3);

        // produce=1
        iq.produce(1, &[a1_down, a1_up]).unwrap();
        assert_eq!(iq.current_frame, 1);
        assert_eq!(collect_start_event_id(&iq), vec![1]);
        assert_eq!(collect_xend_event_id(&iq), vec![3]);
        assert!(iq.iter_current(0).is_err());

        assert_eq!(
            iq.iter_current(1).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, a1_down), &evt(2, 1, a1_up)]
        );
        assert_eq!(iq.iter_preinput(1, 0).unwrap().count(), 0);

        assert_eq!(iq.iter_current(2).unwrap().count(), 0);
        assert_eq!(
            iq.iter_preinput(2, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, a1_down), &evt(2, 1, a1_up)]
        );

        assert_eq!(iq.iter_current(4).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(4, 0).unwrap().count(), 2);
        assert_eq!(iq.iter_preinput(4, 1).unwrap().count(), 2);
        assert_eq!(iq.iter_preinput(4, 2).unwrap().count(), 1);
        assert_eq!(iq.iter_preinput(4, 3).unwrap().count(), 0);
        assert!(iq.iter_preinput(4, 4).is_err());

        assert_eq!(iq.iter_current(5).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(5, 0).unwrap().count(), 0);

        // produce=3
        iq.produce(2, &[]).unwrap();
        iq.produce(3, &[a2_down]).unwrap();
        assert_eq!(iq.current_frame, 3);
        assert_eq!(collect_start_event_id(&iq), vec![1, 3, 3]);
        assert_eq!(collect_xend_event_id(&iq), vec![3, 3, 4]);
        assert!(iq.iter_current(0).is_err());

        assert_eq!(
            iq.iter_current(1).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, a1_down), &evt(2, 1, a1_up)]
        );
        assert_eq!(iq.iter_preinput(1, 0).unwrap().count(), 0);

        assert_eq!(
            iq.iter_current(3).unwrap().collect::<Vec<_>>(),
            vec![&evt(3, 3, a2_down)]
        );
        assert_eq!(
            iq.iter_preinput(3, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, a1_down), &evt(2, 1, a1_up)]
        );

        assert_eq!(iq.iter_current(5).unwrap().count(), 0);
        assert_eq!(
            iq.iter_preinput(5, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(3, 3, a2_down)]
        );

        assert_eq!(iq.iter_preinput(7, 0).unwrap().count(), 0);

        // produce=6
        iq.produce(4, &[a2_up]).unwrap();
        iq.produce(5, &[]).unwrap();
        iq.produce(6, &[s1_down, s1_up]).unwrap();
        assert_eq!(iq.current_frame, 6);
        assert_eq!(collect_start_event_id(&iq), vec![1, 3, 3, 4, 5, 5]);
        assert_eq!(collect_xend_event_id(&iq), vec![3, 3, 4, 5, 5, 7]);

        assert_eq!(
            iq.iter_current(1).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, a1_down), &evt(2, 1, a1_up)]
        );
        assert_eq!(iq.iter_preinput(1, 0).unwrap().count(), 0);

        assert_eq!(
            iq.iter_current(3).unwrap().collect::<Vec<_>>(),
            vec![&evt(3, 3, a2_down)]
        );
        assert_eq!(
            iq.iter_preinput(3, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, a1_down), &evt(2, 1, a1_up)]
        );

        assert_eq!(
            iq.iter_current(6).unwrap().collect::<Vec<_>>(),
            vec![&evt(5, 6, s1_down), &evt(6, 6, s1_up)]
        );
        assert_eq!(
            iq.iter_preinput(6, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(3, 3, a2_down), &evt(4, 4, a2_up)]
        );
        assert_eq!(iq.iter_preinput(6, 3).unwrap().count(), 2);
        assert_eq!(iq.iter_preinput(6, 4).unwrap().count(), 1);
        assert_eq!(iq.iter_preinput(6, 5).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(6, 6).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(6, 7).unwrap().count(), 0);
        assert!(iq.iter_preinput(6, 8).is_err());
    }

    #[test]
    fn test_input_queue_confirm() {
        let s1_down = RawEvent::new_button(RawKey::Skill1, true);
        let s1_up = RawEvent::new_button(RawKey::Skill1, false);
        let s2_down = RawEvent::new_button(RawKey::Skill2, true);
        let s2_up = RawEvent::new_button(RawKey::Skill2, false);
        let aim = RawEvent::new_button(RawKey::Aim, true);
        let shoot = RawEvent::new_button(RawKey::Shot1, true);

        let mut iq = InputEventQueue::new(0, 3);

        // produce=0
        iq.produce(1, &[s1_down, s1_up]).unwrap();
        iq.confirm(1).unwrap();
        assert_eq!(iq.current_frame, 1);
        assert_eq!(iq.synced_frame, 1);
        assert_eq!(iq.base_frame, 1);
        assert_eq!(iq.base_id, 1);
        assert_eq!(collect_start_event_id(&iq), vec![1]);
        assert_eq!(collect_xend_event_id(&iq), vec![3]);
        assert!(iq.iter_current(0).is_err());

        assert_eq!(
            iq.iter_current(1).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, s1_down), &evt(2, 1, s1_up)]
        );
        assert_eq!(iq.iter_preinput(1, 0).unwrap().count(), 0);

        assert_eq!(iq.iter_current(2).unwrap().count(), 0);
        assert_eq!(
            iq.iter_preinput(2, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, s1_down), &evt(2, 1, s1_up)]
        );

        assert_eq!(iq.iter_current(4).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(4, 0).unwrap().count(), 2);

        assert_eq!(iq.iter_current(5).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(5, 0).unwrap().count(), 0);

        // produce=3
        iq.produce(2, &[s2_down]).unwrap();
        iq.produce(3, &[s2_down, aim]).unwrap();
        iq.confirm(3).unwrap();
        assert_eq!(iq.current_frame, 3);
        assert_eq!(iq.synced_frame, 3);
        assert_eq!(iq.base_frame, 1);
        assert_eq!(iq.base_id, 1);
        assert_eq!(collect_start_event_id(&iq), vec![1, 3, 4]);
        assert_eq!(collect_xend_event_id(&iq), vec![3, 4, 6]);
        assert_eq!(iq.iter_current(3).unwrap().count(), 2);
        assert_eq!(iq.iter_preinput(3, 0).unwrap().count(), 3);

        // produce=5
        iq.produce(4, &[]).unwrap();
        iq.produce(5, &[]).unwrap();
        iq.confirm(5).unwrap();
        assert_eq!(iq.current_frame, 5);
        assert_eq!(iq.synced_frame, 5);
        assert_eq!(iq.base_frame, 2);
        assert_eq!(iq.base_id, 3);
        assert_eq!(collect_start_event_id(&iq), vec![3, 4, 6, 6]);
        assert_eq!(collect_xend_event_id(&iq), vec![4, 6, 6, 6]);

        assert_eq!(iq.iter_current(4).unwrap().count(), 0);
        assert_eq!(
            iq.iter_preinput(4, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(3, 2, s2_down), &evt(4, 3, s2_down), &evt(5, 3, aim)]
        );
        assert_eq!(iq.iter_preinput(6, 0).unwrap().count(), 2);

        // produce=7
        iq.produce(6, &[shoot]).unwrap();
        iq.produce(7, &[s2_down, s2_up]).unwrap();
        iq.confirm(6).unwrap();
        assert_eq!(iq.current_frame, 7);
        assert_eq!(iq.synced_frame, 6);
        assert_eq!(iq.base_frame, 3);
        assert_eq!(iq.base_id, 4);
        assert_eq!(collect_start_event_id(&iq), vec![4, 6, 6, 6, 7]);
        assert_eq!(collect_xend_event_id(&iq), vec![6, 6, 6, 7, 9]);

        assert_eq!(iq.iter_current(6).unwrap().collect::<Vec<_>>(), vec![&evt(6, 6, shoot)]);
        assert_eq!(
            iq.iter_preinput(6, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(4, 3, s2_down), &evt(5, 3, aim)]
        );

        assert_eq!(
            iq.iter_current(7).unwrap().collect::<Vec<_>>(),
            vec![&evt(7, 7, s2_down), &evt(8, 7, s2_up)]
        );
        assert_eq!(
            iq.iter_preinput(7, 0).unwrap().collect::<Vec<_>>(),
            vec![&evt(6, 6, shoot)]
        );

        assert_eq!(iq.iter_current(10).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(10, 0).unwrap().count(), 2);
        assert_eq!(iq.iter_preinput(10, 8).unwrap().count(), 1);
        assert_eq!(iq.iter_preinput(10, 9).unwrap().count(), 0);
        assert!(iq.iter_preinput(10, 10).is_err());

        assert_eq!(iq.iter_current(11).unwrap().count(), 0);
        assert_eq!(iq.iter_preinput(11, 0).unwrap().count(), 0);
    }

    #[test]
    fn test_input_queue_view_move() {
        let a1_down = RawEvent::new_button(RawKey::Attack1, true);
        let a1_up = RawEvent::new_button(RawKey::Attack1, false);
        let a5_down = RawEvent::new_button(RawKey::Attack5, true);
        let a5_up = RawEvent::new_button(RawKey::Attack5, false);
        let s1_down = RawEvent::new_button(RawKey::Skill1, true);
        let s1_up = RawEvent::new_button(RawKey::Skill1, false);

        let mut iq = InputEventQueue::new(0, 3);
        iq.produce(
            1,
            &[
                a5_down,
                RawEvent::new_view(Vec2::new(PI, 0.0)),
                RawEvent::new_move(Vec2::new(0.0, -1.0)),
                a5_up,
                RawEvent::new_move(Vec2::new(0.0, 0.0)),
                a1_down,
            ],
        )
        .unwrap();
        let events = iq.iter_current(1).unwrap().collect::<Vec<_>>();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].key, VirtualKey::Attack5);
        assert_ulps_eq!(events[0].view_dir_2d, Vec2::NEG_Y);
        assert_ulps_eq!(events[0].view_dir_3d, Vec3A::NEG_Z);
        assert_ulps_eq!(events[0].world_move_dir, Vec2::ZERO);
        assert_eq!(events[1].key, VirtualKey::Attack5);
        assert_ulps_eq!(events[1].view_dir_2d, Vec2::NEG_X);
        assert_ulps_eq!(events[1].view_dir_3d, Vec3A::NEG_X);
        assert_ulps_eq!(events[1].world_move_dir, Vec2::X);
        assert_eq!(events[2].key, VirtualKey::Attack1);
        assert_ulps_eq!(events[2].view_dir_2d, Vec2::NEG_X);
        assert_ulps_eq!(events[2].view_dir_3d, Vec3A::NEG_X);
        assert_ulps_eq!(events[2].world_move_dir, Vec2::ZERO);
        assert_eq!(iq.variables.view_rads, Vec2::new(PI, 0.0));
        assert_eq!(iq.variables.device_move, InputMoveState::new(Vec2::ZERO));
        assert_eq!(iq.variables.optimized_device_move, InputMoveState::new(Vec2::ZERO));

        iq.produce(
            2,
            &[
                RawEvent::new_move(Vec2::new(1.0, 0.0)),
                RawEvent::new_move(Vec2::new(0.0, 0.0)),
            ],
        )
        .unwrap();
        assert_eq!(iq.iter_current(2).unwrap().count(), 0);
        assert_eq!(iq.variables.view_rads, Vec2::new(PI, 0.0));
        assert_eq!(iq.variables.device_move, InputMoveState::new(Vec2::ZERO));
        assert_eq!(
            iq.variables.optimized_device_move,
            InputMoveState::new(Vec2::new(1.0, 0.0))
        );

        iq.produce(3, &[]).unwrap();
        assert_eq!(iq.variables.device_move, InputMoveState::new(Vec2::ZERO));
        assert_eq!(iq.variables.optimized_device_move, InputMoveState::new(Vec2::ZERO));

        iq.produce(
            4,
            &[
                RawEvent::new_move(Vec2::new(0.3, 0.3)),
                RawEvent::new_view(Vec2::new(0.0, PI / 4.0)),
                a1_up,
            ],
        )
        .unwrap();
        assert_eq!(iq.variables.view_rads, Vec2::new(0.0, PI / 4.0));
        assert_eq!(iq.variables.device_move, InputMoveState::new(Vec2::new(0.3, 0.3)));
        assert_eq!(
            iq.variables.optimized_device_move,
            InputMoveState::new(Vec2::new(0.3, 0.3))
        );
        let events = iq.iter_current(4).unwrap().collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key, VirtualKey::Attack1);
        assert_ulps_eq!(events[0].view_dir_2d, Vec2::X);
        assert_ulps_eq!(events[0].view_dir_3d, Vec3A::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0));
        assert_ulps_eq!(events[0].world_move_dir, Vec2::new(1.0, 1.0).normalize());

        iq.produce(5, &[s1_down, s1_up]).unwrap();
        let events = iq.iter_current(5).unwrap().collect::<Vec<_>>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].key, VirtualKey::Skill1);
        assert_ulps_eq!(events[0].view_dir_2d, Vec2::X);
        assert_ulps_eq!(events[0].view_dir_3d, Vec3A::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0));
        assert_ulps_eq!(events[0].world_move_dir, Vec2::new(1.0, 1.0).normalize());
        assert_eq!(events[1].key, VirtualKey::Skill1);
        assert_ulps_eq!(events[0].view_dir_2d, Vec2::X);
        assert_ulps_eq!(events[0].view_dir_3d, Vec3A::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0));
        assert_ulps_eq!(events[0].world_move_dir, Vec2::new(1.0, 1.0).normalize());
    }

    #[test]
    fn test_input_system() {
        let a1_down = RawEvent::new_button(RawKey::Attack1, true);
        let a1_up = RawEvent::new_button(RawKey::Attack1, false);
        let s1_down = RawEvent::new_button(RawKey::Skill1, true);
        let s1_up = RawEvent::new_button(RawKey::Skill1, false);
        let aim = RawEvent::new_button(RawKey::Aim, false);
        let shot = RawEvent::new_button(RawKey::Shot1, false);

        let mut si = SystemInput::new(3);
        assert!(si.init(0).is_err());
        assert!(si.init(MAX_PLAYER + 1).is_err());
        si.init(2).unwrap();
        assert!(si.init(1).is_err());
        assert_eq!(si.player_count(), 2);
        assert_eq!(si.input_window(), 3);

        let player0 = si.player_events(100).unwrap();
        let player1 = si.player_events(101).unwrap();

        // p1=1, p2=?
        let events = vec![InputPlayerEvents {
            player_id: 100,
            frame: 1,
            events: vec![a1_down, a1_up],
        }];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.synced_frame(), 0);
        assert_eq!(si.unsynced_frame(), 1);
        assert_eq!(
            player0.borrow().iter_current(1).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, a1_down), &evt(2, 1, a1_up),]
        );
        assert_eq!(player0.borrow().iter_preinput(1, 0).unwrap().count(), 0);
        assert_eq!(player1.borrow().iter_current(1).unwrap().count(), 0);
        assert_eq!(player1.borrow().iter_preinput(1, 0).unwrap().count(), 0);

        // p1=1, p2=1
        let events = vec![InputPlayerEvents {
            player_id: 101,
            frame: 1,
            events: vec![s1_down],
        }];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.synced_frame(), 1);
        assert_eq!(si.unsynced_frame(), 2);
        assert_eq!(player0.borrow().iter_current(1).unwrap().count(), 2);
        assert_eq!(player0.borrow().iter_preinput(1, 0).unwrap().count(), 0);
        assert_eq!(
            player1.borrow().iter_current(1).unwrap().collect::<Vec<_>>(),
            vec![&evt(1, 1, s1_down)]
        );
        assert_eq!(player1.borrow().iter_preinput(1, 0).unwrap().count(), 0);

        // p1=3, p2=1
        let events = vec![
            InputPlayerEvents {
                player_id: 100,
                frame: 2,
                events: vec![],
            },
            InputPlayerEvents {
                player_id: 100,
                frame: 3,
                events: vec![aim, shot],
            },
        ];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.synced_frame(), 1);
        assert_eq!(si.unsynced_frame(), 2);
        assert_eq!(player0.borrow().iter_current(3).unwrap().count(), 2);
        assert_eq!(player0.borrow().iter_preinput(3, 0).unwrap().count(), 2);
        assert_eq!(player1.borrow().iter_current(3).unwrap().count(), 0);
        assert_eq!(player1.borrow().iter_preinput(3, 0).unwrap().count(), 1);

        // p1=3, p2=2
        let events = vec![InputPlayerEvents {
            player_id: 101,
            frame: 2,
            events: vec![s1_up],
        }];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.synced_frame(), 2);
        assert_eq!(si.unsynced_frame(), 3);
        assert_eq!(player0.borrow().iter_current(3).unwrap().count(), 2);
        assert_eq!(player0.borrow().iter_preinput(3, 0).unwrap().count(), 2);
        assert_eq!(player1.borrow().iter_current(2).unwrap().count(), 1);
        assert_eq!(player1.borrow().iter_preinput(3, 0).unwrap().count(), 2);
    }

    #[test]
    fn test_input_variables() {
        let mut iv = InputVariables::default();
        assert_eq!(iv.device_move(), InputMoveState::new(Vec2::ZERO));
        assert_eq!(iv.optimized_device_move(), InputMoveState::new(Vec2::ZERO));

        iv.device_move.moving = true;
        iv.device_move.direction = Vec2::Y;
        assert_eq!(
            iv.world_move(),
            InputMoveState {
                moving: true,
                slow: false,
                direction: Vec2::NEG_Y,
            }
        );
        iv.device_move.direction = Vec2::NEG_Y;
        assert_eq!(iv.world_move().direction, Vec2::Y);
        iv.device_move.direction = Vec2::X;
        assert_eq!(iv.world_move().direction, Vec2::X);
        iv.device_move.direction = Vec2::NEG_X;
        assert_eq!(iv.world_move().direction, Vec2::NEG_X);
        iv.device_move.direction = Vec2::new(1.0, 1.0).normalize();
        assert_eq!(iv.world_move().direction, Vec2::new(1.0, -1.0).normalize());
        iv.device_move.direction = Vec2::new(-1.0, 1.0).normalize();
        assert_eq!(iv.world_move().direction, Vec2::new(-1.0, -1.0).normalize());

        iv.view_dir_2d = Vec2::X;
        iv.optimized_device_move.moving = true;
        iv.optimized_device_move.direction = Vec2::Y;
        assert_eq!(
            iv.optimized_world_move(),
            InputMoveState {
                moving: true,
                slow: false,
                direction: Vec2::X,
            }
        );
        iv.optimized_device_move.direction = Vec2::NEG_Y;
        assert_eq!(iv.optimized_world_move().direction, Vec2::NEG_X);
        iv.optimized_device_move.direction = Vec2::X;
        assert_eq!(iv.optimized_world_move().direction, Vec2::Y);
        iv.optimized_device_move.direction = Vec2::NEG_X;
        assert_eq!(iv.optimized_world_move().direction, Vec2::NEG_Y);
    }
}
