use std::collections::VecDeque;

use critical_point_csgen::CsOut;

use crate::consts::FPS_USIZE;
use crate::utils::{xres, NumID, XResult};

#[repr(C)]
#[derive(
    Debug,
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
    CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Value)]
pub struct StateGeneration {
    pub player_id: NumID,
    pub auto_gen_id: NumID,
    pub action_id: u32,
}

#[derive(Debug)]
pub(crate) struct SystemGeneration {
    history: VecDeque<(u32, StateGeneration)>,
    player_id: NumID,
    auto_gen_id: NumID,
    action_id: u32,
}

impl SystemGeneration {
    pub(crate) fn new() -> SystemGeneration {
        SystemGeneration {
            history: VecDeque::with_capacity(2 * FPS_USIZE),
            player_id: NumID::MIN_PLAYER,
            auto_gen_id: NumID::MIN_AUTO_GEN,
            action_id: 0,
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

    pub(crate) fn update(&mut self, frame: u32) {
        self.history.push_back((frame, StateGeneration {
            player_id: self.player_id,
            auto_gen_id: self.auto_gen_id,
            action_id: self.action_id,
        }));
    }

    pub(crate) fn restore(&mut self, frame: u32) {
        while let Some((fr, state)) = self.history.back() {
            if *fr > frame {
                self.history.pop_back();
            }
            else {
                self.player_id = state.player_id;
                self.auto_gen_id = state.auto_gen_id;
                self.action_id = state.action_id;
                break;
            }
        }
    }

    pub(crate) fn discard(&mut self, frame: u32) {
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
        }
    }
}
