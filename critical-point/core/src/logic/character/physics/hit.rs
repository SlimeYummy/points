use glam_ext::Isometry3A;
use jolt_physics_rs::{BodyCreationSettings, BodyID, BodyInterface, MotionType};

use crate::animation::{HitMotion, HitSampler};
use crate::consts::MAX_HIT_TIMES_PER_FRAME;
use crate::logic::character::control::LogicCharaControl;
use crate::logic::character::physics::physics::{LogicCharaPhysics, StateCharaHitBoxPair, StateCharaHitGroupPair};
use crate::logic::game::{ContextHitGenerate, ContextUpdate, HitCharacterEvent};
use crate::logic::physics::{PhyBodyUserData, PhyHitCharacterEvent, phy_layer};
use crate::utils::{XResult, find_offset_by, ok_or, strict_lt, xfrom};

impl LogicCharaPhysics {
    pub(super) fn handle_action_changed(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_act: &LogicCharaControl,
    ) -> XResult<()> {
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

    pub(super) fn update_boxes_and_groups(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_act: &LogicCharaControl,
    ) -> XResult<()> {
        let sampler = ok_or!(chara_act.hit_motion_sampler(); return Ok(()));
        let body_itf = ctx.physics.body_itf();
        let chara_isometry = Isometry3A::new_3a(self.position, self.rotation * self.inst_chara.skeleton_rotation);

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
            zelf: &mut LogicCharaPhysics,
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
        dst_chara_phy: &mut LogicCharaPhysics,
        ctx: &mut ContextHitGenerate<HitCharacterEvent>,
        chara_act: &LogicCharaControl,
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
            dst_chara_phy.be_hit_events.push(ctx.events.len());

            ctx.events.push(HitCharacterEvent {
                src_chara_id: self.chara_id,
                dst_chara_id: dst_chara_phy.chara_id,
                group: asset_box.group,
                box_index: asset_box.box_index,
                group_index: asset_box.group_index,
                box_hit_times: box_pair.hit_times,
                group_hit_times: group_pair.hit_times,
                collision_normal: phy_event.world_space_normal,
                collision_point_average: phy_event.collision_point_average,
                character_vector: dst_chara_phy.position - self.position,
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
