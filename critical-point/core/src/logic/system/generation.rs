use std::collections::VecDeque;

use crate::consts::FPS;
use crate::utils::NumID;

pub struct SystemGeneration {
    history: VecDeque<(u32, NumID)>,
    counter: NumID,
}

impl SystemGeneration {
    pub fn new(init: NumID) -> SystemGeneration {
        SystemGeneration {
            history: VecDeque::with_capacity(2 * FPS as usize),
            counter: init,
        }
    }

    pub fn counter(&self) -> NumID {
        self.counter
    }

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
}
