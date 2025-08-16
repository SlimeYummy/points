use cirtical_point_csgen::{CsEnum, CsOut};
use enum_iterator::{cardinality, Sequence};
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::alloc::Layout;
use std::any::Any;
use std::fmt::Debug;
use std::hint::unlikely;
use std::rc::Rc;
use std::{mem, u32};

use crate::consts::{FPS, MAX_ACTION_ANIMATION, SPF};
use crate::instance::InstActionAny;
use crate::logic::character::LogicCharaPhysics;
use crate::logic::game::ContextUpdate;
use crate::logic::system::input::InputVariables;
use crate::template::TmplType;
use crate::utils::{interface, rkyv_self, xres, NumID, Symbol, TmplID, XError, XResult};

#[repr(u16)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Sequence, serde::Serialize, serde::Deserialize, CsEnum,
)]
pub enum StateActionType {
    Empty,
    Idle,
    Move,
    General,
}

rkyv_self!(StateActionType);

impl StateActionType {
    #[inline]
    pub fn tmpl_typ(&self) -> TmplType {
        match self {
            StateActionType::Empty => TmplType::ActionEmpty,
            StateActionType::Idle => TmplType::ActionIdle,
            StateActionType::Move => TmplType::ActionMove,
            StateActionType::General => TmplType::ActionGeneral,
        }
    }
}

impl From<StateActionType> for u16 {
    #[inline]
    fn from(val: StateActionType) -> Self {
        unsafe { mem::transmute::<StateActionType, u16>(val) }
    }
}

impl TryFrom<u16> for StateActionType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> XResult<Self> {
        if value as usize >= cardinality::<StateActionType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, StateActionType>(value) })
    }
}

impl From<StateActionType> for rkyv::primitive::ArchivedU16 {
    #[inline]
    fn from(val: StateActionType) -> Self {
        unsafe { mem::transmute::<StateActionType, u16>(val) }.into()
    }
}

impl TryFrom<rkyv::primitive::ArchivedU16> for StateActionType {
    type Error = XError;

    #[inline]
    fn try_from(val: rkyv::primitive::ArchivedU16) -> XResult<Self> {
        if val.to_native() as usize >= cardinality::<StateActionType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, StateActionType>(val.to_native()) })
    }
}

//
// StateActionAny & StateActionBase
//

#[typetag::serde(tag = "T")]
pub unsafe trait StateActionAny
where
    Self: Debug + Any + Send + Sync,
{
    fn id(&self) -> NumID;
    fn typ(&self) -> StateActionType;
    fn tmpl_typ(&self) -> TmplType;
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
    pub typ: StateActionType,
    pub tmpl_typ: TmplType,
    pub status: LogicActionStatus,
    pub first_frame: u32,
    pub last_frame: u32,
    pub fade_in_weight: f32,
    pub derive_level: u16,
    pub poise_level: u16,
    pub animations: [StateActionAnimation; MAX_ACTION_ANIMATION],
}

interface!(StateActionAny, StateActionBase);

#[cfg(feature = "debug-print")]
impl Drop for StateActionBase {
    fn drop(&mut self) {
        log::debug!("StateActionBase::drop() id={} tmpl_id={}", self.id, self.tmpl_id);
    }
}

