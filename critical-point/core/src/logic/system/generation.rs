use std::collections::VecDeque;

use crate::consts::FPS_USIZE;
use crate::utils::{NumID, XResult, xres};

#[repr(C)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug, Default))]
pub struct StateGeneration {
    pub player_id: NumID,
    pub auto_gen_id: NumID,
    pub action_id: u32,
    pub ai_task_id: u32,
}

#[derive(Debug)]
pub(crate) struct SystemGeneration {
    history: VecDeque<(u32, StateGeneration)>,
    player_id: NumID,
    auto_gen_id: NumID,
    action_id: u32,
    ai_task_id: u32,
}

impl SystemGeneration {
    pub(crate) fn new() -> SystemGeneration {
        SystemGeneration {
            history: VecDeque::with_capacity(2 * FPS_USIZE),
            player_id: NumID::MIN_PLAYER,
            auto_gen_id: NumID::MIN_AUTO_GEN,
            action_id: 0,
            ai_task_id: 0,
        }
    }

    #[inline]
    pub(crate) fn gen_player_id(&mut self) -> XResult<NumID> {
        if self.player_id > NumID::MAX_PLAYER {
            return xres!(Overflow);
        }
        let id = self.player_id;
        self.player_id = self.player_id + 1;
        Ok(id)
    }

    #[inline]
    pub(crate) fn gen_num_id(&mut self) -> NumID {
        let id = self.auto_gen_id;
        self.auto_gen_id = self.auto_gen_id + 1;
        id
    }

    #[inline]
    pub(crate) fn gen_action_id(&mut self) -> u32 {
        let id = self.action_id;
        self.action_id = self.action_id + 1;
        id
    }

    #[inline]
    pub(crate) fn gen_ai_task_id(&mut self) -> u32 {
        let id = self.ai_task_id;
        self.ai_task_id = self.ai_task_id + 1;
        id
    }

    pub(crate) fn update(&mut self, frame: u32) {
        let last_frame = self.history.back().map_or(0, |(fr, _)| *fr);
        debug_assert!(last_frame + 1 == frame || frame == 0);

        self.history.push_back((frame, StateGeneration {
            player_id: self.player_id,
            auto_gen_id: self.auto_gen_id,
            action_id: self.action_id,
            ai_task_id: self.ai_task_id,
        }));
    }

    pub(crate) fn restore(&mut self, frame: u32) {
        let last_frame = self.history.back().map_or(0, |(fr, _)| *fr);
        debug_assert!(last_frame >= frame);
        let first_frame = self.history.front().map_or(0, |(fr, _)| *fr);
        debug_assert!(first_frame <= frame);

        while let Some((fr, state)) = self.history.back() {
            if *fr > frame {
                self.history.pop_back();
            }
            else {
                self.player_id = state.player_id;
                self.auto_gen_id = state.auto_gen_id;
                self.action_id = state.action_id;
                self.ai_task_id = state.ai_task_id;
                break;
            }
        }
    }

    pub(crate) fn discard(&mut self, frame: u32) {
        let first_frame = self.history.front().map_or(0, |(fr, _)| *fr);
        debug_assert!(first_frame <= frame);
        let last_frame = self.history.back().map_or(0, |(fr, _)| *fr);
        debug_assert!(last_frame > frame);

        while let Some((fr, _)) = self.history.front() {
            if *fr <= frame {
                self.history.pop_front();
            }
            else {
                break;
            }
        }
    }

    #[inline]
    pub(crate) fn state(&self) -> StateGeneration {
        StateGeneration {
            player_id: self.player_id,
            auto_gen_id: self.auto_gen_id,
            action_id: self.action_id,
            ai_task_id: self.ai_task_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_generation_state() {
        let mut sys_gen = SystemGeneration::new();

        sys_gen.update(1); // [1]
        let id1_action = sys_gen.gen_action_id();
        let id1_ai = sys_gen.gen_ai_task_id();

        sys_gen.update(2); // [1, 2]
        let id2_action = sys_gen.gen_action_id();
        let id2_ai = sys_gen.gen_ai_task_id();

        sys_gen.restore(1); // [1]
        assert_eq!(sys_gen.gen_action_id(), id1_action);
        assert_eq!(sys_gen.gen_ai_task_id(), id1_ai);

        sys_gen.restore(1); // [1]
        assert_eq!(sys_gen.gen_action_id(), id1_action);
        assert_eq!(sys_gen.gen_ai_task_id(), id1_ai);

        sys_gen.update(2); // [1, 2]
        assert_eq!(sys_gen.gen_action_id(), id2_action);
        assert_eq!(sys_gen.gen_ai_task_id(), id2_ai);

        sys_gen.update(3); // [1, 2, 3]
        let id3_action = sys_gen.gen_action_id();
        let id3_num = sys_gen.gen_num_id();

        sys_gen.discard(2); // [3]
        assert_eq!(sys_gen.history.len(), 1);

        sys_gen.restore(3); // [3]
        assert_eq!(sys_gen.gen_action_id(), id3_action);
        assert_eq!(sys_gen.gen_num_id(), id3_num);

        sys_gen.update(4); // [3, 4]
        let id4_action = sys_gen.gen_action_id();
        sys_gen.update(5); // [3, 4, 5]
        let id5_action = sys_gen.gen_action_id();

        sys_gen.discard(3); // [4, 5]
        assert_eq!(sys_gen.history.len(), 2);

        sys_gen.restore(5); // [4, 5]
        assert_eq!(sys_gen.gen_action_id(), id5_action);

        sys_gen.restore(4); // [4]
        assert_eq!(sys_gen.gen_action_id(), id4_action);
    }
}
