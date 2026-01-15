// #![allow(improper_ctypes_definitions)]
#![allow(static_mut_refs)]

use glam::FloatExt;
use glam_ext::Mat4;
use ozz_animation_rs::{
    ozz_rc_buf, Animation, BlendingJob, BlendingLayer, LocalToModelJob, SamplingContext, SamplingJob, Skeleton,
    SoaTransform,
};
use std::cell::{Ref, RefCell};
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use tinyvec::ArrayVec;

use critical_point_core::animation::{
    normalize_weapons_by_weight, rest_poses_to_model_matrices, sample_weapons_by_name_with_weight, SkeletonJointMeta,
    SkeletonMeta, WeaponMotion, WeaponTransform,
};
use critical_point_core::consts::{INVALID_ACTION_ANIMATION_ID, MAX_ACTION_ANIMATION, MAX_ACTION_STATE};
use critical_point_core::logic::StateActionAny;
use critical_point_core::utils::{find_mut_offset_by, xfrom, xres, NumID, Symbol, TmplID, XResult};

use crate::skeletal::resource;
use crate::utils::Return;

#[derive(Debug)]
pub struct SkeletalAnimator {
    skeleton: Arc<Skeleton>,
    blending_job: BlendingJob<Arc<Skeleton>>,
    l2m_job: LocalToModelJob<Arc<Skeleton>>,
    model_rest_poses: Vec<Mat4>,
    weapon_transforms: Vec<WeaponTransform>,
    cs_weapon_transforms: Vec<WeaponTransform>,
    actions: ArrayVec<[u16; MAX_ACTION_STATE * 2]>,
    act_arena: ActionArena,
    samp_arena: SamplingArena,
}

impl SkeletalAnimator {
    pub fn new(skel: Symbol, act_cap: usize, samp_cap: usize) -> XResult<SkeletalAnimator> {
        let skeleton = resource::load_skeleton(skel)?;
        let act_arena = ActionArena::new(act_cap.max(1));
        let samp_arena = SamplingArena::new(skeleton.as_ref(), samp_cap.max(1));

        let mut model_rest_poses = vec![Mat4::IDENTITY; skeleton.num_joints()];
        rest_poses_to_model_matrices(&skeleton, &mut model_rest_poses)?;

        let mut sa: SkeletalAnimator = SkeletalAnimator {
            skeleton: skeleton.clone(),
            blending_job: BlendingJob::default(),
            l2m_job: LocalToModelJob::default(),
            model_rest_poses,
            weapon_transforms: Vec::with_capacity(4),
            cs_weapon_transforms: Vec::with_capacity(4),
            actions: ArrayVec::new(),
            act_arena,
            samp_arena,
        };

        sa.blending_job.set_skeleton(sa.skeleton.clone());
        sa.blending_job
            .set_output(ozz_rc_buf(vec![SoaTransform::default(); sa.skeleton.num_soa_joints()]));

        sa.l2m_job.set_skeleton(sa.skeleton.clone());
        sa.l2m_job.set_input(sa.blending_job.output().unwrap().clone());
        sa.l2m_job
            .set_output(ozz_rc_buf(vec![Mat4::default(); sa.skeleton.num_joints()]));
        Ok(sa)
    }

    pub fn update(&mut self, states: &[Box<dyn StateActionAny>]) -> XResult<()> {
        let mut new_acts = ArrayVec::new();
        let is_init = self.actions.is_empty();

        // handle new actions
        if states.is_empty() {
            return xres!(LogicBadState; "states empty");
        }

        let mut offset = 0;
        for state in states {
            let act;
            (act, offset) = find_mut_offset_by(self.actions.as_mut_slice(), offset, |samp| {
                *samp != u16::MAX
                    && self.act_arena.act(*samp).id == state.id
                    && self.act_arena.act(*samp).tmpl_id == state.tmpl_id
            });

            if let Some(act) = act {
                new_acts.push(*act);
                let ad = self.act_arena.act_mut(*act);
                *act = u16::MAX;
                ad.update(false, &mut self.samp_arena, Some(state))?;
            }
            else {
                let act = self.act_arena.alloc();
                new_acts.push(act);
                let ad = self.act_arena.act_mut(act);
                ad.id = state.id;
                ad.tmpl_id = state.tmpl_id;
                ad.update(is_init, &mut self.samp_arena, Some(state))?;
            }
        }

        // handle old actions
        for act in self.actions.iter().cloned() {
            if act == u16::MAX {
                continue;
            }

            let ad = self.act_arena.act_mut(act);
            ad.update(false, &mut self.samp_arena, None)?;

            if !ad.samplings.is_empty() {
                new_acts.push(act);
            }
            else {
                self.act_arena.free(act);
            }
        }

        self.actions = new_acts;

        Ok(())
    }