impl StateActionBase {
    pub fn new(typ: StateActionType, tmpl_typ: TmplType) -> StateActionBase {
        StateActionBase {
            id: 0,
            tmpl_id: TmplID::default(),
            typ,
            tmpl_typ,
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
    fn typ(&self) -> StateActionType;
    fn tmpl_typ(&self) -> TmplType;
}

#[repr(C)]
#[derive(
    Debug,
    Default,
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
pub struct StateActionAnimation {
    pub files: Symbol,
    pub animation_id: u16,
    pub ratio: f32,
    pub weight: f32,
}

impl StateActionAnimation {
    #[inline]
    pub fn new(files: Symbol, animation_id: u16, ratio: f32, weight: f32) -> Self {
        StateActionAnimation {
            files,
            animation_id,
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
    use StateActionType::*;

    impl PartialEq for dyn StateActionAny {
        fn eq(&self, other: &Self) -> bool {
            match (self.typ(), other.typ()) {
                (StateActionType::Empty, StateActionType::Empty) => unsafe {
                    self.cast_unchecked::<StateActionEmpty>() == other.cast_unchecked::<StateActionEmpty>()
                },
                (StateActionType::Idle, StateActionType::Idle) => unsafe {
                    self.cast_unchecked::<StateActionIdle>() == other.cast_unchecked::<StateActionIdle>()
                },
                (StateActionType::Move, StateActionType::Move) => unsafe {
                    self.cast_unchecked::<StateActionMove>() == other.cast_unchecked::<StateActionMove>()
                },
                (StateActionType::General, StateActionType::General) => unsafe {
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
            let typ = StateActionType::try_from(archived.0).expect("Invalid StateActionType");
            let archived_ref: &Self = unsafe {
                match typ {
                    Empty => mem::transmute_copy::<usize, &ArchivedStateActionEmpty>(&0),
                    Idle => mem::transmute_copy::<usize, &ArchivedStateActionIdle>(&0),
                    Move => mem::transmute_copy::<usize, &ArchivedStateActionMove>(&0),
                    General => mem::transmute_copy::<usize, &ArchivedStateActionGeneral>(&0),
                    _ => unreachable!("pointer_metadata() Invalid StateActionType"),
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
                _ => unreachable!("serialize_unsized() Invalid StateActionType"),
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
                _ => unreachable!("deserialize_unsized() Invalid StateActionType"),
            }
        }

        fn deserialize_metadata(&self) -> DynMetadata<dyn StateActionAny> {
            let value_ref: &dyn StateActionAny = unsafe {
                match self.typ() {
                    Empty => mem::transmute_copy::<usize, &StateActionEmpty>(&0),
                    Idle => mem::transmute_copy::<usize, &StateActionIdle>(&0),
                    Move => mem::transmute_copy::<usize, &StateActionMove>(&0),
                    General => mem::transmute_copy::<usize, &StateActionGeneral>(&0),
                    _ => unreachable!("deserialize_metadata() Invalid StateActionType"),
                }
            };
            ptr::metadata(value_ref)
        }
    }
};

macro_rules! impl_state_action {
    ($typ:ty, $tmpl_enum:ident, $state_enum:ident, $serde_tag:expr) => {
        paste::paste! {
            #[typetag::serde(name = $serde_tag)]
            unsafe impl $crate::logic::action::StateActionAny for $typ {
                #[inline]
                fn id(&self) -> $crate::utils::NumID {
                    self._base.id
                }

                #[inline]
                fn typ(&self) -> $crate::logic::action::StateActionType {
                    debug_assert_eq!(
                        self._base.typ,
                        $crate::logic::action::StateActionType::$state_enum
                    );
                    $crate::logic::action::StateActionType::$state_enum
                }

                #[inline]
                fn tmpl_typ(&self) -> $crate::template::TmplType {
                    debug_assert_eq!(self._base.tmpl_typ, $crate::template::TmplType::$tmpl_enum);
                    $crate::template::TmplType::$tmpl_enum
                }

                #[inline]
                fn layout(&self) -> std::alloc::Layout {
                    std::alloc::Layout::new::<Self>()
                }
            }

            impl $crate::logic::action::ArchivedStateActionAny for [<Archived $typ>] {
                #[inline]
                fn id(&self) -> crate::utils::NumID {
                    self._base.id.to_native()
                }

                #[inline]
                fn typ(&self) -> $crate::logic::action::StateActionType {
                    debug_assert_eq!(
                        self._base.typ,
                        $crate::logic::action::StateActionType::$state_enum
                    );
                    $crate::logic::action::StateActionType::$state_enum
                }

                #[inline]
                fn tmpl_typ(&self) -> $crate::template::TmplType {
                    debug_assert_eq!(self._base.tmpl_typ, $crate::template::TmplType::$tmpl_enum);
                    $crate::template::TmplType::$tmpl_enum
                }
            }
        }
    };
}
pub(crate) use impl_state_action;

//
// LogicActionAny & LogicActionBase
//

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize, CsEnum)]
pub enum LogicActionStatus {
    Starting,
    Activing,
    Stopping,
    Finalized,
}

rkyv_self!(LogicActionStatus);

pub unsafe trait LogicActionAny: Debug {
    fn typ(&self) -> StateActionType;
    fn tmpl_typ(&self) -> TmplType;
    fn save(&self) -> Box<dyn StateActionAny>;
    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()>;

    fn start(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.start(ctx, ctxa)
    }

    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<ActionUpdateReturn>;

    fn stop(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.stop(ctx, ctxa)
    }

    fn finalize(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicActionBase) };
        base.finalize(ctx, ctxa)
    }
}

#[derive(Debug)]
pub struct LogicActionBase {
    pub id: NumID,
    pub inst: Rc<dyn InstActionAny>,
    pub status: LogicActionStatus,
    pub first_frame: u32,
    pub last_frame: u32,
    pub fade_in_weight: f32,
    pub derive_level: u16,
    pub poise_level: u16,
}

interface!(LogicActionAny, LogicActionBase);

#[derive(Debug, Default)]
pub struct ActionUpdateReturn {
    pub new_velocity: Option<Vec3A>,
    pub new_direction: Option<Vec2xz>,
    pub derive_keeping: Option<DeriveKeeping>,
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

impl LogicActionBase {
    pub fn new(id: NumID, inst: Rc<dyn InstActionAny>) -> LogicActionBase {
        LogicActionBase {
            id,
            inst,
            status: LogicActionStatus::Starting,
            first_frame: 0,
            last_frame: u32::MAX,
            fade_in_weight: 0.0,
            derive_level: 0,
            poise_level: 0,
        }
    }

    pub fn reuse(&mut self, id: NumID) -> XResult<()> {
        *self = LogicActionBase::new(id, self.inst.clone());
        Ok(())
    }

    pub fn save(&self, typ: StateActionType, tmpl_typ: TmplType) -> StateActionBase {
        StateActionBase {
            id: self.id,
            tmpl_id: self.inst.tmpl_id,
            typ,
            tmpl_typ,
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

    pub fn start(&mut self, ctx: &ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Starting) {
            return xres!(Unexpected; "status != Starting");
        }
        self.status = LogicActionStatus::Activing;
        self.first_frame = ctx.frame;
        if ctxa.prev_action.is_none() {
            self.fade_in_weight = 1.0;
        }
        Ok(())
    }

    pub fn update(&mut self, _ctx: &ContextUpdate<'_>, _ctxa: &mut ContextAction<'_>) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Activing) {
            return xres!(Unexpected; "status != Activing");
        }
        Ok(())
    }

    pub fn stop(&mut self, _ctx: &ContextUpdate<'_>, _ctxa: &mut ContextAction<'_>) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Activing) {
            return xres!(Unexpected; "status != Activing");
        }
        self.status = LogicActionStatus::Stopping;
        Ok(())
    }

    pub fn finalize(&mut self, ctx: &ContextUpdate<'_>, _ctxa: &mut ContextAction<'_>) -> XResult<()> {
        if unlikely(self.status != LogicActionStatus::Stopping) {
            return xres!(Unexpected; "status != Stopping");
        }
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
    pub fn is_finalized(&self) -> bool {
        self.status == LogicActionStatus::Finalized
    }
}

//
// ContextAction
//

pub struct ContextAction<'t> {
    pub player_id: NumID,
    pub chara_physics: &'t LogicCharaPhysics,
    pub prev_action: Option<Rc<dyn InstActionAny>>,
    pub input_vars: InputVariables,
    pub time_speed: f32,
    pub time_step: f32,
}

impl<'t> ContextAction<'t> {
    pub(crate) fn new(
        player_id: NumID,
        chara_physics: &'t LogicCharaPhysics,
        input_vars: InputVariables,
    ) -> ContextAction<'t> {
        ContextAction {
            player_id,
            chara_physics,
            prev_action: None,
            time_speed: 1.0,
            time_step: SPF,
            input_vars,
        }
    }

    #[inline]
    pub(crate) fn set_time_speed(&mut self, time_speed: f32) {
        self.time_speed = time_speed;
        self.time_step = time_speed / FPS;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeriveKeeping {
    pub action_id: TmplID,
    pub derive_level: u16,
    pub end_time: f32,
}

rkyv_self!(DeriveKeeping);

//
// utils
//

#[macro_export]
macro_rules! continue_mode {
    ($mode:expr, $next:expr) => {{
        $mode = $next;
        continue;
    }};
}
pub(crate) use continue_mode;
