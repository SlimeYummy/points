use arrayvec::ArrayVec;
use glam::Vec4;
use glam_ext::{Mat4, Transform3A};
use ozz_animation_rs::{
    Animation, BlendingJob, BlendingLayer, LocalToModelJob, SamplingContext, SamplingJob, Skeleton, SoaTransform,
    ozz_rc_buf,
};
use std::cell::RefCell;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

use crate::animation::hit_motion::{HitMotion, HitMotionSampler};
use crate::animation::joint_weights::JointWeightsTable;
use crate::animation::utils::{matrices_to_transforms, rest_poses_to_model_transforms};
use crate::animation::weapon_control::{WeaponControl, WeaponStates};
use crate::asset::AssetLoader;
use crate::consts::{EMPTY_JOINT_WEIGHT_INDEX, INVALID_ACTION_ID, INVALID_ANIMATION_ID, MAX_ANIMATION_LAYERS};
use crate::logic::{StateActionAnimation, StateActionAny};
use crate::utils::{
    HistoryQueue, SmallVec, Symbol, TmplID, XResult, ifelse, xdbg_assert, xdbg_unreachable, xfrom, xres,
};

type AnimatorBlendingJob =
    BlendingJob<Rc<Skeleton>, Rc<RefCell<Vec<SoaTransform>>>, Rc<RefCell<Vec<SoaTransform>>>, Rc<Vec<Vec4>>>;
type AnimatorBlendingLayer = BlendingLayer<Rc<RefCell<Vec<SoaTransform>>>, Rc<Vec<Vec4>>>;

const DEFAULT_ACTION_STATE_LEN: usize = 6;

// Append more fields for StateActionAny
#[derive(Debug, Clone, Copy)]
pub struct StateActionEx<'t> {
    pub state: &'t dyn StateActionAny,
    pub fade_weight: f32,
    pub no_fade_weight: f32,
}