    pub fn animate(&mut self, ratio: f32) -> XResult<()> {
        let ratio = ratio.clamp(0.0, 1.0);
        self.weapon_transforms.clear();
        self.blending_job.layers_mut().clear();
        for act in self.actions.iter() {
            let ad = self.act_arena.act(*act);
            ad.animate(
                &mut self.samp_arena,
                &mut self.blending_job,
                &mut self.weapon_transforms,
                ratio,
            )?;
        }
        self.blending_job.run().map_err(xfrom!())?;
        self.l2m_job.run().map_err(xfrom!())?;

        normalize_weapons_by_weight(&mut self.weapon_transforms);
        self.cs_weapon_transforms.clear();
        for i in 0..self.weapon_transforms.len() {
            self.cs_weapon_transforms.push(self.weapon_transforms[i].into());
        }
        Ok(())
    }

    pub fn skeleton_meta(&self) -> SkeletonMeta {
        let skeleton = &self.skeleton;
        let mut joint_metas = vec![SkeletonJointMeta::default(); skeleton.num_joints() as usize];
        for (name, index) in skeleton.joint_names() {
            joint_metas[*index as usize] = SkeletonJointMeta {
                index: *index as i16,
                parent: skeleton.joint_parent(*index),
                name: name.clone(),
            };
        }
        SkeletonMeta {
            num_joints: skeleton.num_joints() as u32,
            num_soa_joints: skeleton.num_soa_joints() as u32,
            joint_metas,
        }
    }

    #[inline]
    pub fn model_rest_poses(&self) -> &[Mat4] {
        &self.model_rest_poses
    }

    #[inline]
    pub fn model_poses(&self) -> Ref<'_, Vec<Mat4>> {
        self.l2m_job.output().unwrap().borrow()
    }

    // #[inline]
    // pub fn weapon_transforms(&self) -> &[WeaponTransform] {
    //     &self.weapon_transforms
    // }

    #[inline]
    pub fn cs_weapon_transforms(&self) -> &[WeaponTransform] {
        &self.cs_weapon_transforms
    }
}

#[derive(Debug)]
struct ActionData {
    next: u16,
    id: NumID,
    tmpl_id: TmplID,
    samplings: ArrayVec<[u16; MAX_ACTION_STATE * 2]>,
    phantoms: PhantomData<Arc<Animation>>,
}

#[derive(Debug)]
struct SamplingData {
    animation_id: u16,
    animation_file: Symbol,
    weight: [f32; 2],
    ratio: [f32; 2],
    sampling_job: SamplingJob<Arc<Animation>>,
    weapon_motion: Option<Arc<WeaponMotion>>,
}

impl ActionData {
    fn update(
        &mut self,
        is_init: bool,
        arena: &mut SamplingArena,
        state: Option<&Box<dyn StateActionAny>>,
    ) -> XResult<()> {
        let mut new_samps = ArrayVec::new();

        // handle new samplings
        if let Some(state) = state {
            if state.animations[0].is_empty() {
                return xres!(LogicBadState; "animations empty");
            }

            let mut offset = 0;
            for anim_state in &state.animations {
                if anim_state.is_empty() {
                    break;
                }

                let samp;
                (samp, offset) = find_mut_offset_by(self.samplings.as_mut_slice(), offset, |samp| {
                    *samp != INVALID_ACTION_ANIMATION_ID && arena.samp(*samp).animation_id == anim_state.animation_id
                });

                if let Some(samp) = samp {
                    new_samps.push(*samp);
                    let sd = arena.samp_mut(*samp);
                    *samp = INVALID_ACTION_ANIMATION_ID;
                    sd.weight = [sd.weight[1], anim_state.weight * state.fade_in_weight];
                    sd.ratio = [sd.ratio[1], anim_state.ratio];

                    // debug_assert!(sd.weight[1] >= 0.0 && sd.weight[1] <= 1.0);
                    // debug_assert!(sd.ratio[1] >= 0.0 && sd.ratio[1] <= 1.0);
                }
                else {
                    let samp = arena.alloc();
                    new_samps.push(samp);
                    let sd = arena.samp_mut(samp);
                    sd.animation_id = anim_state.animation_id;
                    sd.animation_file = anim_state.files;
                    sd.sampling_job
                        .set_animation(resource::load_animation(sd.animation_file)?);
                    if anim_state.weapon_motion {
                        sd.weapon_motion = Some(resource::load_weapon_motion(sd.animation_file)?);
                    }
                    if is_init {
                        sd.weight = [anim_state.weight * state.fade_in_weight; 2];
                    }
                    else {
                        sd.weight = [0.0, anim_state.weight * state.fade_in_weight];
                    }
                    sd.ratio = [anim_state.ratio; 2];

                    // debug_assert!(sd.weight[1] >= 0.0 && sd.weight[1] <= 1.0);
                    // debug_assert!(sd.ratio[1] >= 0.0 && sd.ratio[1] <= 1.0);
                }
            }
        }

        // handle old samplings
        for samp in self.samplings.iter().cloned() {
            if samp == INVALID_ACTION_ANIMATION_ID {
                continue;
            }

            let sd = arena.samp_mut(samp);
            if sd.weight[1] != 0.0 {
                new_samps.push(samp);
                sd.weight = [sd.weight[1], 0.0];
                sd.ratio = [sd.ratio[1]; 2];
            }
            else {
                arena.free(samp);
            }
        }

        self.samplings = new_samps;
        Ok(())
    }

