use critical_point_macros::{csharp_out, wasm_struct};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::instance::InstCharacter;
use crate::logic::game::{ContextHitUpdate, ContextRestore, ContextUpdate, HitCharacterEvent};
use crate::logic::physics::PhyHitCharacterEvent;
use crate::script::WsBox;
use crate::utils::{NumID, TimeRange, XResult, cf2s, ifelse};

#[repr(C)]
#[csharp_out(Value)]
#[derive(
    Debug, Default, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct StateCharaValue {
    pub time_speed: f32,
    pub hit_lag_time: TimeRange,
}

#[derive(Debug)]
pub(crate) struct LogicCharaValue {
    chara_id: NumID,
    inst_chara: Rc<InstCharacter>,

    pub(crate) ws: WsBox<WsCharaValue>,
}

#[repr(C)]
#[wasm_struct(20, 4)]
#[derive(Debug)]
pub(crate) struct WsCharaValue {
    pub chara_id: NumID,
    pub time_speed: f32,
    pub hit_lag_time: TimeRange,
    pub is_player: bool,
    pub is_ai_idle: bool,
}

impl Deref for LogicCharaValue {
    type Target = WsCharaValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.ws.as_ref()
    }
}

impl DerefMut for LogicCharaValue {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ws.as_mut()
    }
}

impl LogicCharaValue {
    pub(crate) fn new(ctx: &mut ContextUpdate, chara_id: NumID, inst_chara: Rc<InstCharacter>) -> LogicCharaValue {
        LogicCharaValue {
            chara_id,
            inst_chara,
            ws: WsBox::new_in(
                WsCharaValue {
                    chara_id,
                    time_speed: 1.0,
                    hit_lag_time: TimeRange::EMPTY,
                    is_player: false,
                    is_ai_idle: false,
                },
                ctx.script.alloc(),
            ),
        }
    }

    pub(crate) fn state(&self) -> StateCharaValue {
        StateCharaValue {
            time_speed: self.time_speed,
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

        self.time_speed = ifelse!(self.hit_lag_time().contains(ctx.time), 0.0, 1.0);
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
    pub(crate) fn time_speed(&self) -> f32 {
        self.time_speed
    }

    #[inline]
    pub(crate) fn hit_lag_time(&self) -> TimeRange {
        self.hit_lag_time
    }
}
