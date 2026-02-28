use critical_point_csgen::{CsEnum, CsOut};
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::alloc::Layout;
use std::any::Any;
use std::fmt::Debug;
use std::hint::unlikely;
use std::rc::Rc;

use crate::consts::{MAX_ACTION_ANIMATION, SPF};
use crate::instance::{InstActionAny, InstAnimation};
use crate::logic::character::LogicCharaPhysics;
use crate::logic::game::ContextUpdate;
use crate::logic::system::input::InputVariables;
use crate::utils::{interface, rkyv_self, xres, ActionType, ArrayVec, CustomEvent, NumID, Symbol, TmplID, XResult};

//
// StateActionAny & StateActionBase
//

#[typetag::serde(tag = "T")]
pub unsafe trait StateActionAny
where
    Self: Debug + Any + Send + Sync,
{
    fn id(&self) -> NumID;
    fn typ(&self) -> ActionType;
    fn layout(&self) -> Layout;
}

#[repr(C)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionBase {
    pub id: NumID,
    pub tmpl_id: TmplID,
    pub typ: ActionType,
    pub status: LogicActionStatus,
    pub first_frame: u32,
    pub last_frame: u32,
    pub fade_in_weight: f32,
    pub derive_level: u16,
    pub poise_level: u16,
    pub animations: ArrayVec<StateActionAnimation, MAX_ACTION_ANIMATION>,
}

interface!(StateActionAny, StateActionBase);

#[cfg(feature = "debug-print")]
impl Drop for StateActionBase {
    fn drop(&mut self) {
        log::debug!("StateActionBase::drop() id={} tmpl_id={}", self.id, self.tmpl_id);
    }
}

impl StateActionBase {
    pub fn new(typ: ActionType) -> StateActionBase {
        StateActionBase {
            id: NumID::INVALID,
            tmpl_id: TmplID::default(),
            typ,
            status: LogicActionStatus::Starting,
            first_frame: 0,
            last_frame: 0,
            fade_in_weight: 1.0,
            derive_level: 0,
            poise_level: 0,
            animations: Default::default(),
        }
    }
}

pub trait ArchivedStateActionAny: Debug + Any {
    fn id(&self) -> NumID;
    fn typ(&self) -> ActionType;
}

#[repr(C)]
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
    CsOut,
)]
#[rkyv(derive(Debug))]
pub struct StateActionAnimation {
    pub files: Symbol,
    pub animation_id: u16,
    pub weapon_motion: bool,
    pub hit_motion: bool,
    pub ratio: f32,
    pub weight: f32,
}

impl Default for StateActionAnimation {
    #[inline]
    fn default() -> Self {
        Self {
            files: Symbol::default(),
            animation_id: u16::MAX,
            weapon_motion: false,
            hit_motion: false,
            ratio: 0.0,
            weight: 1.0,
        }
    }
}

impl StateActionAnimation {
    #[inline]
    pub fn new(
        files: Symbol,
        animation_id: u16,
        weapon_motion: bool,
        hit_motion: bool,
        ratio: f32,
        weight: f32,
    ) -> Self {
        StateActionAnimation {
            files,
            animation_id,
            weapon_motion,
            hit_motion,
            ratio,
            weight,
        }
    }

    #[inline]
    pub fn new_no_motion(files: Symbol, animation_id: u16, ratio: f32, weight: f32) -> Self {
        StateActionAnimation {
            files,
            animation_id,
            weapon_motion: false,
            hit_motion: false,
            ratio,
            weight,
        }
    }