    fn animate(
        &self,
        arena: &mut SamplingArena,
        blending_job: &mut BlendingJob<Arc<Skeleton>>,
        weapon_transforms: &mut Vec<WeaponTransform>,
        ratio: f32,
    ) -> XResult<()> {
        for samp in self.samplings.iter().cloned() {
            let sd = arena.samp_mut(samp);

            let anim_weight = f32::lerp(sd.weight[0], sd.weight[1], ratio);
            if anim_weight <= 0.0 {
                continue;
            }

            let ascend = sd.ratio[1] >= sd.ratio[0];
            let inner = (sd.ratio[1] - sd.ratio[0]).abs() <= 0.5;
            let anim_ratio = match (inner, ascend) {
                // 0.1 - 0.2
                (true, true) => f32::lerp(sd.ratio[0], sd.ratio[1], ratio),
                // 0.2 - 0.1
                (true, false) => f32::lerp(sd.ratio[0], sd.ratio[1], ratio),
                // 0.9 - 0.1
                (false, false) => f32::lerp(sd.ratio[0], sd.ratio[1] + 1.0, ratio) % 1.0,
                // 0.1 - 0.9
                (false, true) => f32::lerp(sd.ratio[0] + 1.0, sd.ratio[1], ratio) % 1.0,
            };

            sd.sampling_job.set_ratio(anim_ratio);
            sd.sampling_job.run().map_err(xfrom!())?;
            blending_job.layers_mut().push(BlendingLayer::with_weight(
                sd.sampling_job.output().unwrap().clone(),
                anim_weight,
            ));

            if let Some(weapon_motion) = &sd.weapon_motion {
                sample_weapons_by_name_with_weight(weapon_motion, anim_ratio, anim_weight, weapon_transforms)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ActionArena {
    init_cap: usize,
    arena: Vec<ActionData>,
    free: u16,
}

impl ActionArena {
    fn new(cap: usize) -> ActionArena {
        let mut arena: ActionArena = ActionArena {
            init_cap: cap,
            arena: (0..cap).map(|_| Self::default()).collect(),
            free: 0,
        };
        for (idx, ad) in arena.arena.iter_mut().enumerate() {
            ad.next = idx as u16 + 1;
        }
        arena.arena.last_mut().unwrap().next = u16::MAX;
        arena
    }

    fn alloc(&mut self) -> u16 {
        if self.free == u16::MAX {
            let prev_len = self.arena.len() as u16;
            self.arena.reserve_exact(self.init_cap);
            for idx in prev_len..(prev_len + self.init_cap as u16) {
                let mut ad = Self::default();
                ad.next = idx + 1;
                self.arena.push(ad);
            }
            self.arena.last_mut().unwrap().next = INVALID_ACTION_ANIMATION_ID;
            self.free = prev_len;
        }

        let pos = self.free;
        self.free = self.act(pos).next;
        self.act_mut(pos).next = u16::MAX;
        pos
    }

    fn free(&mut self, pos: u16) {
        let samplings = self.act_mut(pos).samplings.clone();
        for job in samplings.iter().cloned() {
            self.free(job);
        }

        *self.act_mut(pos) = Self::default();
        self.act_mut(pos).next = self.free;
        self.free = pos;
    }

    #[inline(always)]
    fn act(&self, pos: u16) -> &ActionData {
        assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked(pos as usize) };
    }

    #[inline(always)]
    fn act_mut(&mut self, pos: u16) -> &mut ActionData {
        assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked_mut(pos as usize) };
    }

    fn default() -> ActionData {
        ActionData {
            next: u16::MAX,
            id: NumID::default(),
            tmpl_id: TmplID::default(),
            samplings: ArrayVec::new(),
            phantoms: PhantomData,
        }
    }
}

#[derive(Debug)]
struct SamplingArena {
    num_joints: usize,
    num_soa_joints: usize,
    init_cap: usize,
    arena: Vec<SamplingData>,
    free: u16, // use animation_id as next pointer
}

impl SamplingArena {
    fn new(skeleton: &Skeleton, cap: usize) -> SamplingArena {
        let mut sa: SamplingArena = SamplingArena {
            num_joints: skeleton.num_joints(),
            num_soa_joints: skeleton.num_soa_joints(),
            init_cap: cap,
            arena: (0..cap).map(|_| Self::default()).collect(),
            free: 0,
        };
        for (idx, sd) in sa.arena.iter_mut().enumerate() {
            sd.animation_id = idx as u16 + 1;
        }
        sa.arena.last_mut().unwrap().animation_id = u16::MAX;
        sa
    }
}

impl SamplingArena {
    fn alloc(&mut self) -> u16 {
        if self.free == u16::MAX {
            let prev_len = self.arena.len() as u16;
            self.arena.reserve_exact(self.init_cap);
            for idx in prev_len..(prev_len + self.init_cap as u16) {
                let mut sd = Self::default();
                sd.animation_id = idx + 1;
                self.arena.push(sd);
            }
            self.arena.last_mut().unwrap().animation_id = INVALID_ACTION_ANIMATION_ID;
            self.free = prev_len;
        }

        let pos = self.free;
        self.free = self.samp(pos).animation_id;
        let num_joints = self.num_joints;
        let num_soa_joints = self.num_soa_joints;
        Self::init(self.samp_mut(pos), num_joints, num_soa_joints);
        pos
    }

    fn free(&mut self, pos: u16) {
        Self::clear(self.samp_mut(pos));
        self.samp_mut(pos).animation_id = self.free;
        self.free = pos;
    }

    #[inline(always)]
    fn samp(&self, pos: u16) -> &SamplingData {
        assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked(pos as usize) };
    }

    #[inline(always)]
    fn samp_mut(&mut self, pos: u16) -> &mut SamplingData {
        assert!((pos as usize) < self.arena.len());
        return unsafe { self.arena.get_unchecked_mut(pos as usize) };
    }

    fn default() -> SamplingData {
        SamplingData {
            animation_id: u16::MAX,
            animation_file: Symbol::default(),
            weight: [0.0; 2],
            ratio: [0.0; 2],
            sampling_job: SamplingJob::default(),
            weapon_motion: None,
        }
    }

    fn init(data: &mut SamplingData, num_joints: usize, num_soa_joints: usize) {
        data.animation_id = u16::MAX;

        if data.sampling_job.context().is_none() {
            let ctx = SamplingContext::new(num_joints);
            data.sampling_job.set_context(ctx);
        }

        if data.sampling_job.output().is_none() {
            let output = Rc::new(RefCell::new(vec![SoaTransform::default(); num_soa_joints]));
            data.sampling_job.set_output(output);
        }
    }

    fn clear(data: &mut SamplingData) {
        data.animation_id = u16::MAX;
        data.animation_file = Symbol::default();
        data.weight = [0.0; 2];
        data.ratio = [0.0; 2];
        data.sampling_job.clear_animation();
        data.sampling_job.set_ratio(0.0);
        data.weapon_motion = None;
    }
}

#[no_mangle]
pub extern "C" fn skeletal_animator_create(skel: Symbol) -> Return<*mut SkeletalAnimator> {
    let res: XResult<*mut SkeletalAnimator> = (|| {
        let animator = Box::new(SkeletalAnimator::new(
            skel,
            MAX_ACTION_STATE * 2,
            MAX_ACTION_STATE * MAX_ACTION_ANIMATION * 2,
        )?);
        Ok(Box::into_raw(animator))
    })();
    Return::from_result_with(res, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn skeletal_animator_destroy(animator: *mut SkeletalAnimator) {
    if !animator.is_null() {
        unsafe { drop(Box::from_raw(animator)) };
    }
}

#[no_mangle]
pub extern "C" fn skeletal_animator_skeleton_meta<'t>(animator: *mut SkeletalAnimator) -> Return<*const SkeletonMeta> {
    let res: XResult<*const SkeletonMeta> = (|| {
        let animator = as_animator(animator)?;
        let meta = Box::new(animator.skeleton_meta());
        Ok(Box::into_raw(meta) as *const _)
    })();
    Return::from_result_with(res, ptr::null())
}

#[no_mangle]
pub extern "C" fn skeletal_animator_update(
    animator: *mut SkeletalAnimator,
    states: &[Box<dyn StateActionAny>],
) -> Return<()> {
    let res: XResult<()> = (|| {
        let animator = as_animator(animator)?;
        animator.update(states)
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_animator_animate(animator: *mut SkeletalAnimator, ratio: f32) -> Return<()> {
    let res: XResult<()> = (|| {
        let animator = as_animator(animator)?;
        animator.animate(ratio)
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_animator_model_rest_poses<'t>(animator: *mut SkeletalAnimator) -> Return<&'t [Mat4]> {
    let res: XResult<&[Mat4]> = (|| {
        let animator = as_animator(animator)?;
        let ptr = animator.model_rest_poses().as_ptr();
        let len = animator.model_rest_poses().len();
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
    })();
    Return::from_result_with(res, &[])
}

#[no_mangle]
pub extern "C" fn skeletal_animator_model_poses<'t>(animator: *mut SkeletalAnimator) -> Return<&'t [Mat4]> {
    let res: XResult<&[Mat4]> = (|| {
        let animator = as_animator(animator)?;
        let ptr = animator.model_poses().as_slice().as_ptr();
        let len = animator.model_poses().len();
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
    })();
    Return::from_result_with(res, &[])
}

#[no_mangle]
pub extern "C" fn skeletal_animator_weapon_transforms<'t>(
    animator: *mut SkeletalAnimator,
) -> Return<&'t [WeaponTransform]> {
    let res: XResult<&[WeaponTransform]> = (|| {
        let animator = as_animator(animator)?;
        let ptr = animator.cs_weapon_transforms().as_ptr();
        let len = animator.cs_weapon_transforms().len();
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
    })();
    Return::from_result_with(res, &[])
}

fn as_animator<'t>(animator: *mut SkeletalAnimator) -> XResult<&'t mut SkeletalAnimator> {
    if animator.is_null() {
        return xres!(BadArgument; "animator=null");
    }
    Ok(unsafe { &mut *animator })
}

#[cfg(test)]
mod tests {
    use super::*;
    use critical_point_core::engine::LogicEngine;
    use critical_point_core::logic::{StateActionAnimation, StateActionEmpty};
    use critical_point_core::utils::{id, sb};

    const TEST_TMPL_PATH: &str = "../../test-tmp/test-template";
    const TEST_ASSET_PATH: &str = "../../test-tmp/test-asset";

    #[ctor::ctor]
    fn test_init_jolt_physics() {
        LogicEngine::initialize(TEST_TMPL_PATH, TEST_ASSET_PATH).unwrap();
    }

    fn list_action_frees(arena: &ActionArena) -> Vec<u16> {
        let mut frees = Vec::new();
        let mut pos = arena.free;
        while pos != u16::MAX {
            frees.push(pos);
            pos = arena.act(pos).next;
        }
        frees
    }

    fn list_sampling_frees(arena: &SamplingArena) -> Vec<u16> {
        let mut frees = Vec::new();
        let mut pos = arena.free;
        while pos != u16::MAX {
            frees.push(pos);
            pos = arena.samp(pos).animation_id;
        }
        frees
    }

    fn list_actions(animator: &SkeletalAnimator) -> Vec<&ActionData> {
        animator
            .actions
            .iter()
            .map(|act| animator.act_arena.act(*act))
            .collect::<Vec<_>>()
    }

    fn list_samplings<'t>(animator: &'t SkeletalAnimator, ad: &'t ActionData) -> Vec<&'t SamplingData> {
        ad.samplings
            .iter()
            .map(|samp| animator.samp_arena.samp(*samp))
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_action_arena() {
        let mut arena = ActionArena::new(3);
        assert_eq!(arena.arena.iter().map(|x| x.next).collect::<Vec<_>>(), vec![
            1,
            2,
            u16::MAX
        ]);
        assert_eq!(arena.free, 0);
        assert_eq!(list_action_frees(&arena), vec![0, 1, 2]);

        assert_eq!(arena.act(0).next, 1);
        assert_eq!(arena.act_mut(1).next, 2);
        assert_eq!(arena.act_mut(2).next, u16::MAX);

        assert_eq!(arena.alloc(), 0);
        assert_eq!(arena.act(0).next, u16::MAX);
        assert_eq!(arena.free, 1);

        assert_eq!(arena.alloc(), 1);
        assert_eq!(arena.act(1).next, u16::MAX);
        assert_eq!(arena.free, 2);

        assert_eq!(arena.alloc(), 2);
        assert_eq!(arena.act(2).next, u16::MAX);
        assert_eq!(arena.free, u16::MAX);
        assert_eq!(arena.arena.len(), 3);

        assert_eq!(arena.alloc(), 3);
        assert_eq!(arena.act(3).next, u16::MAX);
        assert_eq!(arena.free, 4);
        assert_eq!(arena.arena.len(), 6);
        assert_eq!(list_action_frees(&arena), vec![4, 5]);
        assert_eq!(arena.act(4).next, 5);
        assert_eq!(arena.act(5).next, u16::MAX);

        arena.free(0);
        assert_eq!(arena.act(0).next, 4);
        assert_eq!(list_action_frees(&arena), vec![0, 4, 5]);

        arena.free(1);
        assert_eq!(arena.act(1).next, 0);
        assert_eq!(list_action_frees(&arena), vec![1, 0, 4, 5]);

        assert_eq!(arena.alloc(), 1);
        assert_eq!(arena.act(1).next, u16::MAX);
        assert_eq!(list_action_frees(&arena), vec![0, 4, 5]);
        assert_eq!(arena.arena.len(), 6);
    }

    #[test]
    fn test_sampling_arena() {
        let skeleton = resource::load_skeleton(sb!("Girl.*")).unwrap();
        let mut arena = SamplingArena::new(&skeleton, 3);
        assert_eq!(arena.arena.iter().map(|x| x.animation_id).collect::<Vec<_>>(), vec![
            1,
            2,
            u16::MAX
        ]);
        assert_eq!(arena.free, 0);
        assert_eq!(list_sampling_frees(&arena), vec![0, 1, 2]);

        assert_eq!(arena.samp(0).animation_id, 1);
        assert_eq!(arena.samp_mut(1).animation_id, 2);
        assert_eq!(arena.samp_mut(2).animation_id, u16::MAX);

        assert_eq!(arena.alloc(), 0);
        assert_eq!(arena.samp(0).animation_id, u16::MAX);
        assert_eq!(arena.free, 1);

        assert_eq!(arena.alloc(), 1);
        assert_eq!(arena.samp(1).animation_id, u16::MAX);
        assert_eq!(arena.free, 2);

        assert_eq!(arena.alloc(), 2);
        assert_eq!(arena.samp(2).animation_id, u16::MAX);
        assert_eq!(arena.free, u16::MAX);
        assert_eq!(arena.arena.len(), 3);

        assert_eq!(arena.alloc(), 3);
        assert_eq!(arena.samp(3).animation_id, u16::MAX);
        assert_eq!(arena.free, 4);
        assert_eq!(arena.arena.len(), 6);
        assert_eq!(list_sampling_frees(&arena), vec![4, 5]);
        assert_eq!(arena.samp(4).animation_id, 5);
        assert_eq!(arena.samp(5).animation_id, u16::MAX);

        arena.free(0);
        assert_eq!(arena.samp(0).animation_id, 4);
        assert_eq!(list_sampling_frees(&arena), vec![0, 4, 5]);

        arena.free(1);
        assert_eq!(arena.samp(1).animation_id, 0);
        assert_eq!(list_sampling_frees(&arena), vec![1, 0, 4, 5]);

        assert_eq!(arena.alloc(), 1);
        assert_eq!(arena.samp(1).animation_id, u16::MAX);
        assert_eq!(list_sampling_frees(&arena), vec![0, 4, 5]);
        assert_eq!(arena.arena.len(), 6);
    }

    fn gen_states() -> Vec<Box<dyn StateActionAny>> {
        let mut states: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states[0].id = 21;
        states[0].tmpl_id = id!("Action.Empty/1");
        states[0].fade_in_weight = 1.0;
        states[0].animations[0].animation_id = 101;
        states[0].animations[0].files = Symbol::default();
        states[0].animations[0].ratio = 1.0;
        states[0].animations[0].weight = 1.0;
        states[1].id = 22;
        states[1].tmpl_id = id!("Action.Empty/2");
        states[1].fade_in_weight = 1.0;
        states[1].animations[0].animation_id = 102;
        states[1].animations[0].files = Symbol::default();
        states[1].animations[0].ratio = 1.0;
        states[1].animations[0].weight = 1.0;
        states
    }

    fn empty_states() -> Vec<Box<dyn StateActionAny>> {
        vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ]
    }

    fn assert_action_data(ad: &ActionData, id: u64, tmpl_id: TmplID, samplings: Vec<u16>) {
        assert_eq!(ad.id, id);
        assert_eq!(ad.tmpl_id, tmpl_id);
        assert_eq!(ad.samplings.as_slice(), samplings);
    }

    fn assert_sampling_data(
        sd: &SamplingData,
        animation_id: u16,
        animation_file: Symbol,
        ratio: [f32; 2],
        weight: [f32; 2],
    ) {
        assert_eq!(sd.animation_id, animation_id);
        assert_eq!(sd.animation_file, animation_file);
        assert_eq!(sd.weight, weight);
        assert_eq!(sd.ratio, ratio);
    }

    #[test]
    fn test_skeleton_animator_update_normal() {
        let mut animator = SkeletalAnimator::new(sb!("Girl.*"), 3, 4).unwrap();

        {
            let mut states = empty_states();
            states[0].tmpl_id = id!("Action.Idle/1");
            states[0].id = 21;
            states[0].fade_in_weight = 0.0;
            states[0].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Empty.*"), 101, false, 0.2, 0.0);
            states[1].tmpl_id = id!("Action.Idle/2");
            states[1].id = 22;
            states[1].fade_in_weight = 0.5;
            states[1].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Axe.*"), 102, true, 0.3, 0.1);
            animator.update(states.as_slice()).unwrap();

            assert_eq!(list_action_frees(&animator.act_arena), vec![2]);
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![2, 3]);

            let ads = list_actions(&animator);
            assert_eq!(ads.len(), 2);
            assert_action_data(&ads[0], 21, id!("Action.Idle/1"), vec![0]);
            assert_action_data(&ads[1], 22, id!("Action.Idle/2"), vec![1]);

            let sds = list_samplings(&animator, ads[0]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 101, sb!("Girl_Idle_Empty.*"), [0.2, 0.2], [0.0, 0.0]);

            let sds = list_samplings(&animator, ads[1]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 102, sb!("Girl_Idle_Axe.*"), [0.3, 0.3], [0.05, 0.05]);
        }

        {
            let mut states = empty_states();
            states[0].id = 21;
            states[0].tmpl_id = id!("Action.Idle/1");
            states[0].fade_in_weight = 0.0;
            states[0].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Empty.*"), 101, false, 0.3, 0.0);
            states[1].id = 22;
            states[1].tmpl_id = id!("Action.Idle/2");
            states[1].fade_in_weight = 1.0;
            states[1].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Axe.*"), 102, true, 0.4, 0.2);
            animator.update(states.as_slice()).unwrap();

            assert_eq!(list_action_frees(&animator.act_arena), vec![2]);
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![2, 3]);

            let ads = list_actions(&animator);
            assert_eq!(ads.len(), 2);
            assert_action_data(&ads[0], 21, id!("Action.Idle/1"), vec![0]);
            assert_action_data(&ads[1], 22, id!("Action.Idle/2"), vec![1]);

            let sds = list_samplings(&animator, ads[0]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 101, sb!("Girl_Idle_Empty.*"), [0.2, 0.3], [0.0, 0.0]);

            let sds = list_samplings(&animator, ads[1]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 102, sb!("Girl_Idle_Axe.*"), [0.3, 0.4], [0.05, 0.2]);
        }
    }

    #[test]
    fn test_skeleton_animator_update_change1() {
        let mut animator = SkeletalAnimator::new(sb!("Girl.*"), 3, 4).unwrap();

        {
            let mut states = gen_states();
            states[0].id = 21;
            states[0].tmpl_id = id!("Action.Empty/1");
            states[0].fade_in_weight = 1.0;
            states[0].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Empty.*"), 101, false, 0.2, 0.3);
            states[1].id = 22;
            states[1].tmpl_id = id!("Action.Empty/2");
            states[1].fade_in_weight = 1.0;
            states[1].animations[0] = StateActionAnimation::new(sb!("Girl_Run_Empty.*"), 102, false, 0.3, 0.7);
            animator.update(states.as_slice()).unwrap();

            assert_eq!(list_action_frees(&animator.act_arena), vec![2]);
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![2, 3]);
        }

        {
            let mut states = gen_states();
            states[0].id = 22;
            states[0].tmpl_id = id!("Action.Empty/2");
            states[0].fade_in_weight = 1.0;
            states[0].animations[0] = StateActionAnimation::new(sb!("Girl_Run_Empty.*"), 102, false, 0.4, 0.75);
            states[1].id = 23;
            states[1].tmpl_id = id!("Action.Empty/3");
            states[1].fade_in_weight = 1.0;
            states[1].animations[0] = StateActionAnimation::new(sb!("Girl_RunStop_L_Empty.*"), 31, false, 0.5, 1.0);
            animator.update(states.as_slice()).unwrap();

            assert!(list_action_frees(&animator.act_arena).is_empty());
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![3]);

            let ads = list_actions(&animator);
            assert_eq!(ads.len(), 3);
            assert_action_data(&ads[0], 22, id!("Action.Empty/2"), vec![1]);
            assert_action_data(&ads[1], 23, id!("Action.Empty/3"), vec![2]);
            assert_action_data(&ads[2], 21, id!("Action.Empty/1"), vec![0]);

            let sds = list_samplings(&animator, ads[0]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 102, sb!("Girl_Run_Empty.*"), [0.3, 0.4], [0.7, 0.75]);

            let sds = list_samplings(&animator, ads[1]);
            assert_sampling_data(&sds[0], 31, sb!("Girl_RunStop_L_Empty.*"), [0.5, 0.5], [0.0, 1.0]);
            assert_eq!(sds.len(), 1);

            let sds = list_samplings(&animator, ads[2]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 101, sb!("Girl_Idle_Empty.*"), [0.2, 0.2], [0.3, 0.0]);
        }

        {
            let mut states = gen_states();
            states[0].id = 23;
            states[0].tmpl_id = id!("Action.Empty/3");
            states[0].fade_in_weight = 1.0;
            states[0].animations[0] = StateActionAnimation::new(sb!("Girl_RunStop_L_Empty.*"), 31, false, 0.4, 1.0);
            states[1].id = 35;
            states[1].tmpl_id = id!("Action.Empty/4");
            states[1].fade_in_weight = 1.0;
            states[1].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Empty.*"), 17, false, 0.1, 0.1);
            animator.update(states.as_slice()).unwrap();

            assert_eq!(list_action_frees(&animator.act_arena), vec![0, 4, 5]);
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![0]);

            let ads = list_actions(&animator);
            assert_eq!(ads.len(), 3);
            assert_action_data(&ads[0], 23, id!("Action.Empty/3"), vec![2]);
            assert_action_data(&ads[1], 35, id!("Action.Empty/4"), vec![3]);
            assert_action_data(&ads[2], 22, id!("Action.Empty/2"), vec![1]);

            let sds = list_samplings(&animator, ads[0]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 31, sb!("Girl_RunStop_L_Empty.*"), [0.5, 0.4], [1.0, 1.0]);

            let sds = list_samplings(&animator, ads[1]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 17, sb!("Girl_Idle_Empty.*"), [0.1, 0.1], [0.0, 0.1]);

            let sds = list_samplings(&animator, ads[2]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 102, sb!("Girl_Run_Empty.*"), [0.4, 0.4], [0.75, 0.0]);
        }
    }

    #[test]
    fn test_skeleton_animator_update_change2() {
        let mut animator = SkeletalAnimator::new(sb!("Girl.*"), 3, 4).unwrap();

        {
            let mut states = gen_states();
            states[0].id = 21;
            states[0].tmpl_id = id!("Action.Empty/1");
            states[0].fade_in_weight = 1.0;
            states[0].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Empty.*"), 101, false, 0.2, 0.3);
            states[1].id = 22;
            states[1].tmpl_id = id!("Action.Empty/2");
            states[1].fade_in_weight = 1.0;
            states[1].animations[0] = StateActionAnimation::new(sb!("Girl_Run_Empty.*"), 102, false, 0.3, 0.7);
            animator.update(states.as_slice()).unwrap();

            assert_eq!(list_action_frees(&animator.act_arena), vec![2]);
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![2, 3]);
        }

        let mut states: Vec<Box<dyn StateActionAny>> = vec![Box::new(StateActionEmpty::default())];
        states[0].id = 22;
        states[0].tmpl_id = id!("Action.Empty/2");
        states[0].fade_in_weight = 1.0;
        states[0].animations[0] = StateActionAnimation::new(sb!("Girl_Run_Empty.*"), 102, false, 0.4, 0.75);
        states[0].animations[1] = StateActionAnimation::new(sb!("Girl_RunStop_L_Empty.*"), 31, false, 0.1, 0.2);
        {
            animator.update(states.as_slice()).unwrap();

            assert_eq!(list_action_frees(&animator.act_arena), vec![2]);
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![3]);

            let ads = list_actions(&animator);
            assert_eq!(ads.len(), 2);
            assert_action_data(&ads[0], 22, id!("Action.Empty/2"), vec![1, 2]);
            assert_action_data(&ads[1], 21, id!("Action.Empty/1"), vec![0]);

            let sds = list_samplings(&animator, ads[0]);
            assert_eq!(sds.len(), 2);
            assert_sampling_data(&sds[0], 102, sb!("Girl_Run_Empty.*"), [0.3, 0.4], [0.7, 0.75]);
            assert_sampling_data(&sds[1], 31, sb!("Girl_RunStop_L_Empty.*"), [0.1, 0.1], [0.0, 0.2]);

            let sds = list_samplings(&animator, ads[1]);
            assert_eq!(sds.len(), 1);
            assert_sampling_data(&sds[0], 101, sb!("Girl_Idle_Empty.*"), [0.2, 0.2], [0.3, 0.0]);
        }

