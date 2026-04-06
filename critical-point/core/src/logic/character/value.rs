use critical_point_csgen::CsOut;
use std::rc::Rc;

use crate::instance::InstCharacter;
use crate::logic::character::LogicCharaAction;
use crate::logic::game::{ContextHitUpdate, ContextRestore, ContextUpdate, HitCharacterEvent};
use crate::logic::physics::PhyHitCharacterEvent;
use crate::utils::{cf2s, NumID, TimeRange, XResult};

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
pub struct StateCharaValue {
    pub hit_lag_time: TimeRange,
}

#[derive(Debug)]
pub(crate) struct LogicCharaValue {
    chara_id: NumID,
    inst_chara: Rc<InstCharacter>,

    hit_lag_time: TimeRange,
}

impl LogicCharaValue {
    pub(crate) fn new(ctx: &mut ContextUpdate, chara_id: NumID, inst_chara: Rc<InstCharacter>) -> LogicCharaValue {
        LogicCharaValue {
            chara_id,
            inst_chara,
            hit_lag_time: TimeRange::EMPTY,
        }
    }

    pub(crate) fn state(&self) -> StateCharaValue {
        StateCharaValue {
            hit_lag_time: self.hit_lag_time,
        }
    }

    pub(crate) fn restore(&mut self, _ctx: &ContextRestore, state: &StateCharaValue) -> XResult<()> {
        self.hit_lag_time = state.hit_lag_time;
        Ok(())
    }

    pub(crate) fn init(&mut self, _ctx: &mut ContextUpdate) -> XResult<()> {
        Ok(())
    }

    pub(crate) fn update(&mut self, ctx: &mut ContextUpdate) -> XResult<()> {
        if !self.hit_lag_time.contains(ctx.time) {
            self.hit_lag_time = TimeRange::EMPTY;
        }
        Ok(())
    }

    pub(crate) fn before_hit(
        &mut self,
        dst_chara_val: &mut LogicCharaValue,
        ctx: &mut ContextHitUpdate<HitCharacterEvent>,
        phy_event: &PhyHitCharacterEvent,
    ) -> XResult<()> {
        self.hit_lag_time = TimeRange::new(ctx.time, ctx.time + cf2s(10));
        Ok(())
    }

    pub(crate) fn on_hit(
        &self,
        dst_val: &mut LogicCharaValue,
        ctx: &mut ContextHitUpdate<HitCharacterEvent>,
    ) -> XResult<()> {
        Ok(())
    }

    pub(crate) fn after_hit(
        &self,
        dst_val: &mut LogicCharaValue,
        ctx: &mut ContextHitUpdate<HitCharacterEvent>,
    ) -> XResult<()> {
        Ok(())
    }
}

impl LogicCharaValue {
    #[inline]
    pub(crate) fn hit_lag_time(&self) -> TimeRange {
        self.hit_lag_time
    }
}
