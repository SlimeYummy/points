use cirtical_point_csgen::CsIn;
use glam::Vec2;
use std::collections::{vec_deque, VecDeque};

use crate::consts::{FPS, MAX_PLAYER};
use crate::utils::{KeyCode, KeyEvent, NumID, XError, XResult};

#[derive(Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Deserialize, CsIn)]
#[cs_attr(Class)]
pub struct PlayerKeyEvents {
    pub player_id: NumID,
    pub frame: u32,
    pub events: Vec<KeyEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Deserialize)]
pub struct InputEvent {
    pub idx: u64,
    pub frame: u32,
    pub event: KeyEvent,
}

impl InputEvent {
    #[inline]
    pub fn new(idx: u64, frame: u32, event: KeyEvent) -> InputEvent {
        InputEvent { idx, frame, event }
    }

    #[inline]
    pub fn key(&self) -> KeyCode {
        self.event.key
    }

    #[inline]
    pub fn pressed(&self) -> bool {
        self.event.pressed
    }

    #[inline]
    pub fn motion(&self) -> Vec2 {
        self.event.motion
    }
}

#[derive(Debug)]
pub struct SystemInput {
    queues: Vec<InputEventQueue>,
    input_window: u32,
    latest_frame: u32,
    synced_frame: u32,
}

impl SystemInput {
    pub fn new(input_window: u32) -> XResult<SystemInput> {
        if input_window <= 0 {
            return Err(XError::bad_argument("SystemInput::new() input_window"));
        }
        Ok(SystemInput {
            queues: Vec::with_capacity(MAX_PLAYER),
            input_window,
            latest_frame: 0,
            synced_frame: 0,
        })
    }

    #[inline]
    pub fn player_count(&self) -> u32 {
        self.queues.len() as u32
    }

    #[inline]
    pub fn input_window(&self) -> u32 {
        self.input_window
    }

    #[inline]
    pub fn latest_frame(&self) -> u32 {
        self.latest_frame
    }

    #[inline]
    pub fn synced_frame(&self) -> u32 {
        self.synced_frame
    }

    #[inline]
    pub fn unsynced_frame(&self) -> u32 {
        self.synced_frame + 1
    }

    pub fn init(&mut self, player_ids: &[NumID]) -> XResult<()> {
        if !self.queues.is_empty() {
            return Err(XError::invalid_operation("SystemInput::init() inited"));
        }
        if player_ids.len() > MAX_PLAYER {
            return Err(XError::bad_argument("SystemInput::init() player_ids"));
        }

        let mut normalized_ids = Vec::with_capacity(player_ids.len());
        for player_id in player_ids {
            if normalized_ids.contains(player_id) {
                return Err(XError::bad_argument("SystemInput::init() player_ids"));
            }
            normalized_ids.push(*player_id);
        }
        normalized_ids.sort_unstable();

        for player_id in normalized_ids {
            self.queues.push(InputEventQueue::new(player_id, self.input_window)?);
        }
        Ok(())
    }

    // Returns the frame which the game should restore to.
    pub fn produce(&mut self, player_events: &[PlayerKeyEvents]) -> XResult<Option<u32>> {
        let base_frame = player_events.iter().map(|e| e.frame.wrapping_sub(1)).min();

        for events in player_events {
            self.queues
                .iter_mut()
                .find(|q| q.player_id == events.player_id)
                .ok_or_else(|| XError::bad_argument(format!("player_id not found ({})", events.player_id)))?
                .produce(events.frame, &events.events)?;
        }

        self.latest_frame = 0;
        self.synced_frame = u32::MAX;
        for queue in &mut self.queues {
            self.latest_frame = self.latest_frame.max(queue.current_frame);
            self.synced_frame = self.synced_frame.min(queue.current_frame);
            queue.clear_enter_event();
        }
        Ok(base_frame)
    }

    pub fn confirm(&mut self) -> XResult<()> {
        for queue in &mut self.queues {
            queue.confirm(self.synced_frame, self.input_window)?;
        }
        Ok(())
    }

