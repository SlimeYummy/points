use cirtical_point_csgen::CsOut;
use glam::Mat4;
use ozz_animation_rs::{
    ozz_rc_buf, Animation, BlendingJob, BlendingLayer, LocalToModelJob, SamplingContext, SamplingJob, Skeleton,
    SoaTransform,
};
use std::cell::{Ref, RefCell};
use std::mem;
use std::rc::Rc;

use crate::consts::MAX_ACTION_ANIMATION;
use crate::logic::StateAction;
use crate::utils::{HistoryQueue, NumID, StrID, Symbol, XError, XResult};

#[repr(C)]
#[derive(Debug, Default, Clone, CsOut)]
#[cs_attr(Ref)]
pub struct SkeletonMeta {
    pub num_joints: u32,
    pub num_soa_joints: u32,
    pub joint_metas: Vec<SkeletonJointMeta>,
}

#[repr(C)]
#[derive(Debug, Default, Clone, CsOut)]
#[cs_attr(Ref)]
pub struct SkeletonJointMeta {
    pub index: i16,
    pub parent: i16,
    pub name: String,
}

#[cfg(debug_assertions)]
impl Drop for SkeletonMeta {
    fn drop(&mut self) {
        println!("SkeletonMeta.drop()");
    }
}

#[derive(Debug)]
pub struct SkeletalAnimator {
    skeleton: Rc<Skeleton>,
    blending_job: BlendingJob,
    l2m_job: Option<LocalToModelJob>,

    action_queue: HistoryQueue<ActionData>,
    sampling_arena: SamplingArena,
}

impl SkeletalAnimator {
    pub fn new(skeleton: Rc<Skeleton>, skip_l2m: bool, action_cap: usize, sampling_cap: usize) -> SkeletalAnimator {
        let mut sa: SkeletalAnimator = SkeletalAnimator {
            skeleton: skeleton.clone(),
            blending_job: BlendingJob::default(),
            l2m_job: None,

            action_queue: HistoryQueue::with_capacity(action_cap),
            sampling_arena: SamplingArena::new(sampling_cap),
        };

        sa.blending_job.set_skeleton(sa.skeleton.clone());
        sa.blending_job
            .set_output(ozz_rc_buf(vec![SoaTransform::default(); sa.skeleton.num_soa_joints()]));

        if !skip_l2m {
            let mut l2m_job = LocalToModelJob::default();
            l2m_job.set_skeleton(sa.skeleton.clone());
            l2m_job.set_input(sa.blending_job.output().unwrap().clone());
            l2m_job.set_output(ozz_rc_buf(vec![Mat4::default(); sa.skeleton.num_joints()]));
            sa.l2m_job = Some(l2m_job);
        }
        sa
    }

    pub fn update<F>(&mut self, frame: u32, states: &[Box<dyn StateAction>], mut load: F) -> XResult<()>
    where
        F: FnMut(&Symbol) -> XResult<Rc<Animation>>,
    {
        if states.is_empty() {
            return Err(XError::unexpected("SkeletalAnimator::update() action states len"));
        }

        // 1. dequeue unused actions
        self.action_queue.dequeue(|ad| states[0].id != ad.id);

        // 2. verify using actions
        if self.action_queue.len() > states.len() {
            return Err(XError::unexpected("SkeletalAnimator::update() action states len"));
        }
        for (idx, state) in states.iter().enumerate() {
            let ad = match self.action_queue.get_mut(idx) {
                Some(ad) => ad,
                None => break,
            };
            if state.id != ad.id {
                return Err(XError::unexpected("SkeletalAnimator::update() action states order"));
            }
            ad.update(&mut self.sampling_arena, frame, state, &self.skeleton, &mut load)?;
        }

        // 3. try reuse actions
        for idx in self.action_queue.len()..states.len() {
            let state = &states[idx];
            let reused = self
                .action_queue
                .enqueue_reuse(|ad| {
                    if ad.tmpl_id == state.tmpl_id {
                        ad.reuse(&mut self.sampling_arena, frame, state, &self.skeleton, &mut load)?;
                    }
                    Ok(ad.tmpl_id == state.tmpl_id)
                })?
                .is_some();
            if !reused {
                break;
            }
        }

        // 4. enqueue new actions
        for idx in self.action_queue.len()..states.len() {
            let state = &states[idx];
            let mut ad = ActionData::default();
            ad.init(&mut self.sampling_arena, frame, state, &self.skeleton, &mut load)?;
            self.action_queue.enqueue_new(ad);
        }
        Ok(())
    }

    pub fn restore(&mut self, frame: u32, states: &[Box<dyn StateAction>]) -> XResult<()> {
        if states.is_empty() {
            return Err(XError::unexpected("SkeletalAnimator::restore() action states len"));
        }

        let mut state_iter = states.iter();
        self.action_queue.restore_when(|ad| {
            if ad.frame < frame {
                return Ok(-1);
            }
            if let Some(state) = state_iter.next() {
                if ad.id == state.id {
                    ad.restore(&mut self.sampling_arena, frame, state)?;
                    return Ok(0);
                } else {
                    return Err(XError::unexpected("SkeletalAnimator::restore() action states order"));
                }
            }
            Ok(1)
        })?;
        if state_iter.next().is_some() {
            return Err(XError::unexpected("SkeletalAnimator::restore() action states order"));
        }
        Ok(())
    }

