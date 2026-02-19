use glam_ext::{Mat4, Transform3A};
use ozz_animation_rs::{
    ozz_rc_buf, Animation, BlendingJob, BlendingLayer, LocalToModelJob, SamplingContext, SamplingJob, Skeleton,
    SoaTransform,
};
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use crate::animation::hit_motion::{HitMotion, HitMotionSampler};
use crate::animation::rest_poses_to_model_transforms;
use crate::animation::utils::{matrices_to_transforms, WeaponTransform};
use crate::animation::weapon_motion::{normalize_weapons_by_weight, sample_weapons_by_name_with_weight, WeaponMotion};
use crate::asset::AssetLoader;
use crate::logic::{LogicActionAnimationID, StateActionAnimation, StateActionAny};
use crate::utils::{xfrom, xres, HistoryQueue, NumID, Symbol, TmplID, XResult};

#[derive(Debug)]
pub struct Animator {
    skeleton: Rc<Skeleton>,
    blending_job: BlendingJob,
    l2m_job: LocalToModelJob,
    weapon_transforms: Vec<WeaponTransform>,
    action_queue: HistoryQueue<ActionData>,
    sampling_arena: SamplingArena,
    model_transforms: Vec<Transform3A>,
}

impl Animator {
    pub fn new(skeleton: Rc<Skeleton>, action_cap: usize, sampling_cap: usize) -> XResult<Animator> {
        let mut model_transforms = vec![Transform3A::ZERO; skeleton.num_joints()];
        rest_poses_to_model_transforms(&skeleton, &mut model_transforms)?;

        let mut animator: Animator = Animator {
            skeleton: skeleton.clone(),
            blending_job: BlendingJob::default(),
            l2m_job: LocalToModelJob::default(),
            weapon_transforms: Vec::with_capacity(4),
            action_queue: HistoryQueue::with_capacity(action_cap.max(1)),
            sampling_arena: SamplingArena::new(sampling_cap.max(1)),
            model_transforms,
        };

        animator.blending_job.set_skeleton(animator.skeleton.clone());
        animator.blending_job.set_output(ozz_rc_buf(vec![
            SoaTransform::default();
            animator.skeleton.num_soa_joints()
        ]));

        animator.l2m_job.set_skeleton(animator.skeleton.clone());
        animator
            .l2m_job
            .set_input(animator.blending_job.output().unwrap().clone());
        animator
            .l2m_job
            .set_output(ozz_rc_buf(vec![Mat4::default(); animator.skeleton.num_joints()]));
        Ok(animator)
    }