    pub fn player_events(&mut self, player_id: NumID, frame: u32) -> XResult<InputQueueAgent<'_>> {
        let queue = match self.queues.iter_mut().find(|q| q.player_id == player_id) {
            Some(queue) => queue,
            None => return Err(XError::not_found(format!("SystemInput::player_events() {}", player_id))),
        };
        return Ok(queue.events(frame, self.input_window));
    }

    pub fn player_enter_event(&self, player_id: NumID) -> XResult<Option<KeyEvent>> {
        let queue = match self.queues.iter().find(|q| q.player_id == player_id) {
            Some(queue) => queue,
            None => {
                return Err(XError::not_found(format!(
                    "SystemInput::player_enter_event() {}",
                    player_id
                )))
            }
        };
        return Ok(queue.enter_event());
    }
}

#[derive(Debug)]
struct FrameMeta {
    offset: usize,   // event counts from events[0] to the first event of this frame
    unconsumed: u64, // the first unconsumed event index in this frame
}

impl FrameMeta {
    fn new(offset: usize, unconsumed: u64) -> FrameMeta {
        FrameMeta { offset, unconsumed }
    }
}

#[derive(Debug)]
struct InputEventQueue {
    player_id: NumID,
    idx_counter: u64,
    events: VecDeque<InputEvent>,
    metas: VecDeque<FrameMeta>,
    current_frame: u32,
    unsynced_frame: u32,
    base_frame: u32,
    unconsumed_idx: u64,
    enter_event: Option<KeyEvent>,
}

impl InputEventQueue {
    fn new(player_id: NumID, input_window: u32) -> XResult<InputEventQueue> {
        if input_window <= 0 {
            return Err(XError::bad_argument("InputEventQueue::new() input_window"));
        }

        let mut iq = InputEventQueue {
            player_id,
            idx_counter: 0,
            events: VecDeque::with_capacity(256),
            metas: VecDeque::with_capacity((2 * FPS as usize) + (input_window as usize)),
            current_frame: 0,
            unsynced_frame: 1,
            base_frame: 1,
            unconsumed_idx: 0,
            enter_event: None,
        };
        iq.metas.push_back(FrameMeta::new(0, 0));
        Ok(iq)
    }

    // The next unproduced future event index.
    #[inline]
    fn future_idx(&self) -> u64 {
        self.idx_counter
    }

    #[inline]
    fn unconsumed_idx(&self) -> u64 {
        self.unconsumed_idx
    }

    #[inline]
    fn enter_event(&self) -> Option<KeyEvent> {
        self.enter_event
    }

    fn produce(&mut self, current_frame: u32, events: &[KeyEvent]) -> XResult<()> {
        if current_frame != self.current_frame + 1 {
            return Err(XError::bad_argument("InputEventQueue::produce() current_frame"));
        }
        self.current_frame += 1;
        for event in events {
            self.events
                .push_back(InputEvent::new(self.idx_counter, self.current_frame, *event));
            self.idx_counter += 1;
        }
        self.metas
            .push_back(FrameMeta::new(self.events.len(), self.unconsumed_idx));
        Ok(())
    }

    fn consume(&mut self, event_idx: u64) -> XResult<()> {
        if event_idx >= self.future_idx() || event_idx < self.unconsumed_idx {
            return Err(XError::overflow("InputEventQueue::consume() event_idx"));
        }
        self.unconsumed_idx = event_idx + 1;
        Ok(())
    }

    fn confirm(&mut self, synced_frame: u32, input_window: u32) -> XResult<()> {
        if synced_frame > self.current_frame {
            return Err(XError::bad_argument("InputEventQueue::confirm() synced_frame"));
        }
        if synced_frame < self.unsynced_frame {
            return Ok(());
        }

        let base_frame = synced_frame.saturating_sub(input_window - 1).max(self.base_frame); // keep one more frame before unsynced_frame
        for _ in self.base_frame..base_frame {
            self.metas.pop_front();
        }
        let discard_count = match self.metas.front() {
            Some(meta) => meta.offset,
            None => return Err(XError::unexpected("InputEventQueue::confirm() self.metas")),
        };
        self.metas.iter_mut().for_each(|meta| meta.offset -= discard_count);

        for _ in 0..discard_count {
            let event = self.events.pop_front();
            assert!(event.is_some());
        }

        self.unsynced_frame = synced_frame + 1;
        self.base_frame = base_frame;
        Ok(())
    }