    #[inline]
    pub fn new_with_anim(inst: &InstAnimation, ratio: f32, weight: f32) -> Self {
        StateActionAnimation {
            files: inst.files,
            animation_id: inst.local_id,
            weapon_motion: inst.weapon_motion,
            hit_motion: inst.hit_motion,
            ratio,
            weight,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, rkyv::Portable)]
pub struct StateActionAnyMetadata(rkyv::primitive::ArchivedU16);

impl Default for StateActionAnyMetadata {
    #[inline]
    fn default() -> Self {
        Self(u16::MAX.into())
    }
}

#[allow(unreachable_patterns)]
const _: () = {
    use ptr_meta::Pointee;
    use rkyv::rancor::{Fallible, Source};
    use rkyv::ser::{Allocator, Writer, WriterExt};
    use rkyv::traits::{ArchivePointee, LayoutRaw, NoUndef, Portable};
    use rkyv::{
        Archive, ArchiveUnsized, Archived, ArchivedMetadata, Deserialize, DeserializeUnsized, Serialize,
        SerializeUnsized,
    };
    use std::alloc::LayoutError;
    use std::ptr::DynMetadata;
    use std::{mem, ptr};

    use crate::logic::action::empty::{ArchivedStateActionEmpty, StateActionEmpty};
    use crate::logic::action::general::{ArchivedStateActionGeneral, StateActionGeneral};
    use crate::logic::action::idle::{ArchivedStateActionIdle, StateActionIdle};
    use crate::logic::action::r#move::{ArchivedStateActionMove, StateActionMove};
    use crate::utils::Castable;
    use ActionType::*;

    impl PartialEq for dyn StateActionAny {
        fn eq(&self, other: &Self) -> bool {
            match (self.typ(), other.typ()) {
                (Empty, Empty) => unsafe {
                    self.cast_unchecked::<StateActionEmpty>() == other.cast_unchecked::<StateActionEmpty>()
                },
                (Idle, Idle) => unsafe {
                    self.cast_unchecked::<StateActionIdle>() == other.cast_unchecked::<StateActionIdle>()
                },
                (Move, Move) => unsafe {
                    self.cast_unchecked::<StateActionMove>() == other.cast_unchecked::<StateActionMove>()
                },
                (General, General) => unsafe {
                    self.cast_unchecked::<StateActionGeneral>() == other.cast_unchecked::<StateActionGeneral>()
                },
                _ => false,
            }
        }
    }

    impl LayoutRaw for dyn StateActionAny {
        fn layout_raw(metadata: DynMetadata<dyn StateActionAny>) -> Result<Layout, LayoutError> {
            unsafe {
                let null = ptr::from_raw_parts::<dyn StateActionAny>(ptr::null() as *const u8, metadata);
                Ok((*null).layout())
            }
        }
    }

    unsafe impl Pointee for dyn StateActionAny {
        type Metadata = DynMetadata<dyn StateActionAny>;
    }

    unsafe impl Pointee for dyn ArchivedStateActionAny {
        type Metadata = DynMetadata<dyn ArchivedStateActionAny>;
    }

    unsafe impl Portable for dyn ArchivedStateActionAny {}

    unsafe impl NoUndef for StateActionAnyMetadata {}

    impl ArchivePointee for dyn ArchivedStateActionAny {
        type ArchivedMetadata = StateActionAnyMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let typ = ActionType::try_from(archived.0).expect("Invalid ActionType");
            let archived_ref: &Self = unsafe {
                match typ {
                    Empty => mem::transmute_copy::<usize, &ArchivedStateActionEmpty>(&0),
                    Idle => mem::transmute_copy::<usize, &ArchivedStateActionIdle>(&0),
                    Move => mem::transmute_copy::<usize, &ArchivedStateActionMove>(&0),
                    General => mem::transmute_copy::<usize, &ArchivedStateActionGeneral>(&0),
                    _ => unreachable!("pointer_metadata() Invalid ActionType"),
                }
            };
            ptr::metadata(archived_ref)
        }
    }

    impl ArchiveUnsized for dyn StateActionAny {
        type Archived = dyn ArchivedStateActionAny;

        fn archived_metadata(&self) -> ArchivedMetadata<Self> {
            StateActionAnyMetadata(self.typ().into())
        }
    }

    impl<S> SerializeUnsized<S> for dyn StateActionAny
    where
        S: Fallible + Allocator + Writer + ?Sized,
        S::Error: Source,
    {
        fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
            #[inline(always)]
            fn serialize<T, S>(
                state_any: &(dyn StateActionAny + 'static),
                serializer: &mut S,
            ) -> Result<usize, S::Error>
            where
                T: StateActionAny + Serialize<S> + 'static,
                S: Fallible + Allocator + Writer + ?Sized,
                S::Error: Source,
            {
                let state_ref = unsafe { state_any.cast_unchecked::<T>() };
                let resolver = state_ref.serialize(serializer)?;
                let res = serializer.align_for::<T>()?;
                unsafe { serializer.resolve_aligned(state_ref, resolver)? };
                Ok(res)
            }

            match self.typ() {
                Empty => serialize::<StateActionEmpty, _>(self, serializer),
                Idle => serialize::<StateActionIdle, _>(self, serializer),
                Move => serialize::<StateActionMove, _>(self, serializer),
                General => serialize::<StateActionGeneral, _>(self, serializer),
                _ => unreachable!("serialize_unsized() Invalid ActionType"),
            }
        }
    }

