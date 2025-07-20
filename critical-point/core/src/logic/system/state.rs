use cirtical_point_csgen::CsOut;
use log::debug;
use std::collections::{vec_deque, VecDeque};
use std::ops::{Index, RangeBounds};
use std::sync::Arc;

use super::super::base::StateAny;
use crate::consts::FPS_USIZE;
use crate::utils::{xres, xresf, XResult};

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[cs_attr(Ref)]
pub struct StateSet {
    pub frame: u32,
    pub inits: Vec<Arc<dyn StateAny>>,
    pub updates: Vec<Box<dyn StateAny>>,
}

#[cfg(feature = "debug-print")]
impl Drop for StateSet {
    fn drop(&mut self) {
        debug!("StateSet::drop() frame={}", self.frame);
    }
}

impl StateSet {
    #[inline]
    pub fn new(frame: u32, new_cap: usize, update_cap: usize) -> StateSet {
        StateSet {
            frame,
            inits: Vec::with_capacity(new_cap),
            updates: Vec::with_capacity(update_cap),
        }
    }
}

#[derive(Debug)]
pub struct SystemState {
    state_sets: VecDeque<Arc<StateSet>>,
    current_frame: u32,
    synced_frame: u32,
}

impl Default for SystemState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SystemState {
    #[inline]
    pub fn new() -> SystemState {
        SystemState {
            state_sets: VecDeque::with_capacity(2 * FPS_USIZE),
            current_frame: 0,
            synced_frame: 0,
        }
    }

    #[inline]
    pub fn current_frame(&self) -> u32 {
        self.current_frame
    }

    #[inline]
    pub fn synced_frame(&self) -> u32 {
        self.synced_frame
    }

    #[inline]
    pub fn unsynced_frame(&self) -> u32 {
        self.synced_frame + 1
    }

    pub fn init(&mut self, state_set: Arc<StateSet>) -> XResult<()> {
        if state_set.frame != 0 {
            return xresf!(BadArgument; "state_set.frame={}", state_set.frame);
        }
        self.state_sets.push_back(state_set);
        Ok(())
    }

    pub fn append(&mut self, state_set: Arc<StateSet>) -> XResult<()> {
        if state_set.frame != self.current_frame + 1 {
            return xresf!(BadArgument; "state_set.frame={}, current_frame={}", state_set.frame, self.current_frame);
        }
        self.current_frame += 1;
        self.state_sets.push_back(state_set);
        Ok(())
    }

    pub fn confirm(&mut self, synced_frame: u32) -> XResult<Vec<Arc<StateSet>>> {
        let mut outs = vec![];
        if synced_frame > self.current_frame {
            return xresf!(BadArgument; "synced_frame={}, current_frame={}", synced_frame, self.current_frame);
        }
        if synced_frame <= self.synced_frame {
            return Ok(outs);
        }

        while let Some(state) = self.state_sets.front() {
            if state.frame < synced_frame {
                let state_set = self.state_sets.pop_front();
                assert!(state_set.is_some());
                if let Some(state_set) = state_set {
                    outs.push(state_set);
                }
            } else {
                break;
            }
        }
        self.synced_frame = synced_frame;
        Ok(outs)
    }

    pub fn restore(&mut self, frame: u32) -> XResult<()> {
        if frame < self.synced_frame || frame > self.current_frame {
            return xresf!(BadArgument; "frame={}, synced_frame={}, current_frame={}", frame, self.synced_frame, self.current_frame);
        }
        while let Some(state) = self.state_sets.back() {
            if state.frame > frame {
                self.state_sets.pop_front();
            } else {
                break;
            }
        }
        self.current_frame = frame;
        Ok(())
    }

    #[inline]
    pub fn get(&self, frame: u32) -> Option<&Arc<StateSet>> {
        if frame < self.synced_frame || frame > self.current_frame {
            return None;
        }
        return self.state_sets.get((frame - self.synced_frame) as usize);
    }

    pub fn range<R>(&self, frame_range: R) -> XResult<vec_deque::Iter<'_, Arc<StateSet>>>
    where
        R: RangeBounds<u32>,
    {
        let start = match frame_range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => self.synced_frame,
        };
        if start < self.synced_frame {
            return xres!(BadArgument; "range start");
        }
        let start_pos = (start - self.synced_frame) as usize;

        let end = match frame_range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => self.current_frame + 1,
        };
        let end_pos = (end - self.synced_frame) as usize;
        if end_pos > self.state_sets.len() {
            return xres!(BadArgument; "range end");
        }

        return Ok(self.state_sets.range(start_pos..end_pos));
    }
}

impl Index<u32> for SystemState {
    type Output = Arc<StateSet>;

    #[inline]
    fn index(&self, frame: u32) -> &Arc<StateSet> {
        return self.get(frame).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager() {
        let mut ss = SystemState::new();
        assert_eq!(ss.current_frame(), 0);
        assert_eq!(ss.synced_frame(), 0);
        assert_eq!(ss.unsynced_frame(), 1);

        assert!(ss.init(Arc::new(StateSet::new(1, 0, 0))).is_err());
        ss.init(Arc::new(StateSet::new(0, 0, 0))).unwrap();
        assert_eq!(ss.current_frame(), 0);
        assert_eq!(ss.synced_frame(), 0);
        assert_eq!(ss.unsynced_frame(), 1);
        assert_eq!(ss.state_sets.len(), 1);
        assert_eq!(ss.range(0..0).unwrap().count(), 0);
        assert_eq!(ss.range(0..1).unwrap().count(), 1);

        assert!(ss.confirm(2).is_err());
        assert!(ss.restore(2).is_err());

        // frame=1
        ss.append(Arc::new(StateSet::new(1, 0, 0))).unwrap();
        assert_eq!(ss.current_frame(), 1);
        assert_eq!(ss.synced_frame(), 0);
        assert_eq!(ss.unsynced_frame(), 1);
        assert_eq!(ss.state_sets.len(), 2);
        assert_eq!(ss.range(0..=1).unwrap().count(), 2);

        ss.confirm(1).unwrap();
        assert_eq!(ss.current_frame(), 1);
        assert_eq!(ss.synced_frame(), 1);
        assert_eq!(ss.unsynced_frame(), 2);
        assert_eq!(ss.state_sets.len(), 1);
        assert_eq!(ss.range(1..=1).unwrap().count(), 1);

        ss.restore(1).unwrap();
        assert_eq!(ss.current_frame(), 1);
        assert_eq!(ss.synced_frame(), 1);
        assert_eq!(ss.unsynced_frame(), 2);
        assert_eq!(ss.state_sets.len(), 1);
        assert_eq!(ss.range(1..=1).unwrap().count(), 1);

        //frame=4
        ss.append(Arc::new(StateSet::new(2, 0, 0))).unwrap();
        ss.append(Arc::new(StateSet::new(3, 0, 0))).unwrap();
        ss.append(Arc::new(StateSet::new(4, 0, 0))).unwrap();
        assert_eq!(ss.current_frame(), 4);
        assert_eq!(ss.synced_frame(), 1);
        assert_eq!(ss.unsynced_frame(), 2);
        assert_eq!(ss.state_sets.len(), 4);
        assert_eq!(ss.range(1..=4).unwrap().count(), 4);

        ss.confirm(3).unwrap();
        assert_eq!(ss.current_frame(), 4);
        assert_eq!(ss.synced_frame(), 3);
        assert_eq!(ss.unsynced_frame(), 4);
        assert_eq!(ss.state_sets.len(), 2);
        assert_eq!(ss.range(3..=4).unwrap().count(), 2);

        assert!(ss.confirm(5).is_err());
    }
}
