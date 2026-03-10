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
    pub player: NumID,
    pub auto_gen: NumID,
}

#[derive(Debug)]
pub(crate) struct SystemGeneration {
    history: VecDeque<(u32, NumID)>,
    player: NumID,
    auto_gen: NumID,
}

impl SystemGeneration {
    pub(crate) fn new() -> SystemGeneration {
        SystemGeneration {
            history: VecDeque::with_capacity(2 * FPS_USIZE),
            player: NumID::MIN_PLAYER,
            auto_gen: NumID::MIN_AUTO_GEN,
        }
    }

    #[inline]
    pub(crate) fn gen_player_id(&mut self) -> XResult<NumID> {
        if self.player > NumID::MAX_PLAYER {
            return xres!(Overflow);
        }
        let id = self.player;
        self.player = self.player + 1;
        Ok(id)
    }

    #[inline]
    pub(crate) fn gen_num_id(&mut self) -> NumID {
        let id = self.auto_gen;
        self.auto_gen = self.auto_gen + 1;
        id
    }

    pub(crate) fn update(&mut self, frame: u32) {
        self.history.push_back((frame, self.auto_gen));
    }

    pub(crate) fn restore(&mut self, frame: u32) {
        while let Some((fr, id)) = self.history.back() {
            if *fr > frame {
                self.history.pop_back();
            }
            else {
                self.auto_gen = *id;
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
            player: self.player,
            auto_gen: self.auto_gen,
        }
    }
}
