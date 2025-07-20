use std::collections::VecDeque;

use crate::consts::FPS_USIZE;
use crate::utils::{xres, NumID, XResult, MAX_PLAYER_ID, MIN_PLAYER_ID};

const MIN_AUTO_GEN_ID: NumID = 1000;

pub struct SystemGeneration {
    history: VecDeque<(u32, NumID)>,
    player: NumID,
    counter: NumID,
}

impl SystemGeneration {
    pub fn new() -> SystemGeneration {
        SystemGeneration {
            history: VecDeque::with_capacity(2 * FPS_USIZE),
            player: MIN_PLAYER_ID,
            counter: MIN_AUTO_GEN_ID,
        }
    }

    #[inline]
    pub fn gen_player_id(&mut self) -> XResult<NumID> {
        if self.player > MAX_PLAYER_ID {
            return xres!(Overflow);
        }
        let id = self.player;
        self.player += 1;
        Ok(id)
    }

    #[inline]
    pub fn gen_id(&mut self) -> NumID {
        let id = self.counter;
        self.counter += 1;
        id
    }

    pub fn update(&mut self, frame: u32) {
        self.history.push_back((frame, self.counter));
    }

    pub fn restore(&mut self, frame: u32) {
        while let Some((fr, id)) = self.history.back() {
            if *fr > frame {
                self.history.pop_back();
            } else {
                self.counter = *id;
                break;
            }
        }
    }

    pub fn discard(&mut self, frame: u32) {
        while let Some((fr, _)) = self.history.front() {
            if *fr <= frame {
                self.history.pop_front();
            } else {
                break;
            }
        }
    }

    #[inline]
    pub fn counter(&self) -> NumID {
        self.counter
    }
}