        {
            animator.update(states.as_slice()).unwrap();

            assert_eq!(list_action_frees(&animator.act_arena), vec![0, 2]);
            assert_eq!(list_sampling_frees(&animator.samp_arena), vec![0, 3]);

            let ads = list_actions(&animator);
            assert_eq!(ads.len(), 1);
            assert_action_data(&ads[0], 22, id!("Action.Empty/2"), vec![1, 2]);

            let sds = list_samplings(&animator, ads[0]);
            assert_eq!(sds.len(), 2);
            assert_sampling_data(&sds[0], 102, sb!("Girl_Run_Empty.*"), [0.4, 0.4], [0.75, 0.75]);
            assert_sampling_data(&sds[1], 31, sb!("Girl_RunStop_L_Empty.*"), [0.1, 0.1], [0.2, 0.2]);
        }
    }

    #[test]
    fn test_skeleton_animator_weapon_transforms() {
        let mut animator = SkeletalAnimator::new(sb!("Girl.*"), 3, 4).unwrap();

        let mut states = empty_states();
        states[0].tmpl_id = id!("Action.Idle/1");
        states[0].id = 21;
        states[0].fade_in_weight = 0.5;
        states[0].animations[0] = StateActionAnimation::new(sb!("Girl_Idle_Axe.*"), 101, true, 0.8, 0.6);
        states[1].tmpl_id = id!("Action.Attack/2");
        states[1].id = 22;
        states[1].fade_in_weight = 0.5;
        states[1].animations[0] = StateActionAnimation::new(sb!("Girl_Attack_01A.*"), 102, true, 0.1, 0.4);
        animator.update(states.as_slice()).unwrap();
        animator.animate(0.0).unwrap();

        assert_eq!(animator.weapon_transforms.len(), 1);
        assert_eq!(animator.weapon_transforms[0].name, "Axe");
        assert_eq!(animator.weapon_transforms[0].weight, 1.0);

        let idle_tracks = resource::load_weapon_motion(sb!("Girl_Idle_Axe.*")).unwrap();
        let idle_track = idle_tracks.get(0).unwrap();
        let (idle_pos, idle_rot) = idle_track.sample(0.8).unwrap();
        let attack_tracks = resource::load_weapon_motion(sb!("Girl_Attack_01A.*")).unwrap();
        let attack_track = attack_tracks.get(0).unwrap();
        let (attack_pos, attack_rot) = attack_track.sample(0.1).unwrap();

        let pos = (idle_pos * 0.6 * 0.5 + attack_pos * 0.4 * 0.5) / 0.5;
        assert!(animator.weapon_transforms[0].position.abs_diff_eq(pos, 1e-6));

        let rot = (idle_rot * 0.6 * 0.5 + attack_rot * 0.4 * 0.5).normalize();
        assert!(animator.weapon_transforms[0].rotation.abs_diff_eq(rot, 1e-6));
    }
}