    fn events(&mut self, frame: u32, input_window: u32) -> InputQueueAgent<'_> {
        let start_frame = frame.saturating_sub(input_window - 1).max(self.base_frame);
        let endx_frame = frame.min(self.current_frame) + 1; // end frame + 1
        if start_frame >= endx_frame {
            return InputQueueAgent {
                queue: self,
                start_pos: 0,
                end_pos: 0,
            };
        }

        let start_pos = match self.metas.get((start_frame - self.base_frame) as usize) {
            Some(meta) => meta.offset,
            None => 0,
        };
        let end_pos = match self.metas.get((endx_frame - self.base_frame) as usize) {
            Some(meta) => meta.offset,
            None => 0,
        };
        InputQueueAgent {
            queue: self,
            start_pos,
            end_pos,
        }
    }

    fn set_enter_event(&mut self, event_idx: u64) -> XResult<()> {
        let idx_offset = match self.events.front() {
            Some(event) => event.idx,
            None => return Err(XError::unexpected("InputEventQueue::set_enter_event() self.events")),
        };
        if event_idx < idx_offset || event_idx >= self.future_idx() {
            return Err(XError::overflow("InputEventQueue::set_enter_event() event_idx"));
        }
        self.enter_event = Some(self.events[(event_idx - idx_offset) as usize].event);
        Ok(())
    }

    fn clear_enter_event(&mut self) {
        self.enter_event = None;
    }
}

#[derive(Debug)]
pub struct InputQueueAgent<'t> {
    queue: &'t mut InputEventQueue,
    start_pos: usize,
    end_pos: usize,
}