impl Deref for StateActionEx<'_> {
    type Target = dyn StateActionAny;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'t> StateActionEx<'t> {
    #[inline]
    pub fn new(state: &'t Box<dyn StateActionAny>, fade_weight: f32, no_fade_weight: f32) -> StateActionEx<'t> {
        xdbg_assert!(fade_weight <= no_fade_weight);
        StateActionEx {
            state: state.as_ref(),
            fade_weight,
            no_fade_weight,
        }
    }

    #[inline]
    pub fn prev_fade_weight(&self) -> f32 {
        self.no_fade_weight - self.fade_weight
    }
}

#[derive(Debug)]
pub struct Animator {
    skeleton: Rc<Skeleton>,
    joint_weights_table: Option<Rc<JointWeightsTable>>,
    blending_job: AnimatorBlendingJob,
    l2m_job: LocalToModelJob,
    weapon_states: WeaponStates,
    action_queue: HistoryQueue<ActionData>,
    sampling_arena: SamplingArena,
    tmp_blending_output: Option<Rc<RefCell<Vec<SoaTransform>>>>,
    model_transforms: Vec<Transform3A>,
}

impl Animator {
    pub fn new(
        skeleton: Rc<Skeleton>,
        joint_weights_table: Option<Rc<JointWeightsTable>>,
        action_cap: usize,
        sampling_cap: usize,
    ) -> XResult<Animator> {
        let mut model_transforms = vec![Transform3A::ZERO; skeleton.num_joints()];
        rest_poses_to_model_transforms(&skeleton, &mut model_transforms)?;

        let mut animator: Animator = Animator {
            skeleton: skeleton.clone(),
            joint_weights_table,
            blending_job: AnimatorBlendingJob::default(),
            l2m_job: LocalToModelJob::default(),
            weapon_states: WeaponStates::default(),
            action_queue: HistoryQueue::with_capacity(action_cap.max(1)),
            sampling_arena: SamplingArena::new(&skeleton, sampling_cap.max(1)),
            tmp_blending_output: None,
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

    // Prepare and verify action states
    #[inline]
    fn prepare_states<'t>(
        states: &'t [Box<dyn StateActionAny>],
    ) -> XResult<SmallVec<[StateActionEx<'t>; DEFAULT_ACTION_STATE_LEN]>> {
        let mut new_states = SmallVec::new();
        let mut unused_weight = 1.0;

        for (idx, state) in states.iter().enumerate().rev() {
            if state.previous_frame() {
                let next_idx = idx + 1;

                // Check previous frame
                #[cfg(debug_assertions)]
                {
                    xdbg_assert!(
                        states.get(next_idx).is_some(),
                        "action state previous_frame, no next, act_idx={}, states={:?}",
                        idx,
                        states
                    );
                    let next_state = &states[next_idx];
                    xdbg_assert!(
                        state.id == next_state.id,
                        "action state previous_frame, id miss match, act_idx={}, states={:?}",
                        idx,
                        states
                    );
                    xdbg_assert!(
                        state.tmpl_id == next_state.tmpl_id,
                        "action state previous_frame, tmpl_id miss match, act_idx={}, states={:?}",
                        idx,
                        states
                    );
                    xdbg_assert!(
                        state.fade_joint_weights_index == next_state.fade_joint_weights_index,
                        "action state previous_frame, fade joint weights miss match, act_idx={}, states={:?}",
                        idx,
                        states
                    );
                    xdbg_assert!(
                        next_state.previous_frame() == false,
                        "action state previous_frame, duplicate previous_frame, act_idx={}, states={:?}",
                        idx,
                        states
                    );
                }

                continue;
            }

            if state.fade_joint_weights_index == EMPTY_JOINT_WEIGHT_INDEX {
                let fade_weight = (unused_weight * state.fade_in_weight).clamp(0.0, 1.0);
                new_states.push(StateActionEx::new(state, fade_weight, fade_weight));
                unused_weight = (unused_weight - fade_weight).max(0.0);
            }
            else {
                let fade_weight = (unused_weight * state.fade_in_weight).clamp(0.0, 1.0);
                new_states.push(StateActionEx::new(state, fade_weight, unused_weight));
                unused_weight = 1.0;

                xdbg_assert!(
                    idx != 0,
                    "action state fade_joint_weights_index, bad position, act_idx=0, states={:?}",
                    states
                );
            }

            // Check reuse previous
            #[cfg(debug_assertions)]
            {
                let mut base_idx = 0;
                let mut reuse_layer_cnt = 0;
                for (anim_idx, anim_state) in state.animations.iter().enumerate() {
                    if !anim_state.reuse_previous() {
                        base_idx = anim_idx;
                        reuse_layer_cnt = 0;
                    }
                    else {
                        xdbg_assert!(
                            anim_idx != 0,
                            "animation state reuse_previous, no previous, act_idx={}, anim_idx={}, states={:?}",
                            idx,
                            anim_idx,
                            states
                        );

                        let base_state = &state.animations[base_idx];
                        xdbg_assert!(
                            base_state.animation_id == anim_state.animation_id,
                            "animation state reuse_previous, id mismatch, act_idx={}, anim_idx={}, states={:?}",
                            idx,
                            anim_idx,
                            states
                        );
                        xdbg_assert!(
                            base_state.files == anim_state.files,
                            "animation state reuse_previous, files mismatch, act_idx={}, anim_idx={}, states={:?}",
                            idx,
                            anim_idx,
                            states
                        );
                        xdbg_assert!(
                            base_state.ratio == anim_state.ratio,
                            "animation state reuse_previous, ratio mismatch, act_idx={}, anim_idx={}, states={:?}",
                            idx,
                            anim_idx,
                            states
                        );

                        reuse_layer_cnt += 1;
                        if reuse_layer_cnt > MAX_ANIMATION_LAYERS {
                            xdbg_assert!(
                                base_state.ratio == anim_state.ratio,
                                "animation state reuse_previous, too many, act_idx={}, anim_idx={}, states={:?}",
                                idx,
                                anim_idx,
                                states
                            );
                        }
                    }
                }
            }

            // Check unique id
            #[cfg(debug_assertions)]
            {
                xdbg_assert!(
                    states
                        .iter()
                        .filter(|s| !s.previous_frame() && s.id == state.id)
                        .count()
                        == 1,
                    "action state duplicate id, act_idx={}, states={:?}",
                    idx,
                    states
                );

                for (anim_idx, anim_state) in state.animations.iter().enumerate() {
                    if anim_state.reuse_previous() {
                        continue;
                    }
                    xdbg_assert!(
                        state
                            .animations
                            .iter()
                            .filter(|a| !a.reuse_previous() && a.animation_id == anim_state.animation_id)
                            .count()
                            == 1,
                        "animation state duplicate animation_id, act_idx={}, anim_idx={}, states={:?}",
                        idx,
                        anim_idx,
                        states
                    );
                }

                for (anim_idx, anim_state) in state.animations.iter().enumerate() {
                    xdbg_assert!(
                        state
                            .animations
                            .iter()
                            .filter(|a| {
                                a.animation_id == anim_state.animation_id
                                    && a.reuse_layer_id == anim_state.reuse_layer_id
                            })
                            .count()
                            == 1,
                        "animation state duplicate reuse_layer_id, act_idx={}, anim_idx={}, states={:?}",
                        idx,
                        anim_idx,
                        states
                    );
                }
            }
        }

        new_states.reverse();
        if new_states.is_empty() {
            return xres!(LogicBadState; "states empty");
        }
        Ok(new_states)
    }

    pub fn update(&mut self, frame: u32, states: &[Box<dyn StateActionAny>], loader: &mut AssetLoader) -> XResult<()> {
        let states = Self::prepare_states(states)?;

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
            ad.update(&mut self.sampling_arena, frame, *state, &self.skeleton, loader)?;
        }

        // 3. try reuse actions
        for idx in self.action_queue.len()..states.len() {
            let state = &states[idx];
            let reused = self
                .action_queue
                .enqueue_reuse(|ad| {
                    if ad.tmpl_id == state.tmpl_id {
                        ad.reuse(&mut self.sampling_arena, frame, *state, &self.skeleton, loader)?;
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
            ad.init(&mut self.sampling_arena, frame, *state, &self.skeleton, loader)?;
            self.action_queue.enqueue_new(ad);
        }
        Ok(())
    }

    pub fn restore(&mut self, frame: u32, states: &[Box<dyn StateActionAny>]) -> XResult<()> {
        let states = Self::prepare_states(states)?;

        let mut state_iter = states.iter();
        self.action_queue.restore_when(|ad| {
            if ad.frame < frame {
                return Ok(-1);
            }
            if let Some(state) = state_iter.next() {
                if ad.id == state.id {
                    ad.restore(&mut self.sampling_arena, frame, *state)?;
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
        log::debug!(target:"#anim_debug.player", "--------------------");

        self.blending_job.layers_mut().clear();
        self.blending_job.additive_layers_mut().clear();
        self.weapon_states.clear();

        for ad in self.action_queue.iter_mut() {
            // Handle fade in with joint weights
            if ad.prev_fade_joint_weights_index != EMPTY_JOINT_WEIGHT_INDEX {
                Self::handle_previous_fade_blending(
                    &self.skeleton,
                    &self.joint_weights_table,
                    ad,
                    &mut self.blending_job,
                    &mut self.tmp_blending_output,
                )?;
            }

            ad.animate(
                &mut self.sampling_arena,
                &mut self.blending_job,
                &self.joint_weights_table,
            )?;
        }

        self.blending_job.run().map_err(xfrom!())?;
        self.l2m_job.run().map_err(xfrom!())?;
        matrices_to_transforms(
            self.l2m_job.output().unwrap().borrow().as_slice(),
            &mut self.model_transforms,
        )?;

        if let Some(current_action) = self.action_queue.last_mut() {
            current_action.animate_weapons_and_hits(
                &mut self.sampling_arena,
                &self.model_transforms,
                &mut self.weapon_states,
            )?;
        }
        Ok(())
    }

    #[inline]
    fn handle_previous_fade_blending(
        skeleton: &Skeleton,
        joint_weights_table: &Option<Rc<JointWeightsTable>>,
        ad: &ActionData,
        blending_job: &mut AnimatorBlendingJob,
        tmp_blending_output: &mut Option<Rc<RefCell<Vec<SoaTransform>>>>,
    ) -> XResult<()> {
        let mut joint_weights = None;
        if let Some(joint_weights_table) = joint_weights_table {
            joint_weights = joint_weights_table
                .get_weights(ad.prev_fade_joint_weights_index)
                .cloned();
        }
        xdbg_assert!(
            joint_weights.is_some(),
            "joint weights not found, id={}, tmpl_id={}, table={}, index={}",
            ad.id,
            ad.tmpl_id,
            joint_weights_table.is_some(),
            ad.prev_fade_joint_weights_index
        );

        if blending_job.layers().is_empty() && blending_job.additive_layers().is_empty() {
            // do nothing
            xdbg_unreachable!("Unreachable, empty layers");
            return Ok(());
        }

        // Quick path, we simply setup weight and joint weights.
        #[rustfmt::skip]
        if blending_job.layers().len() + blending_job.additive_layers().len() <= 4
            && blending_job.layers().iter().all(|l| l.joint_weights.is_none())
            && blending_job.additive_layers().iter().all(|l| l.joint_weights.is_none())
        {
            for layer in blending_job.layers_mut() {
                layer.weight *= ad.prev_fade_weight;
                layer.joint_weights = joint_weights.clone();
            }
            for layer in blending_job.additive_layers_mut() {
                layer.weight *= ad.prev_fade_weight;
                layer.joint_weights = joint_weights.clone();
            }

            log::debug!(
                target:"#anim_debug.animator",
                "prev_fade.update, fade_weight={}, fade_jwt_idx={}",
                ad.prev_fade_weight, ad.prev_fade_joint_weights_index
            );
            return Ok(());
        }

        // Blend previous animation, use blending job output as new layer.
        if tmp_blending_output.is_none() {
            *tmp_blending_output = Some(Rc::new(RefCell::new(vec![
                SoaTransform::default();
                skeleton.num_soa_joints()
            ])));
        }
        let old_output = blending_job.replace_output(tmp_blending_output.take().unwrap());
        blending_job.run().map_err(xfrom!())?;

        *tmp_blending_output = blending_job.replace_output(old_output.unwrap());
        blending_job.layers_mut().push(BlendingLayer {
            transform: tmp_blending_output.clone().unwrap(),
            weight: ad.prev_fade_weight,
            joint_weights,
        });
        blending_job.layers_mut().clear();
        blending_job.additive_layers_mut().clear();

        log::debug!(
            target:"#anim_debug.animator",
            "prev_fade.blend, fade_weight={}, fade_jwt_idx={}",
            ad.prev_fade_weight, ad.prev_fade_joint_weights_index
        );
        Ok(())
    }

    #[inline]
    pub fn model_transforms(&self) -> &[Transform3A] {
        &self.model_transforms
    }

    #[inline]
    pub fn weapon_states(&self) -> &WeaponStates {
        &self.weapon_states
    }

    #[inline]
    pub fn action_animation_id(&self) -> (u32, u16) {
        match self.action_queue.last() {
            Some(ad) => match ad.current_sampling_data(&self.sampling_arena) {
                Some(sd) => (ad.id, sd.animation_id),
                None => (ad.id, INVALID_ANIMATION_ID),
            },
            None => (INVALID_ACTION_ID, INVALID_ANIMATION_ID),
        }
    }

    #[inline]
    pub(crate) fn hit_motion_sampler(&self) -> Option<&HitMotionSampler> {
        if let Some(ad) = self.action_queue.last() {
            if let Some(sd) = ad.current_sampling_data(&self.sampling_arena) {
                return sd.hit_motion_sampler.as_ref();
            }
        }
        None
    }

    #[inline]
    pub fn joint_weights_table(&self) -> Option<Rc<JointWeightsTable>> {
        self.joint_weights_table.clone()
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
    id: u32,
    tmpl_id: TmplID,
    frame: u32,
    job_current: u32,
    job_past: u32,
    job_future: u32,

    prev_fade_joint_weights_index: u8,
    prev_fade_weight: f32,
}

impl Default for ActionData {
    fn default() -> ActionData {
        ActionData {
            id: INVALID_ACTION_ID,
            tmpl_id: TmplID::default(),
            frame: 0,
            job_current: u32::MAX,
            job_past: u32::MAX,
            job_future: u32::MAX,

            prev_fade_joint_weights_index: EMPTY_JOINT_WEIGHT_INDEX,
            prev_fade_weight: 0.0,
        }
    }
}

impl ActionData {
    fn load_resources(
        loader: &mut AssetLoader,
        anim_state: &StateActionAnimation,
    ) -> XResult<(Rc<Animation>, Option<Rc<WeaponControl>>, Option<Rc<HitMotion>>)> {
        let animation = loader.load_animation(anim_state.files)?;
        let weapon_control = match anim_state.weapon_control() {
            true => Some(loader.load_weapon_control(anim_state.files)?),
            false => None,
        };
        let hit_motion = match anim_state.hit_motion() {
            true => Some(loader.load_hit_motion(anim_state.files)?),
            false => None,
        };
        Ok((animation, weapon_control, hit_motion))
    }

    // Return reuse previous states count (aka how many animations state should be skipped)
    fn fill_sampling_data_layers(sd: &mut SamplingData, state: StateActionEx<'_>, base_idx: usize) -> usize {
        sd.layers.clear();

        let base_state = &state.animations[base_idx];
        sd.layers.push(SamplingDataLayer {
            weight: base_state.weight * ifelse!(base_state.skip_fade_in(), state.no_fade_weight, state.fade_weight),
            joint_weights_index: base_state.joint_weights_index,
        });

        let mut idx = base_idx + 1;
        let mut layer_cnt = 1;
        while idx < state.animations.len() {
            let reused_state = &state.animations[idx];
            if !reused_state.reuse_previous() {
                break;
            }

            idx += 1;
            layer_cnt = idx - base_idx;
            if layer_cnt > MAX_ANIMATION_LAYERS {
                continue; // skip more reused_state here, we checked in prepare_states()
            }

            sd.layers.push(SamplingDataLayer {
                weight: reused_state.weight
                    * ifelse!(reused_state.skip_fade_in(), state.no_fade_weight, state.fade_weight),
                joint_weights_index: reused_state.joint_weights_index,
            });
        }
        layer_cnt - 1
    }

    #[inline]
    fn fill_prev_fade(&mut self, state: StateActionEx<'_>) {
        self.prev_fade_joint_weights_index = state.fade_joint_weights_index;
        if state.fade_joint_weights_index != EMPTY_JOINT_WEIGHT_INDEX {
            self.prev_fade_weight = state.prev_fade_weight();
        }
        else {
            self.prev_fade_weight = 0.0;
        }
    }

    fn init(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: StateActionEx<'_>,
        skeleton: &Skeleton,
        loader: &mut AssetLoader,
    ) -> XResult<()> {
        if state.animations.is_empty() {
            return xres!(LogicBadState; "animations empty");
        }

        self.id = state.id;
        self.tmpl_id = state.tmpl_id;
        self.frame = frame;
        self.fill_prev_fade(state);

        let mut pnext: *mut u32 = &mut self.job_current;
        let mut idx = 0;
        while idx < state.animations.len() {
            let anim_state = &state.animations[idx];
            let pos = arena.alloc_and_reptr(&mut pnext);
            let sd = arena.get_mut(pos);
            let (animation, weapon_control, hit_motion) = Self::load_resources(loader, anim_state)?;

            sd.animation_id = anim_state.animation_id;
            sd.animation_file = anim_state.files;
            sd.additive_blending = anim_state.additive_blending();
            sd.layers.clear();
            sd.sampling_job.set_animation(animation);
            sd.weapon_control = weapon_control;
            if let Some(hit_motion) = hit_motion {
                sd.hit_motion_sampler = Some(HitMotionSampler::new(hit_motion, skeleton)?);
            }
            sd.frame = frame;
            sd.sampling_job.set_ratio(anim_state.ratio);
            log::debug!("ActionData::init() anim={:?}", anim_state.files);

            idx += 1 + Self::fill_sampling_data_layers(sd, state, idx);

            unsafe { *pnext = pos };
            pnext = &mut sd.next;
        }
        self.job_past = self.job_current;
        Ok(())
    }

    fn reuse(
        &mut self,
        arena: &mut SamplingArena,
        frame: u32,
        state: StateActionEx<'_>,
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
        state: StateActionEx<'_>,
        skeleton: &Skeleton,
        loader: &mut AssetLoader,
    ) -> XResult<()> {
        if state.animations.is_empty() {
            return xres!(LogicBadState; "animations empty");
        }
        self.frame = frame;
        self.fill_prev_fade(state);

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

        // 2. verify & update using jobs
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
            sd.sampling_job.set_ratio(anim_state.ratio);

            state_idx += 1 + Self::fill_sampling_data_layers(sd, state, state_idx);

            last = iter;
            iter = sd.next;
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
                sd.sampling_job.set_ratio(anim_state.ratio);
                log::debug!("ActionData::update() reuse anim={:?}", anim_state.files);

                state_idx += 1 + Self::fill_sampling_data_layers(sd, state, state_idx);

                last = self.job_future;
                self.job_future = sd.next;
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
            let (animation, weapon_control, hit_motion) = Self::load_resources(loader, anim_state)?;
            sd.animation_id = anim_state.animation_id;
            sd.animation_file = anim_state.files;
            sd.additive_blending = anim_state.additive_blending();
            sd.layers.clear();
            sd.sampling_job.set_animation(animation);
            sd.weapon_control = weapon_control;
            if let Some(hit_motion) = hit_motion {
                sd.hit_motion_sampler = Some(HitMotionSampler::new(hit_motion, skeleton)?);
            }
            sd.frame = frame;
            sd.sampling_job.set_ratio(anim_state.ratio);
            log::debug!("ActionData::update() enqueue anim={:?}", anim_state.files);

            state_idx += 1 + Self::fill_sampling_data_layers(sd, state, state_idx);

            unsafe { *pnext = pos };
            pnext = &mut sd.next;
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

    fn restore(&mut self, arena: &mut SamplingArena, frame: u32, state: StateActionEx) -> XResult<()> {
        debug_assert!(self.job_past != u32::MAX);

        if state.animations.is_empty() {
            return xres!(LogicBadState; "animations empty");
        }
        self.frame = frame;
        self.fill_prev_fade(state);

        // Find new job list header
        let mut iter = self.job_past;
        while iter != u32::MAX {
            let sd = arena.get_ref(iter);
            if sd.frame >= frame {
                break;
            }
            iter = sd.next;
        }
        self.job_current = iter;

        // Reinit sampling data for new job list
        let mut idx = 0;
        while idx < state.animations.len() {
            if iter == u32::MAX {
                return xres!(LogicBadState; "animations iter");
            }

            let anim_state = &state.animations[idx];
            let sd: &mut SamplingData = arena.get_mut(iter);
            if sd.animation_id != anim_state.animation_id {
                return xres!(LogicBadState; "animation id");
            }
            sd.frame = frame;
            sd.sampling_job.set_ratio(anim_state.ratio);

            idx += 1 + Self::fill_sampling_data_layers(sd, state, idx);

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
        blending_job: &mut AnimatorBlendingJob,
        joint_weights_table: &Option<Rc<JointWeightsTable>>,
    ) -> XResult<()> {
        let mut iter = self.job_current;
        while iter != self.job_future {
            let sd = arena.get_mut(iter);
            iter = sd.next;

            if sd.layers.is_empty() {
                continue;
            }

            if Self::need_run_sampling_job(&sd.sampling_job) {
                sd.sampling_job.run().map_err(xfrom!())?;
            }

            for sd_layer in &sd.layers {
                let mut layer_joint_weights = None;
                if sd_layer.joint_weights_index != EMPTY_JOINT_WEIGHT_INDEX {
                    if let Some(joint_weights_table) = joint_weights_table {
                        layer_joint_weights = joint_weights_table.get_weights(sd_layer.joint_weights_index).cloned();
                    }
                    xdbg_assert!(
                        layer_joint_weights.is_some(),
                        "joint weights not found, table={}, index={}",
                        joint_weights_table.is_some(),
                        sd_layer.joint_weights_index
                    );
                }

                log::debug!(
                    target:"#anim_debug.animator",
                    "anim: {}, ratio: {}, additive: {}, weight: {}, jwt_idx: {}",
                    sd.animation_file,
                    sd.sampling_job.ratio(),
                    sd.additive_blending,
                    sd_layer.weight,
                    sd_layer.joint_weights_index
                );

                let blending_layer = AnimatorBlendingLayer {
                    transform: sd.sampling_job.output().unwrap().clone(),
                    weight: sd_layer.weight,
                    joint_weights: layer_joint_weights,
                };

                match sd.additive_blending {
                    true => blending_job.additive_layers_mut().push(blending_layer),
                    false => blending_job.layers_mut().push(blending_layer),
                }
            }
        }
        Ok(())
    }

    // If the ratio and animation_id are unchanged, we can skip sampling_job.run() to improve performance
    fn need_run_sampling_job(sampling_job: &SamplingJob) -> bool {
        let (prev_ratio, prev_anim_id) = match sampling_job.context() {
            Some(ctx) => (ctx.ratio(), ctx.animation_id()),
            None => (0.0, 0),
        };
        let curr_ratio = sampling_job.ratio();
        let curr_anim_id = match sampling_job.animation() {
            Some(anim) => anim.as_id(),
            None => 0,
        };
        (prev_ratio != curr_ratio) || (prev_anim_id != curr_anim_id)
    }

    fn animate_weapons_and_hits(
        &mut self,
        arena: &mut SamplingArena,
        model_transforms: &[Transform3A],
        weapon_states: &mut WeaponStates,
    ) -> XResult<()> {
        if self.job_current == self.job_future {
            return Ok(());
        }

        let sd: &mut SamplingData = arena.get_mut(self.job_current);
        if let Some(weapon_control) = &sd.weapon_control {
            *weapon_states = weapon_control.sample(sd.sampling_job.ratio())?;
        }

        let hit_motion_sampler = match &mut sd.hit_motion_sampler {
            Some(hit_motion_sampler) => hit_motion_sampler,
            None => return Ok(()),
        };

        let animation = sd.sampling_job.animation().unwrap();
        let ratio = sd.sampling_job.ratio();
        let time = animation.duration() * ratio;

        hit_motion_sampler.sample(time, model_transforms);
        Ok(())
    }

    #[inline]
    fn current_sampling_data<'a, 'b>(&'a self, arena: &'b SamplingArena) -> Option<&'b SamplingData> {
        if self.job_current == self.job_future {
            None
        }
        else {
            Some(arena.get_ref(self.job_current))
        }
    }
}

#[derive(Debug)]
struct SamplingData {
    next: u32,
    animation_id: u16,
    frame: u32,
    animation_file: Symbol,
    additive_blending: bool,
    layers: ArrayVec<SamplingDataLayer, MAX_ANIMATION_LAYERS>,
    sampling_job: SamplingJob,
    weapon_control: Option<Rc<WeaponControl>>,
    hit_motion_sampler: Option<HitMotionSampler>,
}

#[derive(Debug, Clone, Copy)]
struct SamplingDataLayer {
    weight: f32,
    joint_weights_index: u8,
}

#[derive(Debug)]
struct SamplingArena {
    num_joints: usize,
    num_soa_joints: usize,
    init_cap: usize,
    arena: Vec<SamplingData>,
    free: u32,
}

impl SamplingArena {
    fn new(skeleton: &Skeleton, cap: usize) -> SamplingArena {
        let mut sa: SamplingArena = SamplingArena {
            num_joints: skeleton.num_joints(),
            num_soa_joints: skeleton.num_soa_joints(),
            init_cap: cap,
            arena: (0..cap).map(|_| Self::default_sd()).collect(),
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
                let mut sd = Self::default_sd();
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
        let num_joints = self.num_joints;
        let num_soa_joints = self.num_soa_joints;
        Self::init_sd(self.get_mut(pos), num_joints, num_soa_joints);
        pos
    }

    fn free(&mut self, pos: u32) {
        let sd = self.get_mut(pos);
        SamplingArena::clear_sd(sd);

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

    fn default_sd() -> SamplingData {
        SamplingData {
            next: u32::MAX,
            animation_id: INVALID_ANIMATION_ID,
            frame: 0,
            animation_file: Symbol::default(),
            additive_blending: false,
            layers: ArrayVec::new(),
            sampling_job: SamplingJob::default(),
            weapon_control: None,
            hit_motion_sampler: None,
        }
    }

    fn init_sd(data: &mut SamplingData, num_joints: usize, num_soa_joints: usize) {
        data.animation_id = u16::MAX;

        if data.sampling_job.context().is_none() {
            let ctx = SamplingContext::new(num_joints);
            data.sampling_job.set_context(ctx);
        }

        if data.sampling_job.output().is_none() {
            let output = Rc::new(RefCell::new(vec![SoaTransform::default(); num_soa_joints]));
            data.sampling_job.set_output(output);
        }

        data.next = u32::MAX;
    }

    fn clear_sd(data: &mut SamplingData) {
        data.next = u32::MAX;
        data.animation_id = INVALID_ANIMATION_ID;
        data.frame = 0;
        data.animation_file = Symbol::default();
        data.additive_blending = false;
        data.layers.clear();
        data.sampling_job.clear_animation();
        data.sampling_job.set_ratio(0.0);
        data.weapon_control = None;
        data.hit_motion_sampler = None;
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
        let skeleton = asset_loader.load_skeleton(sb!("Girl/Girl.*")).unwrap();
        (asset_loader, skeleton)
    }

    #[test]
    fn test_sampling_arena() {
        let (_asset_loader, skeleton) = prepare_resource();
        let mut arena = SamplingArena::new(&skeleton, 3);
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
            u32::MAX,
            u32::MAX,
            u32::MAX,
            u32::MAX,
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
        let mut arena = SamplingArena::new(&skeleton, 3);

        let state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
        let state_ex = StateActionEx::new(&state, state.fade_in_weight, state.fade_in_weight);
        let mut ad: ActionData = ActionData::default();
        let res = ad.init(&mut arena, 300, state_ex, &skeleton, &mut asset_loader);
        assert!(res.is_err());

        {
            let mut state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state.id = 12345;
            state.tmpl_id = id!("Action.Empty");
            state.fade_in_weight = 0.7;

            let mut anim0 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 101, 0.1, 0.7);
            anim0.set_joint_weights_index(5);
            state.animations.push(anim0);

            let mut anim_reuse = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 101, 0.1, 0.1);
            anim_reuse.reuse_layer_id = 1;
            state.animations.push(anim_reuse);

            let mut anim1 = StateActionAnimation::new(sb!("Girl/Run_Empty.*"), 102, 0.2, 0.3);
            anim1.set_additive_blending(true);
            state.animations.push(anim1);

            let state_ex = StateActionEx::new(&state, state.fade_in_weight, state.fade_in_weight);
            let mut ad: ActionData = ActionData::default();
            ad.init(&mut arena, 120, state_ex, &skeleton, &mut asset_loader)
                .unwrap();
            assert_eq!(ad.id, 12345);
            assert_eq!(ad.tmpl_id, id!("Action.Empty"));
            assert_eq!(ad.frame, 120);

            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);

            assert_eq!(current[0].animation_id, 101);
            assert_eq!(current[0].frame, 120);
            assert_eq!(current[0].animation_file, "Girl/Idle_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.1);
            assert_eq!(current[0].additive_blending, false);
            assert_eq!(current[0].layers.len(), 2);
            assert_eq!(current[0].layers[0].weight, 0.7 * 0.7);
            assert_eq!(current[0].layers[0].joint_weights_index, 5);
            assert_eq!(current[0].layers[1].weight, 0.1 * 0.7);
            assert_eq!(current[0].layers[1].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            assert_eq!(current[1].frame, 120);
            assert_eq!(current[1].animation_file, "Girl/Run_Empty.*");
            assert_eq!(current[1].sampling_job.ratio(), 0.2);
            assert_eq!(current[1].additive_blending, true);
            assert_eq!(current[1].layers.len(), 1);
            assert_eq!(current[1].layers[0].weight, 0.3 * 0.7);
            assert_eq!(current[1].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }
    }

    #[test]
    fn test_action_data_update() {
        let (mut asset_loader, skeleton) = prepare_resource();
        let mut arena = SamplingArena::new(&skeleton, 3);
        let mut ad: ActionData = ActionData::default();

        let state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
        let state_ex = StateActionEx::new(&state, state.fade_in_weight, state.fade_in_weight);
        let res = ad.update(&mut arena, 30, state_ex, &skeleton, &mut asset_loader);
        assert!(res.is_err());

        {
            let mut state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state.fade_in_weight = 0.4;

            let mut anim0 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 11, 0.4, 0.7);
            anim0.set_joint_weights_index(3);
            state.animations.push(anim0);

            let mut anim_reuse = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 11, 0.4, 0.5);
            anim_reuse.reuse_layer_id = 1;
            state.animations.push(anim_reuse);

            let mut anim1 = StateActionAnimation::new(sb!("Girl/Run_Empty.*"), 12, 0.6, 0.3);
            anim1.set_additive_blending(true);
            state.animations.push(anim1);

            let state_ex = StateActionEx::new(&state, state.fade_in_weight, state.fade_in_weight);
            ad.update(&mut arena, 31, state_ex, &skeleton, &mut asset_loader)
                .unwrap();

            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);

            assert_eq!(current[0].animation_id, 11);
            assert_eq!(current[0].frame, 31);
            assert_eq!(current[0].layers.len(), 2);
            assert_eq!(current[0].layers[0].weight, 0.7 * 0.4);
            assert_eq!(current[0].layers[0].joint_weights_index, 3);
            assert_eq!(current[0].layers[1].weight, 0.5 * 0.4);
            assert_eq!(current[0].layers[1].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);
            assert_eq!(current[0].animation_file, "Girl/Idle_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.4);
            assert_eq!(current[0].additive_blending, false);

            assert_eq!(current[1].animation_id, 12);
            assert_eq!(current[1].frame, 31);
            assert_eq!(current[1].animation_file, "Girl/Run_Empty.*");
            assert_eq!(current[1].sampling_job.ratio(), 0.6);
            assert_eq!(current[1].additive_blending, true);
            assert_eq!(current[1].layers.len(), 1);
            assert_eq!(current[1].layers[0].weight, 0.3 * 0.4);
            assert_eq!(current[1].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
        }

        {
            let mut state: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state.fade_in_weight = 1.0;

            let mut anim2 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 12, 0.7, 1.0);
            anim2.set_additive_blending(true);
            anim2.set_joint_weights_index(7);
            state.animations.push(anim2);
            state
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Attack_01A.*"), 13, 0.0, 0.0));
            state
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Attack_02A.*"), 14, 0.0, 0.0));

            let state_ex = StateActionEx::new(&state, state.fade_in_weight, state.fade_in_weight);
            ad.update(&mut arena, 32, state_ex, &skeleton, &mut asset_loader)
                .unwrap();

            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 3);

            assert_eq!(current[0].animation_id, 12);
            assert_eq!(current[0].frame, 32);
            assert_eq!(current[0].layers[0].weight, 1.0);
            assert_eq!(current[0].animation_file, "Girl/Run_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.7);
            assert_eq!(current[0].additive_blending, true);
            assert_eq!(current[0].layers[0].joint_weights_index, 7);

            assert_eq!(current[1].animation_id, 13);
            assert_eq!(current[1].animation_file, "Girl/Attack_01A.*");
            assert_eq!(current[1].additive_blending, false);
            assert_eq!(current[1].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            assert_eq!(current[2].animation_id, 14);
            assert_eq!(current[2].animation_file, "Girl/Attack_02A.*");
            assert_eq!(current[2].additive_blending, false);
            assert_eq!(current[2].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 1);
            assert_eq!(past[0].animation_id, 11);
            assert_eq!(past[0].frame, 31);
            assert_eq!(past[0].animation_file, "Girl/Idle_Empty.*");
            assert_eq!(past[0].layers.len(), 2);
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
            let mut arena = SamplingArena::new(&skeleton, 3);
            let mut ad: ActionData = ActionData::default();

            let mut state1: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state1.fade_in_weight = 1.0;

            let mut anim0 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 11, 0.1, 0.4);
            anim0.set_joint_weights_index(2);
            state1.animations.push(anim0);

            let mut anim1 = StateActionAnimation::new(sb!("Girl/Run_Empty.*"), 12, 0.2, 0.6);
            anim1.set_additive_blending(true);
            state1.animations.push(anim1);

            let state1_ex = StateActionEx::new(&state1, state1.fade_in_weight, state1.fade_in_weight);
            ad.update(&mut arena, 50, state1_ex, &skeleton, &mut asset_loader)
                .unwrap();
            let past = list_sampling(&arena, ad.job_past, u32::MAX);
            assert_eq!(past.len(), 2);

            let mut state2: Box<dyn StateActionAny> = Box::new(StateActionEmpty::default());
            state2.fade_in_weight = 0.8;
            state2
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Run_Empty.*"), 12, 0.2, 1.0));

            state2
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Attack_01A.*"), 13, 0.0, 0.0));

            let mut anim_reuse = StateActionAnimation::new(sb!("Girl/Attack_01A.*"), 13, 0.0, 0.321);
            anim_reuse.reuse_layer_id = 1;
            anim_reuse.set_joint_weights_index(3);
            state2.animations.push(anim_reuse);

            state2
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Attack_02A.*"), 14, 0.0, 0.0));

            let state2_ex = StateActionEx::new(&state2, state2.fade_in_weight, state2.fade_in_weight);
            ad.update(&mut arena, 51, state2_ex, &skeleton, &mut asset_loader)
                .unwrap();
            let past = list_sampling(&arena, ad.job_past, u32::MAX);
            assert_eq!(past.len(), 4);

            (asset_loader, skeleton, arena, ad, state1, state2)
        }

        {
            let (mut asset_loader, skeleton, mut arena, mut ad, state1, state2) = prepare();
            let state1_ex = StateActionEx::new(&state1, state1.fade_in_weight, state1.fade_in_weight);
            ad.restore(&mut arena, 50, state1_ex).unwrap();
            let current = list_sampling(&arena, ad.job_current, ad.job_future);
            assert_eq!(current.len(), 2);

            assert_eq!(current[0].animation_id, 11);
            assert_eq!(current[0].frame, 50);
            assert_eq!(current[0].animation_file, "Girl/Idle_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.1);
            assert_eq!(current[0].additive_blending, false);
            assert_eq!(current[0].layers.len(), 1);
            assert_eq!(current[0].layers[0].weight, 0.4);
            assert_eq!(current[0].layers[0].joint_weights_index, 2);

            assert_eq!(current[1].animation_id, 12);
            assert_eq!(current[1].frame, 50);
            assert_eq!(current[1].animation_file, "Girl/Run_Empty.*");
            assert_eq!(current[1].sampling_job.ratio(), 0.2);
            assert_eq!(current[1].additive_blending, true);
            assert_eq!(current[1].layers[0].weight, 0.6);
            assert_eq!(current[1].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 0);
            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 2);
            assert_eq!(list_next(&arena, arena.free), vec![4, 5]);

            let state2_ex = StateActionEx::new(&state2, state2.fade_in_weight, state2.fade_in_weight);
            ad.update(&mut arena, 51, state2_ex, &skeleton, &mut asset_loader)
                .unwrap();
            let current = list_sampling(&arena, ad.job_current, u32::MAX);
            assert_eq!(current.len(), 3);

            assert_eq!(current[0].animation_id, 12);
            assert_eq!(current[0].frame, 51);
            assert_eq!(current[0].animation_file, "Girl/Run_Empty.*");
            assert_eq!(current[0].sampling_job.ratio(), 0.2);
            assert_eq!(current[0].additive_blending, true);
            assert_eq!(current[0].layers.len(), 1);
            assert_eq!(current[0].layers[0].weight, 1.0 * 0.8);
            assert_eq!(current[0].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            assert_eq!(current[1].animation_id, 13);
            assert_eq!(current[1].animation_file, "Girl/Attack_01A.*");
            assert_eq!(current[1].additive_blending, false);
            assert_eq!(current[1].layers.len(), 2);
            assert_eq!(current[1].layers[0].weight, 0.0);
            assert_eq!(current[1].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);
            assert_eq!(current[1].layers[1].weight, 0.321 * 0.8);
            assert_eq!(current[1].layers[1].joint_weights_index, 3);

            assert_eq!(current[2].animation_id, 14);
            assert_eq!(current[2].animation_file, "Girl/Attack_02A.*");
            assert_eq!(current[2].additive_blending, false);
            assert_eq!(current[2].layers.len(), 1);
            assert_eq!(current[2].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

            let past = list_sampling(&arena, ad.job_past, ad.job_current);
            assert_eq!(past.len(), 1);
            assert_eq!(past[0].animation_id, 11);
            assert_eq!(past[0].layers.len(), 1);

            let future = list_sampling(&arena, ad.job_future, u32::MAX);
            assert_eq!(future.len(), 0);
            assert_eq!(list_next(&arena, arena.free), vec![4, 5]);
        }

        {
            let (mut asset_loader, skeleton, mut arena, mut ad, state1, mut state2) = prepare();
            let state1_ex = StateActionEx::new(&state1, state1.fade_in_weight, state1.fade_in_weight);
            ad.restore(&mut arena, 50, state1_ex).unwrap();
            state2.animations[1].animation_id = 14;
            state2.animations[1].files = sb!("Girl/Walk_Empty.*");
            state2.animations[1].ratio = 0.5;
            state2.animations[1].weight = 0.5;
            state2.animations.pop();
            state2.animations.pop();

            let state2_ex = StateActionEx::new(&state2, state2.fade_in_weight, state2.fade_in_weight);
            ad.update(&mut arena, 51, state2_ex, &skeleton, &mut asset_loader)
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
        let mut animator = Animator::new(skeleton, Some(Rc::new(JointWeightsTable::default())), 0, 3).unwrap();

        let mut states: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states[0].id = 21;
        states[0].tmpl_id = id!("Action.Empty^1");
        let mut anim0 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 101, 0.0, 0.0);
        anim0.set_joint_weights_index(1);
        states[0].animations.push(anim0);
        states[1].id = 22;
        states[1].tmpl_id = id!("Action.Empty^2");
        let mut anim1 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 102, 0.0, 0.0);
        anim1.set_additive_blending(true);
        states[1].animations.push(anim1);
        animator.update(105, &states, &mut asset_loader).unwrap();
        assert_eq!(animator.action_queue.len(), 2);
        assert_eq!(animator.action_queue[0].id, 21);
        assert_eq!(animator.action_queue[0].tmpl_id, id!("Action.Empty^1"));
        assert_eq!(animator.action_queue[0].frame, 105);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 101);
        assert_eq!(sampling[0].additive_blending, false);
        assert_eq!(sampling[0].layers[0].joint_weights_index, 1);
        assert_eq!(animator.action_queue[1].id, 22);
        assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^2"));
        assert_eq!(animator.action_queue[1].frame, 105);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 102);
        assert_eq!(sampling[0].additive_blending, true);
        assert_eq!(sampling[0].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);

        let mut states: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states[0].id = 22;
        states[0].tmpl_id = id!("Action.Empty^2");
        // verify path: existing job with same animation_id gets flags/joint_weights updated
        let mut anim2 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 102, 0.0, 0.0);
        anim2.set_additive_blending(true);
        anim2.set_joint_weights_index(2);
        states[0].animations.push(anim2);
        states[1].id = 23;
        states[1].tmpl_id = id!("Action.Empty^3");
        // enqueue path: new job
        let mut anim3 = StateActionAnimation::new(sb!("Girl/Attack_01A.*"), 103, 0.0, 0.0);
        anim3.set_joint_weights_index(3);
        states[1].animations.push(anim3);
        animator.update(106, &states, &mut asset_loader).unwrap();
        assert_eq!(animator.action_queue.len(), 2);
        assert_eq!(animator.action_queue[0].id, 22);
        assert_eq!(animator.action_queue[0].tmpl_id, id!("Action.Empty^2"));
        assert_eq!(animator.action_queue[0].frame, 106);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 102);
        assert_eq!(sampling[0].additive_blending, true);
        assert_eq!(sampling[0].layers[0].joint_weights_index, 2);
        assert_eq!(animator.action_queue[1].id, 23);
        assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^3"));
        assert_eq!(animator.action_queue[1].frame, 106);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
        assert_eq!(sampling.len(), 1);
        assert_eq!(sampling[0].animation_id, 103);
        assert_eq!(sampling[0].additive_blending, false);
        assert_eq!(sampling[0].layers[0].joint_weights_index, 3);
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
            let mut animator = Animator::new(skeleton, Some(Rc::new(JointWeightsTable::default())), 0, 3).unwrap();

            let mut states1: Vec<Box<dyn StateActionAny>> = vec![
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
            ];
            states1[0].id = 41;
            states1[0].tmpl_id = id!("Action.Empty^1");
            let mut anim0 = StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 101, 0.0, 0.0);
            anim0.set_joint_weights_index(4);
            states1[0].animations.push(anim0);
            states1[1].id = 42;
            states1[1].tmpl_id = id!("Action.Empty^2");
            let mut anim1 = StateActionAnimation::new(sb!("Girl/Run_Empty.*"), 102, 0.0, 0.0);
            anim1.set_additive_blending(true);
            states1[1].animations.push(anim1);
            animator.update(205, &states1, &mut asset_loader).unwrap();

            let mut states2: Vec<Box<dyn StateActionAny>> = vec![
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
                Box::new(StateActionEmpty::default()),
            ];
            states2[0].id = 42;
            states2[0].tmpl_id = id!("Action.Empty^2");
            states2[0]
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Run_Empty.*"), 102, 0.0, 0.0));
            states2[1].id = 43;
            states2[1].tmpl_id = id!("Action.Empty^3");
            states2[1]
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Attack_01A.*"), 103, 0.0, 0.0));
            states2[2].id = 44;
            states2[2].tmpl_id = id!("Action.Empty^4");
            states2[2]
                .animations
                .push(StateActionAnimation::new(sb!("Girl/Attack_02A.*"), 104, 0.0, 0.0));
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
            assert_eq!(sampling[0].additive_blending, false);
            assert_eq!(sampling[0].layers[0].joint_weights_index, 4);
            assert_eq!(animator.action_queue[1].id, 42);
            assert_eq!(animator.action_queue[1].tmpl_id, id!("Action.Empty^2"));
            assert_eq!(animator.action_queue[1].frame, 205);
            let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[1].job_current, u32::MAX);
            assert_eq!(sampling.len(), 1);
            assert_eq!(sampling[0].animation_id, 102);
            assert_eq!(sampling[0].additive_blending, true);
            assert_eq!(sampling[0].layers[0].joint_weights_index, EMPTY_JOINT_WEIGHT_INDEX);
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
            states2[1].id = 45;
            states2[1].tmpl_id = id!("Action.Empty^X");
            states2[1].animations[0].animation_id = 105;
            states2[1].animations[0].files = sb!("Girl/Walk_Empty.*");
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
        let mut animator = Animator::new(skeleton, Some(Rc::new(JointWeightsTable::default())), 0, 3).unwrap();

        let mut states1: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states1[0].id = 41;
        states1[0].tmpl_id = id!("Action.Empty^1");
        states1[0]
            .animations
            .push(StateActionAnimation::new(sb!("Girl/Idle_Empty.*"), 101, 0.0, 0.0));
        states1[1].id = 42;
        states1[1].tmpl_id = id!("Action.Empty^2");
        states1[1]
            .animations
            .push(StateActionAnimation::new(sb!("Girl/Run_Empty.*"), 102, 0.0, 0.0));
        animator.update(205, &states1, &mut asset_loader).unwrap();

        let mut states2: Vec<Box<dyn StateActionAny>> = vec![
            Box::new(StateActionEmpty::default()),
            Box::new(StateActionEmpty::default()),
        ];
        states2[0].id = 42;
        states2[0].tmpl_id = id!("Action.Empty^2");
        states2[0]
            .animations
            .push(StateActionAnimation::new(sb!("Girl/Attack_01A.*"), 103, 0.0, 0.0));
        states2[1].id = 43;
        states2[1].tmpl_id = id!("Action.Empty^4");
        states2[1]
            .animations
            .push(StateActionAnimation::new(sb!("Girl/Attack_02A.*"), 104, 0.0, 0.0));
        animator.update(206, &states2, &mut asset_loader).unwrap();

        animator.discard(205);
        assert_eq!(animator.action_queue.len(), 2);
        assert_eq!(animator.action_queue.past_len(), 0);
        assert_eq!(animator.action_queue.future_len(), 0);
        let sampling = list_sampling(&animator.sampling_arena, animator.action_queue[0].job_past, u32::MAX);
        assert_eq!(sampling.len(), 1);
    }
}
