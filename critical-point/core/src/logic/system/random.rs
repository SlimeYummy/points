use oorandom::{Rand32, Rand64};
use std::collections::VecDeque;

use crate::consts::FPS_USIZE;

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
#[rkyv(derive(Debug))]
pub struct StateRandom {
    pub state32: (u64, u64),
    pub state64: (u128, u128),
}

#[derive(Debug, Clone)]
pub(crate) struct SystemRandom {
    history: VecDeque<(u32, StateRandom)>,
    rand32: Rand32,
    rand64: Rand64,
}

impl SystemRandom {
    pub(crate) fn new(seed32: u64, seed64: u128) -> SystemRandom {
        SystemRandom {
            history: VecDeque::with_capacity(2 * FPS_USIZE),
            rand32: Rand32::new(seed32),
            rand64: Rand64::new(seed64),
        }
    }

    #[inline]
    pub(crate) fn rand_u32(&mut self) -> u32 {
        self.rand32.rand_u32()
    }

    #[inline]
    pub(crate) fn rand_f32(&mut self) -> f32 {
        self.rand32.rand_float()
    }

    #[inline]
    pub(crate) fn rand_u64(&mut self) -> u64 {
        self.rand64.rand_u64()
    }

    #[inline]
    pub(crate) fn rand_f64(&mut self) -> f64 {
        self.rand64.rand_float()
    }

    pub(crate) fn update(&mut self, frame: u32) {
        let last_frame = self.history.back().map_or(0, |(fr, _)| *fr);
        debug_assert!(last_frame + 1 == frame || frame == 0);

        self.history.push_back((frame, StateRandom {
            state32: self.rand32.state(),
            state64: self.rand64.state(),
        }));
    }

    pub(crate) fn restore(&mut self, frame: u32) {
        let last_frame = self.history.back().map_or(0, |(fr, _)| *fr);
        println!("last_frame: {} frame: {}", last_frame, frame);
        debug_assert!(last_frame >= frame);
        let first_frame = self.history.front().map_or(0, |(fr, _)| *fr);
        debug_assert!(first_frame <= frame);

        while let Some((fr, state)) = self.history.back() {
            if *fr > frame {
                self.history.pop_back();
            }
            else {
                self.rand32 = Rand32::from_state(state.state32);
                self.rand64 = Rand64::from_state(state.state64);
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
    pub(crate) fn state(&self) -> StateRandom {
        StateRandom {
            state32: self.rand32.state(),
            state64: self.rand64.state(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_random_state() {
        let mut sys_rand = SystemRandom::new(12345, 67890);

        sys_rand.update(1); // [1]
        let f1_32 = sys_rand.rand_f32();
        let f1_64 = sys_rand.rand_f64();

        sys_rand.update(2); // [1, 2]
        let f2_32 = sys_rand.rand_f32();
        let f2_64 = sys_rand.rand_f64();

        sys_rand.restore(1); // [1]
        assert_eq!(sys_rand.rand_f32(), f1_32);
        assert_eq!(sys_rand.rand_f64(), f1_64);

        sys_rand.restore(1); // in => [1]
        assert_eq!(sys_rand.rand_f32(), f1_32);
        assert_eq!(sys_rand.rand_f64(), f1_64);

        sys_rand.update(2); // [1, 2]
        assert_eq!(sys_rand.rand_f32(), f2_32);
        assert_eq!(sys_rand.rand_f64(), f2_64);

        sys_rand.update(3); // [1, 2, 3]
        let u3_32 = sys_rand.rand_u32();
        let u3_64 = sys_rand.rand_u64();

        sys_rand.discard(2); // [3]
        assert_eq!(sys_rand.history.len(), 1);

        sys_rand.restore(3); // [3]
        assert_eq!(sys_rand.rand_u32(), u3_32);
        assert_eq!(sys_rand.rand_u64(), u3_64);

        sys_rand.update(4); // [3, 4]
        let u4_32 = sys_rand.rand_u32();
        sys_rand.update(5); // [3, 4, 5]
        let u5_32 = sys_rand.rand_u32();

        sys_rand.discard(3); // [4, 5]
        assert_eq!(sys_rand.history.len(), 2);

        sys_rand.restore(5); // [4, 5]
        assert_eq!(sys_rand.rand_u32(), u5_32);

        sys_rand.restore(4); // [4]
        assert_eq!(sys_rand.rand_u32(), u4_32);
    }
}