    impl<D> DeserializeUnsized<dyn StateActionAny, D> for dyn ArchivedStateActionAny
    where
        D: Fallible + ?Sized,
        D::Error: Source,
    {
        unsafe fn deserialize_unsized(
            &self,
            deserializer: &mut D,
            out: *mut dyn StateActionAny,
        ) -> Result<(), D::Error> {
            #[inline(always)]
            fn deserialize<T, D>(
                archived_any: &(dyn ArchivedStateActionAny + 'static),
                deserializer: &mut D,
                out: *mut dyn StateActionAny,
            ) -> Result<(), D::Error>
            where
                T: StateActionAny + Archive + 'static,
                D: Fallible + ?Sized,
                Archived<T>: Deserialize<T, D>,
            {
                let archived_ref: &Archived<T> = unsafe { archived_any.cast_unchecked() };
                let value: T = archived_ref.deserialize(deserializer)?;
                let ptr = out as *mut T;
                unsafe { ptr.write(value) };
                Ok(())
            }

            match self.typ() {
                Empty => deserialize::<StateActionEmpty, _>(self, deserializer, out),
                Idle => deserialize::<StateActionIdle, _>(self, deserializer, out),
                Move => deserialize::<StateActionMove, _>(self, deserializer, out),
                General => deserialize::<StateActionGeneral, _>(self, deserializer, out),
                _ => unreachable!("deserialize_unsized() Invalid ActionType"),
            }
        }

        fn deserialize_metadata(&self) -> DynMetadata<dyn StateActionAny> {
            let value_ref: &dyn StateActionAny = unsafe {
                match self.typ() {
                    Empty => mem::transmute_copy::<usize, &StateActionEmpty>(&0),
                    Idle => mem::transmute_copy::<usize, &StateActionIdle>(&0),
                    Move => mem::transmute_copy::<usize, &StateActionMove>(&0),
                    General => mem::transmute_copy::<usize, &StateActionGeneral>(&0),
                    _ => unreachable!("deserialize_metadata() Invalid ActionType"),
                }
            };
            ptr::metadata(value_ref)
        }
    }
};

macro_rules! impl_state_action {
    ($typ:ty, $state_enum:ident, $serde_tag:expr) => {
        paste::paste! {
            #[typetag::serde(name = $serde_tag)]
            unsafe impl $crate::logic::action::StateActionAny for $typ {
                #[inline]
                fn id(&self) -> $crate::utils::NumID {
                    self._base.id
                }

                #[inline]
                fn typ(&self) -> $crate::utils::ActionType {
                    debug_assert_eq!(
                        self._base.typ,
                        $crate::utils::ActionType::$state_enum
                    );
                    $crate::utils::ActionType::$state_enum
                }

                #[inline]
                fn layout(&self) -> std::alloc::Layout {
                    std::alloc::Layout::new::<Self>()
                }
            }

            impl $crate::logic::action::ArchivedStateActionAny for [<Archived $typ>] {
                #[inline]
                fn id(&self) -> crate::utils::NumID {
                    crate::utils::NumID::from_rkyv(self._base.id)
                }

                #[inline]
                fn typ(&self) -> $crate::utils::ActionType {
                    debug_assert_eq!(
                        self._base.typ,
                        $crate::utils::ActionType::$state_enum
                    );
                    $crate::utils::ActionType::$state_enum
                }
            }
        }
    };
}
pub(crate) use impl_state_action;

#[repr(C)]
#[derive(
    Debug,
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
    CsOut,
)]
#[rkyv(derive(Debug))]
pub struct LogicActionAnimationID {
    pub action_id: NumID,
    pub animation_id: u16,
}

impl Default for LogicActionAnimationID {
    #[inline]
    fn default() -> Self {
        Self {
            action_id: NumID::INVALID,
            animation_id: u16::MAX,
        }
    }
}

impl LogicActionAnimationID {
    pub const INVALID: LogicActionAnimationID = LogicActionAnimationID {
        action_id: NumID::INVALID,
        animation_id: u16::MAX,
    };