    pub fn discard(&mut self, frame: u32) {
        self.action_queue.discard(|ad| {
            if ad.frame <= frame {
                ad.discard_all_animations(&mut self.sampling_arena);
            }
            ad.frame <= frame
        });

        for ad in self.action_queue.iter_mut() {
            ad.discard_animations_by_frame(&mut self.sampling_arena, frame);
        }
    }

    pub fn animate(&mut self) -> XResult<()> {
        for ad in self.action_queue.iter_mut() {
            ad.animate(&mut self.sampling_arena, &mut self.blending_job)?;
        }
        self.blending_job.run()?;
        if let Some(ref mut l2m_job) = self.l2m_job {
            l2m_job.run()?;
        }
        Ok(())
    }

    pub fn skeleton_meta(&self) -> SkeletonMeta {
        let mut joint_metas = vec![SkeletonJointMeta::default(); self.skeleton.num_joints() as usize];
        for (name, index) in self.skeleton.joint_names() {
            joint_metas[*index as usize] = SkeletonJointMeta {
                index: *index as i16,
                parent: self.skeleton.joint_parent(*index),
                name: name.clone(),
            };
        }
        SkeletonMeta {
            num_joints: self.skeleton.num_joints() as u32,
            num_soa_joints: self.skeleton.num_soa_joints() as u32,
            joint_metas,
        }
    }

    #[inline]
    pub fn joint_rest_poses(&self) -> &[SoaTransform] {
        self.skeleton.joint_rest_poses()
    }

    #[inline]
    pub fn local_out_buf(&self) -> Rc<RefCell<Vec<SoaTransform>>> {
        self.blending_job.output().unwrap().clone()
    }

    #[inline]
    pub fn local_out_ref(&self) -> Ref<'_, Vec<SoaTransform>> {
        self.blending_job.output().unwrap().borrow()
    }

    #[inline]
    pub fn model_out_buf(&self) -> Option<Rc<RefCell<Vec<Mat4>>>> {
        match &self.l2m_job {
            Some(l2m_job) => Some(l2m_job.output().unwrap().clone()),
            None => None,
        }
    }

    #[inline]
    pub fn model_out_ref(&self) -> Option<Ref<'_, Vec<Mat4>>> {
        match &self.l2m_job {
            Some(l2m_job) => Some(l2m_job.output().unwrap().borrow()),
            None => None,
        }
    }
}

macro_rules! animation_state {
    ($animations:expr, $idx:expr, $then:block) => {
        match $animations.get($idx) {
            Some(state) if !state.is_empty() => state,
            _ => $then,
        }
    };
    ($animations:expr, $idx:expr, break) => {
        animation_state!($animations, $idx, { break })
    };
    ($animations:expr, $idx:expr, return) => {
        animation_state!($animations, $idx, { return Ok(()) })
    };
}

#[derive(Debug)]
struct ActionData {
    id: NumID,
    tmpl_id: StrID,
    frame: u32,
    job_current: u32,
    job_past: u32,
    job_future: u32,
}

impl Default for ActionData {
    fn default() -> ActionData {
        ActionData {
            id: 0,
            tmpl_id: StrID::default(),
            frame: 0,
            job_current: u32::MAX,
            job_past: u32::MAX,
            job_future: u32::MAX,
        }
    }
}

impl ActionData {
    fn init<F>(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: &Box<dyn StateAction>,
        skeleton: &Skeleton,
        mut load: F,
    ) -> XResult<()>
    where
        F: FnMut(&Symbol) -> XResult<Rc<Animation>>,
    {
        if state.animations[0].is_empty() {
            return Err(XError::unexpected("ActionData::init() animation states len"));
        }

        self.id = state.id;
        self.tmpl_id = state.tmpl_id.clone();
        self.frame = frame;

        let mut pnext: *mut u32 = &mut self.job_current;
        for state in &state.animations {
            if state.is_empty() {
                break;
            }

            let pos = arena.alloc_and_reptr(&mut pnext);
            let sd = arena.get_mut(pos);
            let animation = load(&state.file)?;
            sd.init(state.animation_id, state.file.clone(), skeleton, animation);
            sd.frame = frame;
            sd.weight = state.weight;
            sd.sampling_job.set_ratio(state.ratio);

            unsafe { *pnext = pos };
            pnext = &mut sd.next;
        }
        self.job_past = self.job_current;
        Ok(())
    }

    fn reuse<F>(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: &Box<dyn StateAction>,
        skeleton: &Skeleton,
        load: F,
    ) -> XResult<()>
    where
        F: FnMut(&Symbol) -> XResult<Rc<Animation>>,
    {
        self.id = state.id;
        self.job_future = self.job_past;
        self.job_past = self.job_past;
        self.job_current = self.job_past;
        self.update(arena, frame, state, skeleton, load)
    }

