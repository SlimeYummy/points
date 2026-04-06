use critical_point_csgen::CsOut;
use educe::Educe;
use glam::Vec3;
use glam_ext::Isometry3A;
use jolt_physics_rs::{BodyCreationSettings, BodyID, BodyInterface, MotionType};
use std::rc::Rc;

use crate::animation::{HitMotion, HitSampler};
use crate::consts::MAX_HIT_TIMES_PER_FRAME;
use crate::instance::InstCharacter;
use crate::logic::character::action::LogicCharaAction;
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::game::{ContextHitGenerate, ContextRestore, ContextUpdate, HitCharacterEvent};
use crate::logic::physics::{phy_layer, PhyBodyUserData, PhyHitCharacterEvent};
use crate::utils::{find_offset_by, ok_or, strict_lt, xfrom, NumID, SmallVec, Symbol, XResult};

#[repr(C)]
#[derive(
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Value)]
pub struct StateCharaHit {
    #[cs_hide(32, 8)]
    pub body_ids: SmallVec<[BodyID; 4]>,
    #[cs_hide(64, 8)]
    pub box_pairs: SmallVec<[StateCharaHitBoxPair; 4]>,
    #[cs_hide(64, 8)]
    pub group_pairs: SmallVec<[StateCharaHitGroupPair; 3]>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct StateCharaHitBoxPair {
    pub box_index: u16,
    pub dst_chara_id: NumID,
    pub last_hit_time: f32,
    pub hit_times: u16,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct StateCharaHitGroupPair {
    pub group: Symbol,
    pub dst_chara_id: NumID,
    pub hit_times: u16,
}

#[derive(Educe)]
#[educe(Debug)]
pub(crate) struct LogicCharaHit {
    chara_id: NumID,
    inst_chara: Rc<InstCharacter>,
    body_ids: Vec<BodyID>,
    box_pairs: Vec<StateCharaHitBoxPair>,
    group_pairs: Vec<StateCharaHitGroupPair>,

    hit_events: Vec<usize>,
    be_hit_events: Vec<usize>,
}

impl LogicCharaHit {
    pub(crate) fn new(
        _ctx: &mut ContextUpdate,
        chara_id: NumID,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicCharaHit> {
        Ok(LogicCharaHit {
            chara_id,
            inst_chara,
            body_ids: Vec::with_capacity(32),
            box_pairs: Vec::with_capacity(32),
            group_pairs: Vec::with_capacity(16),

            hit_events: Vec::with_capacity(32),
            be_hit_events: Vec::with_capacity(32),
        })
    }

    pub(crate) fn state(&self) -> StateCharaHit {
        StateCharaHit {
            body_ids: SmallVec::from_slice(&self.body_ids),
            box_pairs: SmallVec::from_slice(&self.box_pairs),
            group_pairs: SmallVec::from_slice(&self.group_pairs),
        }
    }

    pub(crate) fn restore(&mut self, _ctx: &ContextRestore, state: &StateCharaHit) -> XResult<()> {
        self.body_ids.clear();
        self.body_ids.extend_from_slice(&state.body_ids);
        self.box_pairs.clear();
        self.box_pairs.extend_from_slice(&state.box_pairs);
        self.group_pairs.clear();
        self.group_pairs.extend_from_slice(&state.group_pairs);
        Ok(())
    }

    pub(crate) fn init(&mut self, _ctx: &mut ContextUpdate) -> XResult<()> {
        Ok(())
    }

    pub(crate) fn clear_events(&mut self) {
        self.hit_events.clear();
        self.be_hit_events.clear();
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_act: &LogicCharaAction,
        chara_phy: &LogicCharaPhysics,
    ) -> XResult<()> {
        if chara_act.animation_changed() {
            self.update_action_changed(ctx, chara_act)?
        }
        self.update_boxes_and_groups(ctx, chara_act, chara_phy)?;
        Ok(())
    }

    fn update_action_changed(&mut self, ctx: &mut ContextUpdate, chara_act: &LogicCharaAction) -> XResult<()> {
        // clear previous box_pairs

        let body_itf = ctx.physics.body_itf();
        for body_id in self.body_ids.drain(..) {
            if body_id.is_valid() {
                body_itf.remove_body(body_id);
                body_itf.destroy_body(body_id);
            }
        }

        self.box_pairs.clear();
        self.group_pairs.clear();

        if let Some(sampler) = chara_act.hit_motion_sampler() {
            self.body_ids.resize(sampler.hit_motion.count_boxes(), BodyID::INVALID);
        }

        Ok(())
    }

    fn update_boxes_and_groups(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_act: &LogicCharaAction,
        chara_phy: &LogicCharaPhysics,
    ) -> XResult<()> {
        let sampler = ok_or!(chara_act.hit_motion_sampler(); return Ok(()));
        let body_itf = ctx.physics.body_itf();
        let chara_isometry = Isometry3A::new_3a(
            chara_phy.position(),
            chara_phy.rotation() * self.inst_chara.skeleton_rotation,
        );

        for joint in sampler.joints() {
            update(
                self,
                body_itf,
                &sampler.hit_motion,
                joint,
                sampler.time(),
                chara_isometry,
            )?;
        }

        for weapon in sampler.weapons() {
            update(
                self,
                body_itf,
                &sampler.hit_motion,
                weapon,
                sampler.time(),
                chara_isometry,
            )?;
        }

        fn update<S>(
            zelf: &mut LogicCharaHit,
            body_itf: &mut BodyInterface,
            hit_motion: &HitMotion,
            sampler: &HitSampler<S>,
            sampler_time: f32,
            chara_isometry: Isometry3A,
        ) -> XResult<()> {
            let box_idx = sampler.box_index as usize;
            let Some(asset_box) = hit_motion.find_box(sampler.box_index)
            else {
                debug_assert!(
                    false,
                    "character={}, hit_motion={}, box_index={} Invalid hit box",
                    zelf.chara_id,
                    hit_motion.name(),
                    box_idx
                );
                return Ok(());
            };

            // If hit box is active, the sampler will return its position information.
            if let Some(isometry) = sampler.isometry() {
                let hit_isometry = chara_isometry * *isometry;

                if zelf.body_ids[box_idx] == BodyID::INVALID {
                    let mut settings = BodyCreationSettings::new_sensor(
                        asset_box.shape.clone(),
                        phy_layer!(Hit, zelf.inst_chara.is_player => Enemy | Player),
                        MotionType::Static,
                        hit_isometry.translation,
                        hit_isometry.rotation,
                    );
                    settings.user_data = PhyBodyUserData::new_hit(zelf.chara_id, asset_box.box_index).into();
                    zelf.body_ids[box_idx] = body_itf.create_add_body(&settings, true).map_err(xfrom!())?;
                }
                else {
                    body_itf.set_position_rotation(
                        zelf.body_ids[box_idx],
                        hit_isometry.translation,
                        hit_isometry.rotation,
                        true,
                    );
                }
            }
            // No position information, hit box inactive, clean up if needed.
            else {
                if zelf.body_ids[box_idx] != BodyID::INVALID {
                    body_itf.remove_body(zelf.body_ids[box_idx]);
                    body_itf.destroy_body(zelf.body_ids[box_idx]);
                    zelf.body_ids[box_idx] = BodyID::INVALID;

                    zelf.box_pairs.retain(|p| p.box_index != asset_box.box_index);

                    let asset_group = &hit_motion.groups()[asset_box.group_index as usize];
                    if !asset_group.in_time_loose(sampler_time) {
                        zelf.group_pairs.retain(|p| p.group != asset_group.name);
                    }
                }
            }
            Ok(())
        }

        Ok(())
    }

    pub(crate) fn detect_hits(
        &mut self,
        dst_chara_hit: &mut LogicCharaHit,
        ctx: &mut ContextHitGenerate<HitCharacterEvent>,
        chara_act: &LogicCharaAction,
        chara_phy: &LogicCharaPhysics,
        dst_chara_phy: &LogicCharaPhysics,
        phy_event: &PhyHitCharacterEvent,
    ) -> XResult<usize> {
        let curr_act = ok_or!(chara_act.current_action_with_log(); return Ok(0));
        let sampler = ok_or!(chara_act.hit_motion_sampler_with_log(); return Ok(0));

        let Some(asset_box) = sampler.hit_motion.find_box(phy_event.src_box_index)
        else {
            debug_assert!(
                false,
                "character={}, action={}, box_index={}, invalid hit box index",
                self.chara_id, curr_act.inst.tmpl_id, phy_event.src_box_index
            );
            return Ok(0);
        };

        let Some(inst_hit) = find_offset_by(&curr_act.inst.hits, asset_box.group_index as usize, |hit| {
            hit.group == asset_box.group
        })
        else {
            debug_assert!(
                false,
                "character={}, action={}, group_index={}, instHit not found",
                self.chara_id, curr_act.inst.tmpl_id, asset_box.group_index
            );
            return Ok(0);
        };

        let group_pair = match self
            .group_pairs
            .iter_mut()
            .find(|p| p.group == asset_box.group && p.dst_chara_id == phy_event.dst_chara_id)
        {
            Some(p) => p,
            None => {
                self.group_pairs.push(StateCharaHitGroupPair {
                    group: asset_box.group.clone(),
                    dst_chara_id: phy_event.dst_chara_id,
                    hit_times: 0,
                });
                self.group_pairs.last_mut().unwrap()
            }
        };
        if group_pair.hit_times >= inst_hit.group_max_times {
            return Ok(0);
        }

        let box_pair = match self
            .box_pairs
            .iter_mut()
            .find(|p| p.box_index == asset_box.box_index && p.dst_chara_id == phy_event.dst_chara_id)
        {
            Some(p) => p,
            None => {
                self.box_pairs.push(StateCharaHitBoxPair {
                    box_index: phy_event.src_box_index,
                    dst_chara_id: phy_event.dst_chara_id,
                    last_hit_time: ctx.time - inst_hit.box_min_interval,
                    hit_times: 0,
                });
                self.box_pairs.last_mut().unwrap()
            }
        };

        let mut event_count = 0;
        for _ in 0..MAX_HIT_TIMES_PER_FRAME {
            if strict_lt!(ctx.time - box_pair.last_hit_time, inst_hit.box_min_interval) {
                break;
            }
            if box_pair.hit_times >= inst_hit.box_max_times {
                break;
            }
            if group_pair.hit_times >= inst_hit.group_max_times {
                break;
            }

            box_pair.last_hit_time += inst_hit.box_min_interval;
            box_pair.hit_times += 1;
            group_pair.hit_times += 1;

            self.hit_events.push(ctx.events.len());
            dst_chara_hit.be_hit_events.push(ctx.events.len());

            ctx.events.push(HitCharacterEvent {
                src_chara_id: self.chara_id,
                dst_chara_id: dst_chara_hit.chara_id,
                group: asset_box.group,
                box_index: asset_box.box_index,
                group_index: asset_box.group_index,
                box_hit_times: box_pair.hit_times,
                group_hit_times: group_pair.hit_times,
                collision_normal: phy_event.world_space_normal,
                collision_point_average: phy_event.collision_point_average,
                character_vector: dst_chara_phy.position() - chara_phy.position(),
                ..Default::default()
            });

            event_count += 1;
            log::debug!("detect_hits() event:{:?}", ctx.events.last().unwrap());
        }

        Ok(event_count)
    }

    #[inline]
    pub(crate) fn hit_events(&self) -> &Vec<usize> {
        &self.hit_events
    }

    #[inline]
    pub(crate) fn be_hit_events(&self) -> &Vec<usize> {
        &self.be_hit_events
    }
}