    #[inline]
    pub fn new(action_id: NumID, animation_id: u16) -> LogicActionAnimationID {
        LogicActionAnimationID {
            action_id,
            animation_id,
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.action_id != u32::MAX && self.animation_id != u16::MAX
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.action_id == u32::MAX || self.animation_id == u16::MAX
    }
}

//
// LogicActionAny & LogicActionBase
//

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize, CsEnum)]
pub enum LogicActionStatus {
    Starting,
    Activing,
    Fading,
    Stopping,
    Finalized,
}

rkyv_self!(LogicActionStatus);

pub unsafe trait LogicActionAny: Debug + Any {
    fn typ(&self) -> ActionType;
    fn save(&self) -> Box<dyn StateActionAny>;
    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()>;

    fn start(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionStartReturn> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.start(ctx, ctxa)?;
        Ok(ActionStartReturn::new())
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn>;

    fn fade_start(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<bool> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.fade_start(ctx, ctxa)?;
        Ok(false)
    }

    fn fade_update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.fade_update(ctx, ctxa)
    }

    fn stop(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.stop(ctx, ctxa)
    }

    fn finalize(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.finalize(ctx, ctxa)
    }
}

#[derive(Debug, Default)]
pub struct ActionStartReturn {
    pub prev_fade_update: bool,
    pub clear_preinput: bool,
    pub custom_events: Vec<CustomEvent>,
}

impl ActionStartReturn {
    #[inline]
    pub fn new() -> ActionStartReturn {
        ActionStartReturn::default()
    }
}

#[derive(Debug, Default)]
pub struct ActionUpdateReturn {
    pub new_velocity: Option<Vec3A>,
    pub new_direction: Option<Vec2xz>,
    pub clear_preinput: bool,
    pub derive_keeping: DeriveKeeping,
    pub custom_events: Vec<CustomEvent>,
}

impl ActionUpdateReturn {
    #[inline]
    pub fn new() -> ActionUpdateReturn {
        ActionUpdateReturn::default()
    }

    #[inline]
    pub fn set_velocity_2d(&mut self, velocity: Vec2xz) {
        self.new_velocity = Some(velocity.as_vec3a());
    }

    #[inline]
    pub fn set_velocity(&mut self, velocity: Vec3A) {
        self.new_velocity = Some(velocity);
    }

    #[inline]
    pub fn set_direction(&mut self, direction: Vec2xz) {
        self.new_direction = Some(direction);
    }
}

pub const LA_FLAG_DERIVE_SELF: u8 = 0x1;

#[derive(Debug)]
pub struct LogicActionBase {
    pub id: NumID,
    pub inst: Rc<dyn InstActionAny>,
    pub status: LogicActionStatus,
    pub flags: u8,
    pub first_frame: u32,
    pub last_frame: u32,
    pub fade_in_weight: f32,
    pub derive_level: u16,
    pub poise_level: u16,
}

interface!(LogicActionAny, LogicActionBase);

impl LogicActionBase {
    pub fn new(id: NumID, inst: Rc<dyn InstActionAny>) -> LogicActionBase {
        LogicActionBase {
            id,
            inst,
            status: LogicActionStatus::Starting,
            flags: 0,
            first_frame: 0,
            last_frame: u32::MAX,
            fade_in_weight: 0.0,
            derive_level: 0,
            poise_level: 0,
        }
    }

    #[inline(always)]
    pub fn set_derive_self(&mut self, enabled: bool) {
        if enabled {
            self.flags |= LA_FLAG_DERIVE_SELF;
        }
        else {
            self.flags &= !LA_FLAG_DERIVE_SELF;
        }
    }

    #[inline(always)]
    pub fn derive_self(&self) -> bool {
        self.flags & LA_FLAG_DERIVE_SELF != 0
    }

    pub fn reuse(&mut self, id: NumID) -> XResult<()> {
        *self = LogicActionBase::new(id, self.inst.clone());
        Ok(())
    }

    pub fn save(&self, typ: ActionType) -> StateActionBase {
        StateActionBase {
            id: self.id,
            tmpl_id: self.inst.tmpl_id,
            typ,
            status: self.status,
            first_frame: self.first_frame,
            last_frame: self.last_frame,
            fade_in_weight: self.fade_in_weight,
            derive_level: self.derive_level,
            poise_level: self.poise_level,
            animations: Default::default(),
        }
    }