    fn update<F>(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: &Box<dyn StateAction>,
        skeleton: &Skeleton,
        mut load: F,
    ) -> XResult<()>
    where
        F: FnMut(&Symbol) -> XResult<Rc<Animation>>,
    {
        if state.animations[0].is_empty() {
            return Err(XError::unexpected("ActionData::update() animation states len"));
        }
        self.frame = frame;

        // 1. dequeue unused jobs
        let mut iter = self.job_current;
        let mut last = u32::MAX;
        while iter != self.job_future {
            let sd = arena.get_ref(iter);
            if sd.animation_id == state.animations[0].animation_id {
                break;
            }
            last = iter;
            iter = sd.next;
        }
        self.job_current = iter;

        // 2. verify using jobs
        let mut state_idx = 0;
        while iter != self.job_future {
            let state = animation_state!(state.animations, state_idx, {
                // jobs longer than states
                return Err(XError::unexpected("ActionData::update() animation states len"));
            });

            let sd: &mut SamplingData = arena.get_mut(iter);
            if sd.animation_id != state.animation_id {
                return Err(XError::unexpected("ActionData::update() animation states order"));
            }
            sd.frame = frame;
            sd.weight = state.weight;
            sd.sampling_job.set_ratio(state.ratio);

            last = iter;
            iter = sd.next;
            state_idx += 1;
        }
        animation_state!(state.animations, state_idx, return);

        // 3. try reuse jobs
        while self.job_future != u32::MAX {
            let state = animation_state!(state.animations, state_idx, break);

            let sd = arena.get_mut(self.job_future);
            if sd.animation_file == state.file {
                // reuse job already in jobs, don't modify sd.next
                sd.animation_id = state.animation_id;
                sd.frame = frame;
                sd.weight = state.weight;
                sd.sampling_job.set_ratio(state.ratio);

                last = self.job_future;
                self.job_future = sd.next;
                state_idx += 1;
            } else {
                while self.job_future != u32::MAX {
                    let next = arena.get_ref(self.job_future).next;
                    arena.free(self.job_future);
                    self.job_future = next;
                }
                break; // future changed, free all future jobs
            }
        }
        animation_state!(state.animations, state_idx, return);

        // 4. enqueue new jobs
        let mut new_head = u32::MAX;
        let mut pnext: *mut u32 = &mut new_head;
        loop {
            let state = animation_state!(state.animations, state_idx, break);

            let pos = arena.alloc_and_reptr(&mut pnext);
            let sd = arena.get_mut(pos);
            let animation = load(&state.file)?;
            sd.init(state.animation_id, state.file.clone(), skeleton, animation);
            sd.frame = frame;
            sd.weight = state.weight;
            sd.sampling_job.set_ratio(state.ratio);

            unsafe { *pnext = pos };
            pnext = &mut sd.next;
            state_idx += 1;
        }

        // connect new jobs to last existed job
        if last != u32::MAX {
            let sd = arena.get_mut(last);
            sd.next = new_head;
        }
        // jobs is empty / jobs are all dequeued in step1
        if self.job_current == u32::MAX {
            self.job_current = new_head;
        }
        // jobs is empty
        if self.job_past == u32::MAX {
            self.job_past = new_head;
        }
        Ok(())
    }

    fn restore(&mut self, arena: &mut SamplingArena, frame: u32, state: &Box<dyn StateAction>) -> XResult<()> {
        assert!(self.job_past != u32::MAX);

        if state.animations[0].is_empty() {
            return Err(XError::unexpected("ActionData::restore() animation states len"));
        }
        self.frame = frame;

        let mut iter = self.job_past;
        while iter != u32::MAX {
            let sd = arena.get_ref(iter);
            if sd.frame >= frame {
                break;
            }
            iter = sd.next;
        }
        self.job_current = iter;

        let mut state_idx = 0;
        while state_idx < MAX_ACTION_ANIMATION {
            let state = &state.animations[state_idx];
            if state.is_empty() {
                break;
            }

            if iter == u32::MAX {
                return Err(XError::unexpected("ActionData::restore() animation states len"));
            }

            let sd: &mut SamplingData = arena.get_mut(iter);
            if sd.animation_id != state.animation_id {
                return Err(XError::unexpected("ActionData::restore() animation states order"));
            }
            sd.frame = frame;
            sd.weight = state.weight;
            sd.sampling_job.set_ratio(state.ratio);

            state_idx += 1;
            iter = sd.next;
        }
        self.job_future = iter;
        Ok(())
    }

    fn discard_animations_by_frame(&mut self, arena: &mut SamplingArena, frame: u32) {
        assert!(self.job_past != u32::MAX);
        assert!(self.job_current != u32::MAX);

        while self.job_past != self.job_current {
            let sd = arena.get_mut(self.job_past);
            if sd.frame > frame {
                break;
            }
            let past = self.job_past;
            self.job_past = sd.next;
            arena.free(past);
        }
    }

    fn discard_all_animations(&mut self, arena: &mut SamplingArena) {
        assert!(self.job_past != u32::MAX);
        assert!(self.job_current != u32::MAX);

        while self.job_past != u32::MAX {
            let past = self.job_past;
            self.job_past = arena.get_ref(past).next;
            arena.free(past);
        }
        self.job_past = u32::MAX;
        self.job_current = u32::MAX;
        self.job_future = u32::MAX;
    }