impl<'t> InputQueueAgent<'t> {
    pub fn iter(&'t self, start_idx: u64) -> vec_deque::Iter<'t, InputEvent> {
        let mut start_pos = self.start_pos;
        if let Some(event) = self.queue.events.get(start_pos) {
            start_pos += start_idx.max(self.queue.unconsumed_idx()).saturating_sub(event.idx) as usize;
            start_pos = start_pos.min(self.end_pos);
        }
        return self.queue.events.range(start_pos..self.end_pos);
    }

    pub fn future_idx(&self) -> u64 {
        self.queue.future_idx()
    }

    pub fn consume(&mut self, event_idx: u64, enter_event_idx: Option<u64>) -> XResult<()> {
        if let Some(enter_idx) = enter_event_idx {
            self.queue.set_enter_event(enter_idx)?;
        }
        self.queue.consume(event_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: u32 = 3;

    fn collect_offsets(iq: &InputEventQueue) -> Vec<usize> {
        return iq.metas.iter().map(|m| m.offset).collect();
    }

    fn collect_unconsumed(iq: &InputEventQueue) -> Vec<u64> {
        return iq.metas.iter().map(|m| m.unconsumed).collect();
    }

    #[test]
    fn test_input_event_queue_empty() {
        let mut iq: InputEventQueue = InputEventQueue::new(0, 3).unwrap();
        assert_eq!(iq.current_frame, 0);
        assert_eq!(iq.unsynced_frame, 1);
        assert_eq!(iq.base_frame, 1);

        assert!(iq.produce(0, &[]).is_err());
        assert!(iq.produce(5, &[]).is_err());

        assert!(iq.confirm(0, WINDOW).is_ok());
        assert!(iq.confirm(1, WINDOW).is_err());
        assert!(iq.confirm(7, WINDOW).is_err());
        assert_eq!(iq.current_frame, 0);
        assert_eq!(iq.unsynced_frame, 1);
        assert_eq!(iq.base_frame, 1);

        assert_eq!(iq.events(0, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(1, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(100, WINDOW).iter(0).count(), 0);

        iq.produce(1, &[]).unwrap();
        assert_eq!(iq.current_frame, 1);
        assert_eq!(collect_offsets(&iq), vec![0, 0]);
        assert_eq!(collect_unconsumed(&iq), vec![0, 0]);
        assert_eq!(iq.events(0, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(1, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(2, WINDOW).iter(0).count(), 0);

        iq.produce(2, &[]).unwrap();
        iq.produce(3, &[]).unwrap();
        iq.produce(4, &[]).unwrap();
        assert_eq!(iq.current_frame, 4);
        assert_eq!(collect_offsets(&iq), vec![0, 0, 0, 0, 0]);
        assert_eq!(collect_unconsumed(&iq), vec![0, 0, 0, 0, 0]);
        assert_eq!(iq.events(0, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(4, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(6, WINDOW).iter(0).count(), 0);

        assert_eq!(iq.events(0, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(0, WINDOW).iter(1).count(), 0);
        assert_eq!(iq.events(0, WINDOW).iter(10).count(), 0);
    }

    #[test]
    fn test_input_event_queue_produce() {
        let a1_down = KeyEvent::new_button(KeyCode::Attack1, true);
        let a1_up = KeyEvent::new_button(KeyCode::Attack1, false);
        let run_left = KeyEvent::new_motion(KeyCode::Run, Vec2::new(-1.0, 0.0));
        let run_right = KeyEvent::new_motion(KeyCode::Run, Vec2::new(1.0, 0.0));
        let s1_down = KeyEvent::new_button(KeyCode::Skill1, true);
        let s1_up = KeyEvent::new_button(KeyCode::Skill1, false);

        let mut iq = InputEventQueue::new(0, 3).unwrap();

        // produce=1
        iq.produce(1, &[a1_down, a1_up]).unwrap();
        assert_eq!(iq.current_frame, 1);
        assert_eq!(collect_offsets(&iq), vec![0, 2]);
        assert_eq!(iq.events(0, WINDOW).iter(0).count(), 0);
        assert_eq!(
            iq.events(1, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(0, 1, a1_down), &InputEvent::new(1, 1, a1_up),]
        );
        assert_eq!(
            iq.events(2, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(0, 1, a1_down), &InputEvent::new(1, 1, a1_up),]
        );
        assert_eq!(iq.events(2, WINDOW).iter(1).count(), 1);
        assert_eq!(iq.events(2, WINDOW).iter(2).count(), 0);
        assert_eq!(iq.events(4, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(6, WINDOW).iter(0).count(), 0);

        // produce=3
        iq.produce(2, &[]).unwrap();
        iq.produce(3, &[run_left]).unwrap();
        assert_eq!(iq.current_frame, 3);
        assert_eq!(collect_offsets(&iq), vec![0, 2, 2, 3]);
        assert_eq!(iq.events(0, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(1, WINDOW).iter(0).count(), 2);
        assert_eq!(
            iq.events(3, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![
                &InputEvent::new(0, 1, a1_down),
                &InputEvent::new(1, 1, a1_up),
                &InputEvent::new(2, 3, run_left),
            ]
        );
        assert_eq!(iq.events(3, WINDOW).iter(1).count(), 2);
        assert_eq!(iq.events(5, WINDOW).iter(0).count(), 1);

        // produce=6
        iq.produce(4, &[run_right]).unwrap();
        iq.produce(5, &[]).unwrap();
        iq.produce(6, &[s1_down, s1_up]).unwrap();
        assert_eq!(iq.current_frame, 6);
        assert_eq!(collect_offsets(&iq), vec![0, 2, 2, 3, 4, 4, 6]);
        assert_eq!(iq.events(1, WINDOW).iter(0).count(), 2);
        assert_eq!(iq.events(3, WINDOW).iter(0).count(), 3);
        assert_eq!(
            iq.events(6, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![
                &InputEvent::new(3, 4, run_right),
                &InputEvent::new(4, 6, s1_down),
                &InputEvent::new(5, 6, s1_up),
            ]
        );
        assert_eq!(iq.events(6, WINDOW).iter(3).count(), 3);
        assert_eq!(
            iq.events(8, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(4, 6, s1_down), &InputEvent::new(5, 6, s1_up),]
        );
        assert_eq!(iq.events(9, WINDOW).iter(3).count(), 0);
    }

    #[test]
    fn test_input_event_queue_confirm() {
        let b1_down = KeyEvent::new_button(KeyCode::Skill1, true);
        let b1_up = KeyEvent::new_button(KeyCode::Skill1, false);
        let run_left = KeyEvent::new_motion(KeyCode::Run, Vec2::new(-1.0, 0.0));
        let run_right = KeyEvent::new_motion(KeyCode::Run, Vec2::new(1.0, 0.0));
        let aim = KeyEvent::new_button(KeyCode::Aim, true);
        let shoot = KeyEvent::new_button(KeyCode::Shot1, true);

        let mut iq = InputEventQueue::new(0, 3).unwrap();

        // produce=0
        iq.produce(1, &[b1_down, b1_up]).unwrap();
        iq.confirm(0, WINDOW).unwrap();
        assert_eq!(iq.current_frame, 1);
        assert_eq!(iq.unsynced_frame, 1);
        assert_eq!(iq.base_frame, 1);
        assert_eq!(collect_offsets(&iq), vec![0, 2]);
        assert_eq!(iq.events(0, WINDOW).iter(0).count(), 0);
        assert_eq!(
            iq.events(1, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(0, 1, b1_down), &InputEvent::new(1, 1, b1_up),]
        );
        assert_eq!(
            iq.events(2, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(0, 1, b1_down), &InputEvent::new(1, 1, b1_up),]
        );
        assert_eq!(iq.events(4, WINDOW).iter(0).count(), 0);
        assert_eq!(iq.events(5, WINDOW).iter(0).count(), 0);

        // produce=3
        iq.produce(2, &[run_left]).unwrap();
        iq.produce(3, &[run_right, aim]).unwrap();
        iq.confirm(3, WINDOW).unwrap();
        assert_eq!(iq.current_frame, 3);
        assert_eq!(iq.unsynced_frame, 4);
        assert_eq!(iq.base_frame, 1);
        assert_eq!(collect_offsets(&iq), vec![0, 2, 3, 5]);
        assert_eq!(iq.events(3, WINDOW).iter(0).count(), 5);

        // produce=4
        iq.produce(4, &[]).unwrap();
        iq.confirm(4, WINDOW).unwrap();
        assert_eq!(iq.current_frame, 4);
        assert_eq!(iq.unsynced_frame, 5);
        assert_eq!(iq.base_frame, 2);
        assert_eq!(collect_offsets(&iq), vec![0, 1, 3, 3]);
        assert_eq!(
            iq.events(4, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![
                &InputEvent::new(2, 2, run_left),
                &InputEvent::new(3, 3, run_right),
                &InputEvent::new(4, 3, aim),
            ]
        );
        assert_eq!(iq.events(5, WINDOW).iter(0).count(), 2);

        // produce=6
        iq.produce(5, &[shoot]).unwrap();
        iq.produce(6, &[run_left, run_right]).unwrap();
        iq.confirm(5, WINDOW).unwrap();
        assert_eq!(iq.current_frame, 6);
        assert_eq!(iq.unsynced_frame, 6);
        assert_eq!(iq.base_frame, 3);
        assert_eq!(collect_offsets(&iq), vec![0, 2, 2, 3, 5]);
        assert_eq!(
            iq.events(5, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![
                &InputEvent::new(3, 3, run_right),
                &InputEvent::new(4, 3, aim),
                &InputEvent::new(5, 5, shoot),
            ]
        );
        assert_eq!(
            iq.events(6, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![
                &InputEvent::new(5, 5, shoot),
                &InputEvent::new(6, 6, run_left),
                &InputEvent::new(7, 6, run_right),
            ]
        );
        assert_eq!(iq.events(7, WINDOW).iter(0).count(), 3);
        assert_eq!(iq.events(9, WINDOW).iter(0).count(), 0);
    }

    #[test]
    fn test_input_event_queue_consume() {
        let b1_down = KeyEvent::new_button(KeyCode::Skill1, true);
        let b1_up = KeyEvent::new_button(KeyCode::Skill1, false);

        let mut iq = InputEventQueue::new(0, 3).unwrap();

        iq.produce(1, &[b1_down, b1_down, b1_up]).unwrap();
        assert!(iq.consume(1).is_ok());
        assert!(iq.consume(3).is_err());
        iq.confirm(0, WINDOW).unwrap();
        assert_eq!(iq.unconsumed_idx(), 2);
        assert_eq!(collect_unconsumed(&iq), vec![0, 0]);
        assert_eq!(
            iq.events(1, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(2, 1, b1_up),]
        );

        iq.produce(2, &[b1_down, b1_down]).unwrap();
        assert_eq!(collect_unconsumed(&iq), vec![0, 0, 2]);
        assert_eq!(
            iq.events(1, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(2, 1, b1_up),]
        );
        assert_eq!(
            iq.events(2, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![
                &InputEvent::new(2, 1, b1_up),
                &InputEvent::new(3, 2, b1_down),
                &InputEvent::new(4, 2, b1_down),
            ]
        );
        iq.consume(2).unwrap();

        iq.produce(3, &[]).unwrap();
        iq.confirm(3, WINDOW).unwrap();
        iq.produce(4, &[]).unwrap();
        iq.produce(5, &[]).unwrap();
        assert_eq!(collect_unconsumed(&iq), vec![0, 0, 2, 3, 3, 3]);
        assert_eq!(
            iq.events(4, WINDOW).iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(3, 2, b1_down), &InputEvent::new(4, 2, b1_down)]
        );
    }

    #[test]
    fn test_input_system() {
        let a1_down = KeyEvent::new_button(KeyCode::Attack1, true);
        let a1_up = KeyEvent::new_button(KeyCode::Attack1, false);
        let b1_down = KeyEvent::new_button(KeyCode::Skill1, true);
        let b1_up = KeyEvent::new_button(KeyCode::Skill1, false);
        let run_left = KeyEvent::new_motion(KeyCode::Run, Vec2::new(-1.0, 0.0));
        let run_right = KeyEvent::new_motion(KeyCode::Run, Vec2::new(1.0, 0.0));

        assert!(SystemInput::new(0).is_err());
        let mut si = SystemInput::new(3).unwrap();
        // assert!(si.init(&[]).is_err()); // forbid empty
        assert!(si.init(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).is_err());
        assert!(si.init(&[10, 20, 10]).is_err());
        si.init(&[100, 200]).unwrap();
        assert!(si.init(&[300]).is_err());
        assert_eq!(si.player_count(), 2);
        assert_eq!(si.input_window(), 3);

        assert_eq!(si.player_events(100, 0).unwrap().iter(0).count(), 0);
        assert_eq!(si.player_events(200, 0).unwrap().iter(0).count(), 0);

        // p1=1, p2=?
        let events = vec![PlayerKeyEvents {
            player_id: 100,
            frame: 1,
            events: vec![a1_down, a1_up],
        }];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.latest_frame(), 1);
        assert_eq!(si.synced_frame(), 0);
        assert_eq!(si.unsynced_frame(), 1);
        assert_eq!(
            si.player_events(100, 1).unwrap().iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(0, 1, a1_down), &InputEvent::new(1, 1, a1_up),]
        );
        assert_eq!(si.player_events(200, 0).unwrap().iter(0).count(), 0);

        // p1=1, p2=1
        let events = vec![PlayerKeyEvents {
            player_id: 200,
            frame: 1,
            events: vec![b1_down],
        }];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.latest_frame(), 1);
        assert_eq!(si.synced_frame(), 1);
        assert_eq!(si.unsynced_frame(), 2);
        assert_eq!(si.player_events(100, 1).unwrap().iter(0).count(), 2);
        assert_eq!(
            si.player_events(200, 1).unwrap().iter(0).collect::<Vec<_>>(),
            vec![&InputEvent::new(0, 1, b1_down),]
        );

        // p1=3, p2=1
        let events = vec![
            PlayerKeyEvents {
                player_id: 100,
                frame: 2,
                events: vec![],
            },
            PlayerKeyEvents {
                player_id: 100,
                frame: 3,
                events: vec![run_left, run_right],
            },
        ];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.latest_frame(), 3);
        assert_eq!(si.synced_frame(), 1);
        assert_eq!(si.unsynced_frame(), 2);
        assert_eq!(si.player_events(100, 3).unwrap().iter(0).count(), 4);
        assert_eq!(si.player_events(200, 3).unwrap().iter(0).count(), 1);

        // p1=3, p2=2
        let events = vec![PlayerKeyEvents {
            player_id: 200,
            frame: 2,
            events: vec![b1_up],
        }];
        si.produce(&events).unwrap();
        si.confirm().unwrap();
        assert_eq!(si.latest_frame(), 3);
        assert_eq!(si.synced_frame(), 2);
        assert_eq!(si.unsynced_frame(), 3);
        assert_eq!(si.player_events(100, 3).unwrap().iter(0).count(), 4);
        assert_eq!(si.player_events(200, 3).unwrap().iter(0).count(), 2);
    }
}