    pub fn restore(&mut self, state: &StateActionBase) {
        self.status = state.status;
        self.first_frame = state.first_frame;
        self.last_frame = state.last_frame;
        self.fade_in_weight = state.fade_in_weight;
        self.derive_level = state.derive_level;
        self.poise_level = state.poise_level;
    }

    pub fn start(&mut self, ctx: &ContextUpdate, ctxa: &mut ContextAction) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Starting) {
            return xres!(Unexpected; "status != Starting");
        }
        log::info!("LogicActionAny::start() id={} tmpl_id={}", self.id, self.inst.tmpl_id);
        self.status = LogicActionStatus::Activing;
        self.first_frame = ctx.frame;
        if ctxa.prev_action.is_none() {
            self.fade_in_weight = 1.0;
        }
        Ok(())
    }

    pub fn update(&mut self, _ctx: &ContextUpdate, _ctxa: &mut ContextAction) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Activing) {
            return xres!(Unexpected; "status != Activing");
        }
        Ok(())
    }

    pub fn fade_start(&mut self, _ctx: &ContextUpdate, _ctxa: &mut ContextAction) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Activing) {
            return xres!(Unexpected; "status != Activing");
        }
        log::info!(
            "LogicActionAny::fade_start() id={} tmpl_id={}",
            self.id,
            self.inst.tmpl_id
        );
        self.status = LogicActionStatus::Fading;
        Ok(())
    }

    pub fn fade_update(&mut self, _ctx: &ContextUpdate, _ctxa: &mut ContextAction) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Fading) {
            return xres!(Unexpected; "status != Fading");
        }
        Ok(())
    }

    pub fn stop(&mut self, _ctx: &ContextUpdate, _ctxa: &mut ContextAction) -> XResult<()> {
        if unlikely(matches!(
            self.status,
            LogicActionStatus::Stopping | LogicActionStatus::Finalized
        )) {
            return xres!(Unexpected; "status != Starting/Activing/Fading");
        }
        log::info!("LogicActionAny::stop() id={} tmpl_id={}", self.id, self.inst.tmpl_id);
        self.status = LogicActionStatus::Stopping;
        Ok(())
    }

    pub fn finalize(&mut self, ctx: &ContextUpdate, _ctxa: &mut ContextAction) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Stopping) {
            return xres!(Unexpected; "status != Stopping");
        }
        log::info!(
            "LogicActionAny::finalize() id={} tmpl_id={}",
            self.id,
            self.inst.tmpl_id
        );
        self.status = LogicActionStatus::Finalized;
        self.last_frame = ctx.frame;
        Ok(())
    }

    #[inline]
    pub fn tmpl_id(&self) -> TmplID {
        self.inst.tmpl_id
    }

    #[inline]
    pub fn is_starting(&self) -> bool {
        self.status == LogicActionStatus::Starting
    }

    #[inline]
    pub fn is_activing(&self) -> bool {
        self.status == LogicActionStatus::Activing
    }

    #[inline]
    pub fn is_stopping(&self) -> bool {
        self.status == LogicActionStatus::Stopping
    }

    #[inline]
    pub fn is_fading(&self) -> bool {
        self.status == LogicActionStatus::Fading
    }

    #[inline]
    pub fn is_finalized(&self) -> bool {
        self.status == LogicActionStatus::Finalized
    }
}

//
// ContextAction
//

pub struct ContextAction<'a, 'b> {
    pub player_id: NumID,
    pub chara_physics: &'a LogicCharaPhysics,
    pub prev_action: Option<&'b dyn LogicActionAny>,
    pub input_vars: InputVariables,
    pub time_speed: f32,
    pub time_step: f32,
}

impl<'a, 'b> ContextAction<'a, 'b> {
    pub(crate) fn new(
        player_id: NumID,
        chara_physics: &'a LogicCharaPhysics,
        input_vars: InputVariables,
    ) -> ContextAction<'a, 'b> {
        ContextAction {
            player_id,
            chara_physics,
            prev_action: None,
            time_speed: 1.0,
            time_step: SPF,
            input_vars,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, CsOut)]
pub struct DeriveKeeping {
    pub action_id: TmplID,
    pub derive_level: u16,
    pub end_time: f32,
}

impl DeriveKeeping {
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.action_id.is_valid()
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.action_id.is_invalid()
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

rkyv_self!(DeriveKeeping);