    fn animate(&mut self, arena: &mut SamplingArena, blending_job: &mut BlendingJob) -> XResult<()> {
        let mut iter = self.job_current;
        while iter != self.job_future {
            let sd = arena.get_mut(iter);
            iter = sd.next;

            sd.sampling_job.run()?;
            blending_job.layers_mut().push(BlendingLayer::with_weight(
                sd.sampling_job.output().unwrap().clone(),
                sd.weight,
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct SamplingData {
    next: u32,
    animation_id: u32,
    frame: u32,
    weight: f32,
    animation_file: Symbol,
    sampling_job: SamplingJob,
}

impl Default for SamplingData {
    fn default() -> SamplingData {
        SamplingData {
            next: u32::MAX,
            animation_id: 0,
            frame: 0,
            weight: 0.0,
            animation_file: Symbol::default(),
            sampling_job: SamplingJob::default(),
        }
    }
}

impl SamplingData {
    #[inline]
    fn init(&mut self, animation_id: u32, animation_file: Symbol, skeleton: &Skeleton, animation: Rc<Animation>) {
        self.animation_id = animation_id;
        self.frame = 0;
        self.weight = 0.0;
        self.animation_file = animation_file;
        self.sampling_job.set_animation(animation.clone());
        self.sampling_job
            .set_context(SamplingContext::from_animation(&animation));
        self.sampling_job.set_output(Rc::new(RefCell::new(vec![
            SoaTransform::default();
            skeleton.num_soa_joints()
        ])));
        self.next = u32::MAX;
    }
}

#[derive(Debug)]
struct SamplingArena {
    arena: Vec<SamplingData>,
    free: u32,
}

impl SamplingArena {
    #[inline]
    fn new(cap: usize) -> SamplingArena {
        let mut sa: SamplingArena = SamplingArena {
            arena: (0..cap).map(|_| SamplingData::default()).collect(),
            free: 0,
        };
        for (idx, sd) in sa.arena.iter_mut().enumerate() {
            sd.next = idx as u32 + 1;
        }
        sa.arena.last_mut().unwrap().next = u32::MAX;
        sa
    }

    #[inline]
    fn alloc_and_reptr(&mut self, p: &mut *mut u32) -> u32 {
        if self.free == u32::MAX {
            let prev_len = self.arena.len() as u32;
            let old_begin = self.arena.as_ptr() as usize;
            let old_end = old_begin + self.arena.len() * mem::size_of::<SamplingData>();

            self.arena.reserve_exact((prev_len * 2) as usize);
            for idx in prev_len..(prev_len * 2) {
                let mut sd = SamplingData::default();
                sd.next = idx + 1;
                self.arena.push(sd);
            }
            self.get_mut(prev_len * 2 - 1).next = u32::MAX;
            self.free = prev_len;

            let pn = *p as usize;
            if (old_begin <= pn) && (pn < old_end) {
                *p = ((self.arena.as_ptr() as usize) + (pn - old_begin)) as *mut u32;
            }
        }

        let pos = self.free;
        self.free = self.get_ref(pos).next;
        pos
    }

    #[inline]
    fn free(&mut self, pos: u32) {
        let sd = self.get_mut(pos);
        *sd = SamplingData::default();

        unsafe {
            let mut p: *mut u32 = &mut self.free;
            while *p < pos {
                p = &mut self.get_mut(*p).next;
            }
            self.get_mut(pos).next = *p;
            *p = pos;
        }
    }

    #[inline(always)]
    fn get_ref(&self, pos: u32) -> &SamplingData {
        assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked(pos as usize) };
    }

    #[inline(always)]
    fn get_mut(&mut self, pos: u32) -> &mut SamplingData {
        assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked_mut(pos as usize) };
    }
}

#[cfg(test)]
mod tests {
    use jolt_physics_rs::PhysicsSystem;
    use std::ptr;

    use super::*;
    use crate::asset::AssetLoader;
    use crate::logic::test_utils::StateActionEmpty;
    use crate::utils::s;

    fn list_next(arena: &SamplingArena, head: u32) -> Vec<u32> {
        let mut linked_list = Vec::new();
        let mut pos = head;
        while pos != u32::MAX {
            linked_list.push(pos);
            pos = arena.get_ref(pos).next;
        }
        linked_list
    }

    fn list_sampling(arena: &SamplingArena, head: u32, tail: u32) -> Vec<&SamplingData> {
        let mut linked_list = Vec::new();
        let mut pos = head;
        while pos != tail {
            linked_list.push(arena.get_ref(pos));
            pos = arena.get_ref(pos).next;
        }
        linked_list
    }

    fn prepare_resource() -> (Rc<Skeleton>, Rc<Animation>) {
        let mut phy_system = PhysicsSystem::new();
        let body_itf = phy_system.body_interface(false);
        let mut asset_loader = AssetLoader::new(body_itf, "../test-asset").unwrap();
        let skeleton = asset_loader.load_skeleton(&s!("girl_skeleton_logic.ozz")).unwrap();
        let animation = asset_loader
            .load_animation(&s!("girl_animation_logic_stand_idle.ozz"))
            .unwrap();
        (skeleton, animation)
    }

    #[test]
    fn test_sampling_arena() {
        let mut arena = SamplingArena::new(3);
        assert_eq!(
            arena.arena.iter().map(|a| a.next).collect::<Vec<_>>(),
            vec![1, 2, u32::MAX]
        );
        assert_eq!(arena.free, 0);

        assert_eq!(arena.get_ref(0).next, 1);
        assert_eq!(arena.get_mut(1).next, 2);
        assert_eq!(arena.get_mut(2).next, u32::MAX);

        let mut p = (&mut arena.free) as *mut u32;
        let pos0 = arena.alloc_and_reptr(&mut p);
        assert_eq!(pos0, 0);
        assert_eq!(arena.free, 1);
        assert_eq!(p, (&mut arena.free) as *mut u32);

        let pos1 = arena.alloc_and_reptr(&mut ptr::null_mut());
        assert_eq!(pos1, 1);
        assert_eq!(arena.free, 2);

        let pos2 = arena.alloc_and_reptr(&mut ptr::null_mut());
        assert_eq!(pos2, 2);
        assert_eq!(arena.free, u32::MAX);

        let mut p = (&mut arena.get_mut(2).next) as *mut u32;
        let pos3 = arena.alloc_and_reptr(&mut p);
        assert_eq!(pos3, 3);
        assert_eq!(arena.free, 4);
        assert_eq!(
            arena.arena.iter().map(|a| a.next).collect::<Vec<_>>(),
            vec![1, 2, u32::MAX, 4, 5, u32::MAX]
        );
        assert_eq!(p, (&mut arena.get_mut(2).next) as *mut u32);

        arena.free(pos2);
        assert_eq!(arena.free, 2);
        assert_eq!(list_next(&arena, arena.free), vec![2, 4, 5]);

        arena.free(pos0);
        assert_eq!(arena.free, 0);
        assert_eq!(list_next(&arena, arena.free), vec![0, 2, 4, 5]);

        arena.free(pos1);
        assert_eq!(arena.free, 0);
        assert_eq!(list_next(&arena, arena.free), vec![0, 1, 2, 4, 5]);

        arena.free(pos3);
        assert_eq!(arena.free, 0);
        assert_eq!(list_next(&arena, arena.free), vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(
            arena.arena.iter().map(|a| a.next).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, u32::MAX]
        );
    }

    #[test]
    fn test_action_data_init() {
        let (skeleton, animation) = prepare_resource();
        let mut arena = SamplingArena::new(3);

        let state: Box<dyn StateAction> = Box::new(StateActionEmpty::default());
        let mut ad: ActionData = ActionData::default();
        let res = ad.init(&mut arena, 300, &state, &skeleton, |_| Ok(animation.clone()));
        assert!(res.is_err());

        {
            let mut state: Box<dyn StateAction> = Box::new(StateActionEmpty::default());
            state.id = 12345;
            state.tmpl_id = s!("Action.Empty");
            state.animations[0].animation_id = 101;
            state.animations[0].file = s!("anime_1.ozz");
            state.animations[0].ratio = 0.1;
            state.animations[0].weight = 0.7;
            state.animations[1].animation_id = 102;
            state.animations[1].file = s!("anime_2.ozz");
            state.animations[1].ratio = 0.2;
            state.animations[1].weight = 0.3;
            let mut ad: ActionData = ActionData::default();
            ad.init(&mut arena, 120, &state, &skeleton, |_| Ok(animation.clone()))
                .unwrap();
            assert_eq!(ad.id, 12345);
            assert_eq!(ad.tmpl_id, s!("Action.Empty"));
            assert_eq!(ad.frame, 120);
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);
            assert_eq!(current[0].animation_id, 101);
            assert_eq!(current[0].frame, 120);
            assert_eq!(current[0].weight, 0.7);
            assert_eq!(current[0].animation_file, s!("anime_1.ozz"));
            assert_eq!(current[0].sampling_job.ratio(), 0.1);
            assert_eq!(current[1].animation_id, 102);
            assert_eq!(current[1].frame, 120);
            assert_eq!(current[1].weight, 0.3);
            assert_eq!(current[1].animation_file, s!("anime_2.ozz"));
            assert_eq!(current[1].sampling_job.ratio(), 0.2);
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }
    }

    #[test]
    fn test_action_data_update() {
        let (skeleton, animation) = prepare_resource();
        let mut arena = SamplingArena::new(3);
        let mut ad: ActionData = ActionData::default();

        let state: Box<dyn StateAction> = Box::new(StateActionEmpty::default());
        let res = ad.update(&mut arena, 30, &state, &skeleton, |_| Ok(animation.clone()));
        assert!(res.is_err());

        {
            let mut state: Box<dyn StateAction> = Box::new(StateActionEmpty::default());
            state.animations[0].animation_id = 11;
            state.animations[0].file = s!("anime_1.ozz");
            state.animations[0].ratio = 0.4;
            state.animations[0].weight = 0.7;
            state.animations[1].animation_id = 12;
            state.animations[1].file = s!("anime_2.ozz");
            state.animations[1].ratio = 0.6;
            state.animations[1].weight = 0.3;
            ad.update(&mut arena, 31, &state, &skeleton, |_| Ok(animation.clone()))
                .unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);
            assert_eq!(current[0].animation_id, 11);
            assert_eq!(current[0].frame, 31);
            assert_eq!(current[0].weight, 0.7);
            assert_eq!(current[0].animation_file, s!("anime_1.ozz"));
            assert_eq!(current[0].sampling_job.ratio(), 0.4);
            assert_eq!(current[1].animation_id, 12);
            assert_eq!(current[1].frame, 31);
            assert_eq!(current[1].weight, 0.3);
            assert_eq!(current[1].animation_file, s!("anime_2.ozz"));
            assert_eq!(current[1].sampling_job.ratio(), 0.6);
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }

        {
            let mut state: Box<dyn StateAction> = Box::new(StateActionEmpty::default());
            state.animations[0].animation_id = 12;
            state.animations[0].file = s!("anime_2.ozz");
            state.animations[0].ratio = 0.7;
            state.animations[0].weight = 1.0;
            state.animations[1].animation_id = 13;
            state.animations[1].file = s!("anime_3.ozz");
            state.animations[2].animation_id = 14;
            state.animations[2].file = s!("anime_4.ozz");
            ad.update(&mut arena, 32, &state, &skeleton, |_| Ok(animation.clone()))
                .unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 3);
            assert_eq!(current[0].animation_id, 12);
            assert_eq!(current[0].frame, 32);
            assert_eq!(current[0].weight, 1.0);
            assert_eq!(current[0].animation_file, s!("anime_2.ozz"));
            assert_eq!(current[0].sampling_job.ratio(), 0.7);
            assert_eq!(current[1].animation_id, 13);
            assert_eq!(current[1].animation_file, s!("anime_3.ozz"));
            assert_eq!(current[2].animation_id, 14);
            assert_eq!(current[2].animation_file, s!("anime_4.ozz"));
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 1);
            assert_eq!(past[0].animation_id, 11);
            assert_eq!(past[0].frame, 31);
            assert_eq!(past[0].animation_file, s!("anime_1.ozz"));
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }
    }

    #[test]
    fn test_action_data_restore() {
        fn prepare() -> (
            Rc<Skeleton>,
            Rc<Animation>,
            SamplingArena,
            ActionData,
            Box<dyn StateAction>,
            Box<dyn StateAction>,
        ) {
            let (skeleton, animation) = prepare_resource();
            let mut arena = SamplingArena::new(3);
            let mut ad: ActionData = ActionData::default();

            let mut state1: Box<dyn StateAction> = Box::new(StateActionEmpty::default());
            state1.animations[0].animation_id = 11;
            state1.animations[0].file = s!("anime_1.ozz");
            state1.animations[0].ratio = 0.1;
            state1.animations[0].weight = 0.4;
            state1.animations[1].animation_id = 12;
            state1.animations[1].file = s!("anime_2.ozz");
            state1.animations[1].ratio = 0.2;
            state1.animations[1].weight = 0.6;
            ad.update(&mut arena, 50, &state1, &skeleton, |_| Ok(animation.clone()))
                .unwrap();
            let past = list_sampling(&arena, ad.job_past, u32::MAX);
            assert_eq!(past.len(), 2);

            let mut state2: Box<dyn StateAction> = Box::new(StateActionEmpty::default());
            state2.animations[0].animation_id = 12;
            state2.animations[0].file = s!("anime_2.ozz");
            state2.animations[0].ratio = 0.2;
            state2.animations[0].weight = 1.0;
            state2.animations[1].animation_id = 13;
            state2.animations[1].file = s!("anime_3.ozz");
            state2.animations[2].animation_id = 14;
            state2.animations[2].file = s!("anime_4.ozz");
            ad.update(&mut arena, 51, &state2, &skeleton, |_| Ok(animation.clone()))
                .unwrap();
            let past = list_sampling(&arena, ad.job_past, u32::MAX);
            assert_eq!(past.len(), 4);

            (skeleton, animation, arena, ad, state1, state2)
        }

        {
            let (skeleton, animation, mut arena, mut ad, state1, state2) = prepare();
            ad.restore(&mut arena, 50, &state1).unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);
            assert_eq!(current[0].animation_id, 11);
            assert_eq!(current[0].frame, 50);
            assert_eq!(current[0].weight, 0.4);
            assert_eq!(current[0].animation_file, s!("anime_1.ozz"));
            assert_eq!(current[0].sampling_job.ratio(), 0.1);
            assert_eq!(current[1].animation_id, 12);
            assert_eq!(current[1].frame, 50);
            assert_eq!(current[1].weight, 0.6);
            assert_eq!(current[1].animation_file, s!("anime_2.ozz"));
            assert_eq!(current[1].sampling_job.ratio(), 0.2);
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 2);
            assert_eq!(list_next(&arena, arena.free), vec![4, 5]);

            ad.update(&mut arena, 51, &state2, &skeleton, |_| Ok(animation.clone()))
                .unwrap();
            let current = list_sampling(&arena, ad.job_current, u32::MAX);
            assert_eq!(current.len(), 3);
            assert_eq!(current[0].animation_id, 12);
            assert_eq!(current[0].frame, 51);
            assert_eq!(current[0].weight, 1.0);
            assert_eq!(current[0].animation_file, s!("anime_2.ozz"));
            assert_eq!(current[0].sampling_job.ratio(), 0.2);
            assert_eq!(current[1].animation_id, 13);
            assert_eq!(current[1].animation_file, s!("anime_3.ozz"));
            assert_eq!(current[2].animation_id, 14);
            assert_eq!(current[2].animation_file, s!("anime_4.ozz"));
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 1);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
            assert_eq!(list_next(&arena, arena.free), vec![4, 5]);
        }

        {
            let (skeleton, animation, mut arena, mut ad, state1, mut state2) = prepare();
            ad.restore(&mut arena, 50, &state1).unwrap();
            state2.animations[1].animation_id = 13;
            state2.animations[1].file = s!("anime_x.ozz");
            state2.animations[1].ratio = 0.5;
            state2.animations[1].weight = 0.5;
            state2.animations[2].animation_id = 0;
            state2.animations[2].file = Symbol::default();
            ad.update(&mut arena, 51, &state2, &skeleton, |_| Ok(animation.clone()))
                .unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 1);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
            assert_eq!(list_next(&arena, arena.free), vec![3, 4, 5]);
        }
    }

    #[test]
    fn test_skeleton_animator_update() {
        let (skeleton, animation) = prepare_resource();
        let mut sa = SkeletalAnimator::new(skeleton, false, 0, 3);

        let mut states: Vec<Box<dyn StateAction>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states[0].id = 21;
        states[0].tmpl_id = s!("Action.Empty1");
        states[0].animations[0].animation_id = 101;
        states[0].animations[0].file = s!("anime_1.ozz");
        states[1].id = 22;
        states[1].tmpl_id = s!("Action.Empty2");
        states[1].animations[0].animation_id = 102;
        states[1].animations[0].file = s!("anime_2.ozz");
        sa.update(105, &states, |_| Ok(animation.clone())).unwrap();
        assert_eq!(sa.action_queue.len(), 2);
        assert_eq!(sa.action_queue[0].id, 21);
        assert_eq!(sa.action_queue[0].tmpl_id, s!("Action.Empty1"));
        assert_eq!(sa.action_queue[0].frame, 105);
        let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[0].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 101);
        assert_eq!(sa.action_queue[1].id, 22);
        assert_eq!(sa.action_queue[1].tmpl_id, s!("Action.Empty2"));
        assert_eq!(sa.action_queue[1].frame, 105);
        let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[1].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 102);

        let mut states: Vec<Box<dyn StateAction>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states[0].id = 22;
        states[0].tmpl_id = s!("Action.Empty2");
        states[0].animations[0].animation_id = 102;
        states[0].animations[0].file = s!("anime_2.ozz");
        states[1].id = 23;
        states[1].tmpl_id = s!("Action.Empty3");
        states[1].animations[0].animation_id = 103;
        states[1].animations[0].file = s!("anime_3.ozz");
        sa.update(106, &states, |_| Ok(animation.clone())).unwrap();
        assert_eq!(sa.action_queue.len(), 2);
        assert_eq!(sa.action_queue[0].id, 22);
        assert_eq!(sa.action_queue[0].tmpl_id, s!("Action.Empty2"));
        assert_eq!(sa.action_queue[0].frame, 106);
        let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[0].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 102);
        assert_eq!(sa.action_queue[1].id, 23);
        assert_eq!(sa.action_queue[1].tmpl_id, s!("Action.Empty3"));
        assert_eq!(sa.action_queue[1].frame, 106);
        let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[1].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 103);
        assert_eq!(sa.action_queue.past_len(), 1);
        assert_eq!(sa.action_queue.future_len(), 0);
        assert_eq!(sa.action_queue.all_len(), 3);
    }

    #[test]
    fn test_skeleton_animator_restore() {
        fn prepare() -> (
            Rc<Animation>,
            SkeletalAnimator,
            Vec<Box<dyn StateAction>>,
            Vec<Box<dyn StateAction>>,
        ) {
            let (skeleton, animation) = prepare_resource();
            let mut sa = SkeletalAnimator::new(skeleton, false, 0, 3);

            let mut states1: Vec<Box<dyn StateAction>> = vec![
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
            ];
            states1[0].id = 41;
            states1[0].tmpl_id = s!("Action.Empty1");
            states1[0].animations[0].animation_id = 101;
            states1[0].animations[0].file = s!("anime_1.ozz");
            states1[1].id = 42;
            states1[1].tmpl_id = s!("Action.Empty2");
            states1[1].animations[0].animation_id = 102;
            states1[1].animations[0].file = s!("anime_2.ozz");
            sa.update(205, &states1, |_| Ok(animation.clone())).unwrap();

            let mut states2: Vec<Box<dyn StateAction>> = vec![
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
            ];
            states2[0].id = 42;
            states2[0].tmpl_id = s!("Action.Empty2");
            states2[0].animations[0].animation_id = 102;
            states2[0].animations[0].file = s!("anime_2.ozz");
            states2[1].id = 43;
            states2[1].tmpl_id = s!("Action.Empty3");
            states2[1].animations[0].animation_id = 103;
            states2[1].animations[0].file = s!("anime_3.ozz");
            states2[2].id = 44;
            states2[2].tmpl_id = s!("Action.Empty4");
            states2[2].animations[0].animation_id = 104;
            states2[2].animations[0].file = s!("anime_4.ozz");
            sa.update(206, &states2, |_| Ok(animation.clone())).unwrap();

            (animation, sa, states1, states2)
        }

        {
            let (animation, mut sa, states1, mut states2) = prepare();
            sa.restore(205, &states1).unwrap();
            assert_eq!(sa.action_queue.len(), 2);
            assert_eq!(sa.action_queue[0].id, 41);
            assert_eq!(sa.action_queue[0].tmpl_id, s!("Action.Empty1"));
            assert_eq!(sa.action_queue[0].frame, 205);
            let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[0].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 101);
            assert_eq!(sa.action_queue[1].id, 42);
            assert_eq!(sa.action_queue[1].tmpl_id, s!("Action.Empty2"));
            assert_eq!(sa.action_queue[1].frame, 205);
            let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[1].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 102);
            assert_eq!(sa.action_queue.past_len(), 0);
            assert_eq!(sa.action_queue.future_len(), 2);
            assert_eq!(sa.action_queue.all_len(), 4);

            states2.pop();
            sa.update(206, &states2, |_| Ok(animation.clone())).unwrap();
            assert_eq!(sa.action_queue.len(), 2);
            assert_eq!(sa.action_queue[0].id, 42);
            assert_eq!(sa.action_queue[0].tmpl_id, s!("Action.Empty2"));
            assert_eq!(sa.action_queue[0].frame, 206);
            let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[0].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 102);
            assert_eq!(sa.action_queue[1].id, 43);
            assert_eq!(sa.action_queue[1].tmpl_id, s!("Action.Empty3"));
            assert_eq!(sa.action_queue[1].frame, 206);
            let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[1].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 103);
            assert_eq!(sa.action_queue.past_len(), 1);
            assert_eq!(sa.action_queue.future_len(), 1);
            assert_eq!(sa.action_queue.all_len(), 4);
        }

        {
            let (animation, mut sa, states1, mut states2) = prepare();
            sa.restore(205, &states1).unwrap();
            states2[1].id = 45;
            states2[1].tmpl_id = s!("Action.EmptyX");
            states2[1].animations[0].animation_id = 105;
            states2[1].animations[0].file = s!("anime_x.ozz");
            states2.pop();
            sa.update(206, &states2, |_| Ok(animation.clone())).unwrap();
            assert_eq!(sa.action_queue.len(), 2);
            assert_eq!(sa.action_queue[0].id, 42);
            assert_eq!(sa.action_queue[0].tmpl_id, s!("Action.Empty2"));
            assert_eq!(sa.action_queue[0].frame, 206);
            let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[0].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 102);
            assert_eq!(sa.action_queue[1].id, 45);
            assert_eq!(sa.action_queue[1].tmpl_id, s!("Action.EmptyX"));
            assert_eq!(sa.action_queue[1].frame, 206);
            let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[1].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 105);
            assert_eq!(sa.action_queue.past_len(), 1);
            assert_eq!(sa.action_queue.future_len(), 0);
            assert_eq!(sa.action_queue.all_len(), 3);
        }
    }

    #[test]
    fn test_skeleton_animator_discard() {
        let (skeleton, animation) = prepare_resource();
        let mut sa = SkeletalAnimator::new(skeleton, false, 0, 3);

        let mut states1: Vec<Box<dyn StateAction>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states1[0].id = 41;
        states1[0].tmpl_id = s!("Action.Empty1");
        states1[0].animations[0].animation_id = 101;
        states1[0].animations[0].file = s!("anime_1.ozz");
        states1[1].id = 42;
        states1[1].tmpl_id = s!("Action.Empty2");
        states1[1].animations[0].animation_id = 102;
        states1[1].animations[0].file = s!("anime_2.ozz");
        sa.update(205, &states1, |_| Ok(animation.clone())).unwrap();

        let mut states2: Vec<Box<dyn StateAction>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states2[0].id = 42;
        states2[0].tmpl_id = s!("Action.Empty2");
        states2[0].animations[0].animation_id = 103;
        states2[0].animations[0].file = s!("anime_3.ozz");
        states2[1].id = 43;
        states2[1].tmpl_id = s!("Action.Empty4");
        states2[1].animations[0].animation_id = 104;
        states2[1].animations[0].file = s!("anime_4.ozz");
        sa.update(206, &states2, |_| Ok(animation.clone())).unwrap();

        sa.discard(205);
        assert_eq!(sa.action_queue.len(), 2);
        assert_eq!(sa.action_queue.past_len(), 0);
        assert_eq!(sa.action_queue.future_len(), 0);
        let sampling = list_sampling(&sa.sampling_arena, sa.action_queue[0].job_past, u32::MAX);
        assert_eq!(sampling.len(), 1);
    }
}