    pub fn update(&mut self, frame: u32, states: &[Box<dyn StateActionAny>], loader: &mut AssetLoader) -> XResult<()> {
        if states.is_empty() {
            return xres!(LogicBadState; "states empty");
        }

        // 1. dequeue unused actions
        self.action_queue.dequeue(|ad| states[0].id != ad.id);

        // 2. verify using actions
        if self.action_queue.len() > states.len() {
            return xres!(LogicBadState; "states len");
        }
        for (idx, state) in states.iter().enumerate() {
            let Some(ad) = self.action_queue.get_mut(idx)
            else {
                break;
            };
            if state.id != ad.id {
                return xres!(LogicBadState; "state id");
            }
            ad.update(&mut self.sampling_arena, frame, state, &self.skeleton, loader)?;
        }

        // 3. try reuse actions
        for idx in self.action_queue.len()..states.len() {
            let state = &states[idx];
            let reused = self
                .action_queue
                .enqueue_reuse(|ad| {
                    if ad.tmpl_id == state.tmpl_id {
                        ad.reuse(&mut self.sampling_arena, frame, state, &self.skeleton, loader)?;
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
            ad.init(&mut self.sampling_arena, frame, state, &self.skeleton, loader)?;
            self.action_queue.enqueue_new(ad);
        }
        Ok(())
    }

    pub fn restore(&mut self, frame: u32, states: &[Box<dyn StateActionAny>]) -> XResult<()> {
        if states.is_empty() {
            return xres!(LogicBadState; "states empty");
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
                }
                else {
                    return xres!(LogicBadState; "state id");
                }
            }
            Ok(1)
        })?;
        if state_iter.next().is_some() {
            return xres!(LogicBadState; "states next");
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
        self.blending_job.layers_mut().clear();
        self.weapon_transforms.clear();
        for ad in self.action_queue.iter_mut() {
            ad.animate(
                &mut self.sampling_arena,
                &mut self.blending_job,
                &mut self.weapon_transforms,
            )?;
        }
        self.blending_job.run().map_err(xfrom!())?;
        self.l2m_job.run().map_err(xfrom!())?;
        matrices_to_transforms(
            self.l2m_job.output().unwrap().borrow().as_slice(),
            &mut self.model_transforms,
        )?;

        normalize_weapons_by_weight(&mut self.weapon_transforms);

        if let Some(current_action) = self.action_queue.last_mut() {
            current_action.animate_hit_motion(
                &mut self.sampling_arena,
                &self.model_transforms,
                &self.weapon_transforms,
            )?;
        }
        Ok(())
    }

    #[inline]
    pub fn model_transforms(&self) -> &[Transform3A] {
        &self.model_transforms
    }

    #[inline]
    pub fn weapon_transforms(&self) -> &[WeaponTransform] {
        &self.weapon_transforms
    }

    #[inline]
    pub fn hit_motion_info(&self) -> Option<(LogicActionAnimationID, &HitMotionSampler)> {
        if let Some(current_action) = self.action_queue.last() {
            if let (animation_id, Some(sampler)) = current_action.hit_motion_info(&self.sampling_arena) {
                let id = LogicActionAnimationID::new(current_action.id, animation_id);
                return Some((id, sampler));
            }
        }
        None
    }
}

macro_rules! animation_state {
    ($animations:expr, $idx:expr, $then:block) => {
        match $animations.get($idx) {
            Some(state) => state,
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
    tmpl_id: TmplID,
    frame: u32,
    job_current: u32,
    job_past: u32,
    job_future: u32,
}

impl Default for ActionData {
    fn default() -> ActionData {
        ActionData {
            id: NumID::INVALID,
            tmpl_id: TmplID::default(),
            frame: 0,
            job_current: u32::MAX,
            job_past: u32::MAX,
            job_future: u32::MAX,
        }
    }
}

impl ActionData {
    fn init(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: &Box<dyn StateActionAny>,
        skeleton: &Skeleton,
        loader: &mut AssetLoader,
    ) -> XResult<()> {
        if state.animations.is_empty() {
            return xres!(LogicBadState; "animations empty");
        }

        self.id = state.id;
        self.tmpl_id = state.tmpl_id;
        self.frame = frame;

        let mut pnext: *mut u32 = &mut self.job_current;
        for anim_state in &state.animations {
            let pos = arena.alloc_and_reptr(&mut pnext);
            let sd = arena.get_mut(pos);
            let (animation, weapon_motion, hit_motion) = Self::load_resources(loader, anim_state)?;
            sd.init(
                anim_state.animation_id,
                anim_state.files,
                skeleton,
                animation,
                weapon_motion,
                hit_motion,
            )?;
            sd.frame = frame;
            sd.weight = anim_state.weight * state.fade_in_weight;
            sd.sampling_job.set_ratio(anim_state.ratio);

            unsafe { *pnext = pos };
            pnext = &mut sd.next;
        }
        self.job_past = self.job_current;
        Ok(())
    }

    fn load_resources(
        loader: &mut AssetLoader,
        anim_state: &StateActionAnimation,
    ) -> XResult<(Rc<Animation>, Option<Rc<WeaponMotion>>, Option<Rc<HitMotion>>)> {
        let animation = loader.load_animation(anim_state.files)?;
        let weapon_motion = match anim_state.weapon_motion {
            true => Some(loader.load_weapon_motion(anim_state.files)?),
            false => None,
        };
        let hit_motion = match anim_state.hit_motion {
            true => Some(loader.load_hit_motion(anim_state.files)?),
            false => None,
        };
        Ok((animation, weapon_motion, hit_motion))
    }

    fn reuse(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: &Box<dyn StateActionAny>,
        skeleton: &Skeleton,
        loader: &mut AssetLoader,
    ) -> XResult<()> {
        self.id = state.id;
        self.job_future = self.job_past;
        // self.job_past = self.job_past;
        self.job_current = self.job_past;
        self.update(arena, frame, state, skeleton, loader)
    }

    fn update(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: &Box<dyn StateActionAny>,
        skeleton: &Skeleton,
        loader: &mut AssetLoader,
    ) -> XResult<()> {
        if state.animations.is_empty() {
            return xres!(LogicBadState; "animations empty");
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
            let anim_state = animation_state!(state.animations, state_idx, {
                // jobs longer than states
                return xres!(LogicBadState; "animations len");
            });

            let sd: &mut SamplingData = arena.get_mut(iter);
            if sd.animation_id != anim_state.animation_id {
                return xres!(LogicBadState; "animation id");
            }
            sd.frame = frame;
            sd.weight = anim_state.weight * state.fade_in_weight;
            sd.sampling_job.set_ratio(anim_state.ratio);

            last = iter;
            iter = sd.next;
            state_idx += 1;
        }
        animation_state!(state.animations, state_idx, return);

        // 3. try reuse jobs
        while self.job_future != u32::MAX {
            let anim_state = animation_state!(state.animations, state_idx, break);

            let sd = arena.get_mut(self.job_future);
            if sd.animation_file == anim_state.files {
                // reuse job already in jobs, don't modify sd.next
                sd.animation_id = anim_state.animation_id;
                sd.frame = frame;
                sd.weight = anim_state.weight * state.fade_in_weight;
                sd.sampling_job.set_ratio(anim_state.ratio);

                last = self.job_future;
                self.job_future = sd.next;
                state_idx += 1;
            }
            else {
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
            let anim_state = animation_state!(state.animations, state_idx, break);

            let pos = arena.alloc_and_reptr(&mut pnext);
            let sd = arena.get_mut(pos);
            let (animation, weapon_motion, hit_motion) = Self::load_resources(loader, anim_state)?;
            sd.init(
                anim_state.animation_id,
                anim_state.files,
                skeleton,
                animation,
                weapon_motion,
                hit_motion,
            )?;
            sd.frame = frame;
            sd.weight = anim_state.weight * state.fade_in_weight;
            sd.sampling_job.set_ratio(anim_state.ratio);

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

    fn restore(&mut self, arena: &mut SamplingArena, frame: u32, state: &Box<dyn StateActionAny>) -> XResult<()> {
        debug_assert!(self.job_past != u32::MAX);

        if state.animations.is_empty() {
            return xres!(LogicBadState; "animations empty");
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

        for anim_state in &state.animations {
            if iter == u32::MAX {
                return xres!(LogicBadState; "animations iter");
            }

            let sd: &mut SamplingData = arena.get_mut(iter);
            if sd.animation_id != anim_state.animation_id {
                return xres!(LogicBadState; "animation id");
            }
            sd.frame = frame;
            sd.weight = anim_state.weight * state.fade_in_weight;
            sd.sampling_job.set_ratio(anim_state.ratio);

            iter = sd.next;
        }
        self.job_future = iter;
        Ok(())
    }

    fn discard_animations_by_frame(&mut self, arena: &mut SamplingArena, frame: u32) {
        debug_assert!(self.job_past != u32::MAX);
        debug_assert!(self.job_current != u32::MAX);

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
        debug_assert!(self.job_past != u32::MAX);
        debug_assert!(self.job_current != u32::MAX);

        while self.job_past != u32::MAX {
            let past = self.job_past;
            self.job_past = arena.get_ref(past).next;
            arena.free(past);
        }
        self.job_past = u32::MAX;
        self.job_current = u32::MAX;
        self.job_future = u32::MAX;
    }

    fn animate(
        &mut self,
        arena: &mut SamplingArena,
        blending_job: &mut BlendingJob,
        weapon_transforms: &mut Vec<WeaponTransform>,
    ) -> XResult<()> {
        let mut iter = self.job_current;
        while iter != self.job_future {
            let sd = arena.get_mut(iter);
            iter = sd.next;

            sd.sampling_job.run().map_err(xfrom!())?;
            blending_job.layers_mut().push(BlendingLayer::with_weight(
                sd.sampling_job.output().unwrap().clone(),
                sd.weight,
            ));

            if let Some(weapon_motion) = &sd.weapon_motion {
                sample_weapons_by_name_with_weight(
                    weapon_motion,
                    sd.sampling_job.ratio(),
                    sd.weight,
                    weapon_transforms,
                )?;
            }
        }
        Ok(())
    }

    fn animate_hit_motion(
        &mut self,
        arena: &mut SamplingArena,
        model_transforms: &[Transform3A],
        weapon_transforms: &[WeaponTransform],
    ) -> XResult<()> {
        if self.job_current == self.job_future {
            return Ok(());
        }

        let sd: &mut SamplingData = arena.get_mut(self.job_current);
        let hit_motion_sampler = match &mut sd.hit_motion_sampler {
            Some(hit_motion_sampler) => hit_motion_sampler,
            None => return Ok(()),
        };

        let animation = sd.sampling_job.animation().unwrap();
        let ratio = sd.sampling_job.ratio();
        let time = animation.duration() * ratio;

        hit_motion_sampler.sample(time, model_transforms, weapon_transforms);
        Ok(())
    }

    fn hit_motion_info<'a, 'b>(&'a self, arena: &'b SamplingArena) -> (u16, Option<&'b HitMotionSampler>) {
        if self.job_current == self.job_future {
            return (u16::MAX, None);
        }
        let sd: &SamplingData = arena.get_ref(self.job_current);
        (sd.animation_id, sd.hit_motion_sampler.as_ref())
    }
}

#[derive(Debug)]
struct SamplingData {
    next: u32,
    animation_id: u16,
    frame: u32,
    weight: f32,
    animation_file: Symbol,
    sampling_job: SamplingJob,
    weapon_motion: Option<Rc<WeaponMotion>>,
    hit_motion_sampler: Option<HitMotionSampler>,
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
            weapon_motion: None,
            hit_motion_sampler: None,
        }
    }
}

impl SamplingData {
    fn init(
        &mut self,
        animation_id: u16,
        animation_file: Symbol,
        skeleton: &Skeleton,
        animation: Rc<Animation>,
        weapon_motion: Option<Rc<WeaponMotion>>,
        hit_motion: Option<Rc<HitMotion>>,
    ) -> XResult<()> {
        self.animation_id = animation_id;
        self.frame = 0;
        self.weight = 0.0;
        self.animation_file = animation_file;

        let ctx = SamplingContext::from_animation(&animation);
        self.sampling_job.set_animation(animation);
        self.sampling_job.set_context(ctx);
        self.sampling_job.set_output(Rc::new(RefCell::new(vec![
            SoaTransform::default();
            skeleton.num_soa_joints()
        ])));

        self.weapon_motion = weapon_motion;

        if let Some(hit_motion) = hit_motion {
            self.hit_motion_sampler = Some(HitMotionSampler::new(hit_motion, skeleton)?);
        }

        self.next = u32::MAX;
        Ok(())
    }
}

#[derive(Debug)]
struct SamplingArena {
    init_cap: usize,
    arena: Vec<SamplingData>,
    free: u32,
}

impl SamplingArena {
    fn new(cap: usize) -> SamplingArena {
        let mut sa: SamplingArena = SamplingArena {
            init_cap: cap,
            arena: (0..cap).map(|_| SamplingData::default()).collect(),
            free: 0,
        };
        for (idx, sd) in sa.arena.iter_mut().enumerate() {
            sd.next = idx as u32 + 1;
        }
        sa.arena.last_mut().unwrap().next = u32::MAX;
        sa
    }

    fn alloc_and_reptr(&mut self, p: &mut *mut u32) -> u32 {
        if self.free == u32::MAX {
            let prev_len = self.arena.len() as u32;
            let old_begin = self.arena.as_ptr() as usize;
            let old_end = old_begin + self.arena.len() * mem::size_of::<SamplingData>();

            self.arena.reserve_exact(self.init_cap);
            for idx in 0..self.init_cap as u32 {
                let mut sd = SamplingData::default();
                sd.next = prev_len + idx + 1;
                self.arena.push(sd);
            }
            log::info!(
                "{} {} {} {}",
                self.arena.len(),
                prev_len * 2 - 1,
                prev_len,
                self.init_cap
            );
            self.arena.last_mut().unwrap().next = u32::MAX;
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
        debug_assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked(pos as usize) };
    }

    #[inline(always)]
    fn get_mut(&mut self, pos: u32) -> &mut SamplingData {
        debug_assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked_mut(pos as usize) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetLoader;
    use crate::consts::TEST_ASSET_PATH;
    use crate::logic::{StateActionAnimation, StateActionEmpty};
    use crate::utils::{id, sb};
    use std::ptr;

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

    fn prepare_resource() -> (AssetLoader, Rc<Skeleton>) {
        let mut asset_loader = AssetLoader::new(TEST_ASSET_PATH).unwrap();
        let skeleton = asset_loader.load_skeleton(sb!("Girl.*")).unwrap();
        (asset_loader, skeleton)
    }

    #[test]
    fn test_sampling_arena() {
        let mut arena = SamplingArena::new(3);
        assert_eq!(arena.arena.iter().map(|a| a.next).collect::<Vec<_>>(), vec![
            1,
            2,
            u32::MAX
        ]);
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
        assert_eq!(arena.arena.iter().map(|a| a.next).collect::<Vec<_>>(), vec![
            1,
            2,
            u32::MAX,
            4,
            5,
            u32::MAX
        ]);
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
        assert_eq!(arena.arena.iter().map(|a| a.next).collect::<Vec<_>>(), vec![
            1,
            2,
            3,
            4,
            5,
            u32::MAX
        ]);
    }

    #[test]
    fn test_action_data_init() {
        let (mut asset_loader, skeleton) = prepare_resource();
        let mut arena = SamplingArena::new(3);

        let state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
        let mut ad: ActionData = ActionData::default();
        let res = ad.init(&mut arena, 300, &state, &skeleton, &mut asset_loader);
        assert!(res.is_err());

        {
            let mut state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state.id = NumID(12345);
            state.tmpl_id = id!("Action.Empty");
            state.fade_in_weight = 0.7;
            state.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Idle_Empty.*"),
                101,
                0.1,
                0.7,
            ));
            state.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Run_Empty.*"),
                102,
                0.2,
                0.3,
            ));
            let mut ad: ActionData = ActionData::default();
            ad.init(&mut arena, 120, &state, &skeleton, &mut asset_loader).unwrap();
            assert_eq!(ad.id, 12345);
            assert_eq!(ad.tmpl_id, id!("Action.Empty"));
            assert_eq!(ad.frame, 120);
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);
            assert_eq!(current[0].animation_id, 101);
            assert_eq!(current[0].frame, 120);
            assert_eq!(current[0].weight, 0.7 * 0.7);
            assert_eq!(current[0].animation_file, "Girl_Idle_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.1);
            assert_eq!(current[1].animation_id, 102);
            assert_eq!(current[1].frame, 120);
            assert_eq!(current[1].weight, 0.3 * 0.7);
            assert_eq!(current[1].animation_file, "Girl_Run_Empty.*");
            assert_eq!(current[1].sampling_job.ratio(), 0.2);
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }
    }

    #[test]
    fn test_action_data_update() {
        let (mut asset_loader, skeleton) = prepare_resource();
        let mut arena = SamplingArena::new(3);
        let mut ad: ActionData = ActionData::default();

        let state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
        let res = ad.update(&mut arena, 30, &state, &skeleton, &mut asset_loader);
        assert!(res.is_err());

        {
            let mut state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state.fade_in_weight = 0.4;
            state.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Idle_Empty.*"),
                11,
                0.4,
                0.7,
            ));
            state.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Run_Empty.*"),
                12,
                0.6,
                0.3,
            ));
            ad.update(&mut arena, 31, &state, &skeleton, &mut asset_loader).unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);
            assert_eq!(current[0].animation_id, 11);
            assert_eq!(current[0].frame, 31);
            assert_eq!(current[0].weight, 0.7 * 0.4);
            assert_eq!(current[0].animation_file, "Girl_Idle_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.4);
            assert_eq!(current[1].animation_id, 12);
            assert_eq!(current[1].frame, 31);
            assert_eq!(current[1].weight, 0.3 * 0.4);
            assert_eq!(current[1].animation_file, "Girl_Run_Empty.*");
            assert_eq!(current[1].sampling_job.ratio(), 0.6);
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }

        {
            let mut state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state.fade_in_weight = 1.0;
            state.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Idle_Empty.*"),
                12,
                0.7,
                1.0,
            ));
            state.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Attack_01A.*"),
                13,
                0.0,
                0.0,
            ));
            state.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Attack_02A.*"),
                14,
                0.0,
                0.0,
            ));
            ad.update(&mut arena, 32, &state, &skeleton, &mut asset_loader).unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 3);
            assert_eq!(current[0].animation_id, 12);
            assert_eq!(current[0].frame, 32);
            assert_eq!(current[0].weight, 1.0);
            assert_eq!(current[0].animation_file, "Girl_Run_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.7);
            assert_eq!(current[1].animation_id, 13);
            assert_eq!(current[1].animation_file, "Girl_Attack_01A.*");
            assert_eq!(current[2].animation_id, 14);
            assert_eq!(current[2].animation_file, "Girl_Attack_02A.*");
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 1);
            assert_eq!(past[0].animation_id, 11);
            assert_eq!(past[0].frame, 31);
            assert_eq!(past[0].animation_file, "Girl_Idle_Empty.*");
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }
    }

    #[test]
    fn test_action_data_restore() {
        fn prepare() -> (
            AssetLoader,
            Rc<Skeleton>,
            SamplingArena,
            ActionData,
            Box<dyn StateActionAny>,
            Box<dyn StateActionAny>,
        ) {
            let (mut asset_loader, skeleton) = prepare_resource();
            let mut arena = SamplingArena::new(3);
            let mut ad: ActionData = ActionData::default();

            let mut state1: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state1.fade_in_weight = 1.0;
            state1.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Idle_Empty.*"),
                11,
                0.1,
                0.4,
            ));
            state1.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Run_Empty.*"),
                12,
                0.2,
                0.6,
            ));
            ad.update(&mut arena, 50, &state1, &skeleton, &mut asset_loader)
                .unwrap();
            let past = list_sampling(&arena, ad.job_past, u32::MAX);
            assert_eq!(past.len(), 2);

            let mut state2: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state2.fade_in_weight = 0.8;
            state2.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Run_Empty.*"),
                12,
                0.2,
                1.0,
            ));
            state2.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Attack_01A.*"),
                13,
                0.0,
                0.0,
            ));
            state2.animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Attack_02A.*"),
                14,
                0.0,
                0.0,
            ));
            ad.update(&mut arena, 51, &state2, &skeleton, &mut asset_loader)
                .unwrap();
            let past = list_sampling(&arena, ad.job_past, u32::MAX);
            assert_eq!(past.len(), 4);

            (asset_loader, skeleton, arena, ad, state1, state2)
        }

        {
            let (mut asset_loader, skeleton, mut arena, mut ad, state1, state2) = prepare();
            ad.restore(&mut arena, 50, &state1).unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);
            assert_eq!(current[0].animation_id, 11);
            assert_eq!(current[0].frame, 50);
            assert_eq!(current[0].weight, 0.4);
            assert_eq!(current[0].animation_file, "Girl_Idle_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.1);
            assert_eq!(current[1].animation_id, 12);
            assert_eq!(current[1].frame, 50);
            assert_eq!(current[1].weight, 0.6);
            assert_eq!(current[1].animation_file, "Girl_Run_Empty.*");
            assert_eq!(current[1].sampling_job.ratio(), 0.2);
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 2);
            assert_eq!(list_next(&arena, arena.free), vec![4, 5]);

            ad.update(&mut arena, 51, &state2, &skeleton, &mut asset_loader)
                .unwrap();
            let current = list_sampling(&arena, ad.job_current, u32::MAX);
            assert_eq!(current.len(), 3);
            assert_eq!(current[0].animation_id, 12);
            assert_eq!(current[0].frame, 51);
            assert_eq!(current[0].weight, 1.0 * 0.8);
            assert_eq!(current[0].animation_file, "Girl_Run_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.2);
            assert_eq!(current[1].animation_id, 13);
            assert_eq!(current[1].animation_file, "Girl_Attack_01A.*");
            assert_eq!(current[2].animation_id, 14);
            assert_eq!(current[2].animation_file, "Girl_Attack_02A.*");
            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 1);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
            assert_eq!(list_next(&arena, arena.free), vec![4, 5]);
        }

        {
            let (mut asset_loader, skeleton, mut arena, mut ad, state1, mut state2) = prepare();
            ad.restore(&mut arena, 50, &state1).unwrap();
            state2.animations[1].animation_id = 13;
            state2.animations[1].files = sb!("Girl_Walk_Empty.*");
            state2.animations[1].ratio = 0.5;
            state2.animations[1].weight = 0.5;
            state2.animations.pop();
            ad.update(&mut arena, 51, &state2, &skeleton, &mut asset_loader)
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
        let (mut asset_loader, skeleton) = prepare_resource();
        let mut animator = Animator::new(skeleton, 0, 3).unwrap();

        let mut states: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states[0].id = NumID(21);
        states[0].tmpl_id = id!("Action.Empty^1");
        states[0].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Idle_Empty.*"),
            101,
            0.0,
            0.0,
        ));
        states[1].id = NumID(22);
        states[1].tmpl_id = id!("Action.Empty^2");
        states[1].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Idle_Empty.*"),
            102,
            0.0,
            0.0,
        ));
        animator.update(105, &states, &mut asset_loader).unwrap();
        assert_eq!(animator.action_queue.len(), 2);
        assert_eq!(animator.action_queue[0].id, 21);
        assert_eq!(animator.action_queue[0].tmpl_id, id!("Action.Empty^1"));
        assert_eq!(animator.action_queue[0].frame, 105);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 101);
        assert_eq!(animator.action_queue[1].id, 22);
        assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^2"));
        assert_eq!(animator.action_queue[1].frame, 105);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 102);

        let mut states: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states[0].id = NumID(22);
        states[0].tmpl_id = id!("Action.Empty^2");
        states[0].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Idle_Empty.*"),
            102,
            0.0,
            0.0,
        ));
        states[1].id = NumID(23);
        states[1].tmpl_id = id!("Action.Empty^3");
        states[1].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Attack_01A.*"),
            103,
            0.0,
            0.0,
        ));
        animator.update(106, &states, &mut asset_loader).unwrap();
        assert_eq!(animator.action_queue.len(), 2);
        assert_eq!(animator.action_queue[0].id, 22);
        assert_eq!(animator.action_queue[0].tmpl_id, id!("Action.Empty^2"));
        assert_eq!(animator.action_queue[0].frame, 106);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 102);
        assert_eq!(animator.action_queue[1].id, 23);
        assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^3"));
        assert_eq!(animator.action_queue[1].frame, 106);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 103);
        assert_eq!(animator.action_queue.past_len(), 1);
        assert_eq!(animator.action_queue.future_len(), 0);
        assert_eq!(animator.action_queue.all_len(), 3);
    }

    #[test]
    fn test_skeleton_animator_restore() {
        fn prepare() -> (
            AssetLoader,
            Animator,
            Vec<Box<dyn StateActionAny>>,
            Vec<Box<dyn StateActionAny>>,
        ) {
            let (mut asset_loader, skeleton) = prepare_resource();
            let mut animator = Animator::new(skeleton, 0, 3).unwrap();

            let mut states1: Vec<Box<dyn StateActionAny>> = vec![
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
            ];
            states1[0].id = NumID(41);
            states1[0].tmpl_id = id!("Action.Empty^1");
            states1[0].animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Idle_Empty.*"),
                101,
                0.0,
                0.0,
            ));
            states1[1].id = NumID(42);
            states1[1].tmpl_id = id!("Action.Empty^2");
            states1[1].animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Run_Empty.*"),
                102,
                0.0,
                0.0,
            ));
            animator.update(205, &states1, &mut asset_loader).unwrap();

            let mut states2: Vec<Box<dyn StateActionAny>> = vec![
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
            ];
            states2[0].id = NumID(42);
            states2[0].tmpl_id = id!("Action.Empty^2");
            states2[0].animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Run_Empty.*"),
                102,
                0.0,
                0.0,
            ));
            states2[1].id = NumID(43);
            states2[1].tmpl_id = id!("Action.Empty^3");
            states2[1].animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Attack_01A.*"),
                103,
                0.0,
                0.0,
            ));
            states2[2].id = NumID(44);
            states2[2].tmpl_id = id!("Action.Empty^4");
            states2[2].animations.push(StateActionAnimation::new_no_motion(
                sb!("Girl_Attack_02A.*"),
                104,
                0.0,
                0.0,
            ));
            animator.update(206, &states2, &mut asset_loader).unwrap();

            (asset_loader, animator, states1, states2)
        }

        {
            let (mut asset_loader, mut animator, states1, mut states2) = prepare();
            animator.restore(205, &states1).unwrap();
            assert_eq!(animator.action_queue.len(), 2);
            assert_eq!(animator.action_queue[0].id, 41);
            assert_eq!(animator.action_queue[0].tmpl_id, id!("Action.Empty^1"));
            assert_eq!(animator.action_queue[0].frame, 205);
            let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 101);
            assert_eq!(animator.action_queue[1].id, 42);
            assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^2"));
            assert_eq!(animator.action_queue[1].frame, 205);
            let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 102);
            assert_eq!(animator.action_queue.past_len(), 0);
            assert_eq!(animator.action_queue.future_len(), 2);
            assert_eq!(animator.action_queue.all_len(), 4);

            states2.pop();
            animator.update(206, &states2, &mut asset_loader).unwrap();
            assert_eq!(animator.action_queue.len(), 2);
            assert_eq!(animator.action_queue[0].id, 42);
            assert_eq!(animator.action_queue[0].tmpl_id, id!("Action.Empty^2"));
            assert_eq!(animator.action_queue[0].frame, 206);
            let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 102);
            assert_eq!(animator.action_queue[1].id, 43);
            assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^3"));
            assert_eq!(animator.action_queue[1].frame, 206);
            let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 103);
            assert_eq!(animator.action_queue.past_len(), 1);
            assert_eq!(animator.action_queue.future_len(), 1);
            assert_eq!(animator.action_queue.all_len(), 4);
        }

        {
            let (mut asset_loader, mut animator, states1, mut states2) = prepare();
            animator.restore(205, &states1).unwrap();
            states2[1].id = NumID(45);
            states2[1].tmpl_id = id!("Action.Empty^X");
            states2[1].animations[0].animation_id = 105;
            states2[1].animations[0].files = sb!("Girl_Walk_Empty.*");
            states2.pop();
            animator.update(206, &states2, &mut asset_loader).unwrap();
            assert_eq!(animator.action_queue.len(), 2);
            assert_eq!(animator.action_queue[0].id, 42);
            assert_eq!(animator.action_queue[0].tmpl_id, id!("Action.Empty^2"));
            assert_eq!(animator.action_queue[0].frame, 206);
            let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 102);
            assert_eq!(animator.action_queue[1].id, 45);
            assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^X"));
            assert_eq!(animator.action_queue[1].frame, 206);
            let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 105);
            assert_eq!(animator.action_queue.past_len(), 1);
            assert_eq!(animator.action_queue.future_len(), 0);
            assert_eq!(animator.action_queue.all_len(), 3);
        }
    }

    #[test]
    fn test_skeleton_animator_discard() {
        let (mut asset_loader, skeleton) = prepare_resource();
        let mut animator = Animator::new(skeleton, 0, 3).unwrap();

        let mut states1: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states1[0].id = NumID(41);
        states1[0].tmpl_id = id!("Action.Empty^1");
        states1[0].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Idle_Empty.*"),
            101,
            0.0,
            0.0,
        ));
        states1[1].id = NumID(42);
        states1[1].tmpl_id = id!("Action.Empty^2");
        states1[1].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Run_Empty.*"),
            102,
            0.0,
            0.0,
        ));
        animator.update(205, &states1, &mut asset_loader).unwrap();

        let mut states2: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states2[0].id = NumID(42);
        states2[0].tmpl_id = id!("Action.Empty^2");
        states2[0].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Attack_01A.*"),
            103,
            0.0,
            0.0,
        ));
        states2[1].id = NumID(43);
        states2[1].tmpl_id = id!("Action.Empty^4");
        states2[1].animations.push(StateActionAnimation::new_no_motion(
            sb!("Girl_Attack_02A.*"),
            104,
            0.0,
            0.0,
        ));
        animator.update(206, &states2, &mut asset_loader).unwrap();

        animator.discard(205);
        assert_eq!(animator.action_queue.len(), 2);
        assert_eq!(animator.action_queue.past_len(), 0);
        assert_eq!(animator.action_queue.future_len(), 0);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_past, u32::MAX);
        assert_eq!(sampling.len(), 1);
    }
}
