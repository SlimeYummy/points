use approx::abs_diff_eq;
use critical_point_csgen::{CsEnum, CsOut};
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::alloc::Layout;
use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

use crate::consts::{INVALID_AI_TASK_ID, SPF};
use crate::instance::{InstActionAny, InstAiTaskAny};
use crate::logic::character::{LogicCharaControl, LogicCharaPhysics};
use crate::logic::game::ContextUpdate;
use crate::logic::system::input::WorldMoveState;
use crate::logic::zone::LogicZone;
use crate::utils::{AiTaskType, NumID, TmplID, VirtualKey, XResult, interface, rkyv_self};

//
// StateAiTaskAny & StateAiTaskBase
//

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
pub struct StateAiTaskBase {
    pub tmpl_id: TmplID,
    pub id: u32,
    pub typ: AiTaskType,
    pub status: LogicAiTaskStatus,
    pub first_frame: u32,
    pub last_frame: u32,
}

interface!(StateAiTaskAny, StateAiTaskBase);

#[cfg(feature = "debug-print")]
impl Drop for StateAiTaskBase {
    fn drop(&mut self) {
        log::debug!("StateAiTaskBase::drop() id={} tmpl_id={}", self.id, self.tmpl_id);
    }
}

impl StateAiTaskBase {
    pub fn new(typ: AiTaskType) -> StateAiTaskBase {
        StateAiTaskBase {
            tmpl_id: TmplID::default(),
            id: INVALID_AI_TASK_ID,
            typ,
            status: LogicAiTaskStatus::Starting,
            first_frame: 0,
            last_frame: 0,
        }
    }
}

#[typetag::serde(tag = "T")]
pub unsafe trait StateAiTaskAny
where
    Self: Debug + Any + Send + Sync,
{
    fn id(&self) -> u32;
    fn typ(&self) -> AiTaskType;
    fn layout(&self) -> Layout;
}

pub trait ArchivedStateAiTaskAny: Debug + Any {
    fn id(&self) -> u32;
    fn typ(&self) -> AiTaskType;
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, rkyv::Portable)]
pub struct StateAiTaskAnyMetadata(rkyv::primitive::ArchivedU16);

impl Default for StateAiTaskAnyMetadata {
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

    use crate::logic::ai_task::idle::{ArchivedStateAiTaskIdle, StateAiTaskIdle};
    use crate::logic::ai_task::patrol::{ArchivedStateAiTaskPatrol, StateAiTaskPatrol};
    use crate::utils::Castable;
    use AiTaskType::*;

    impl PartialEq for dyn StateAiTaskAny {
        fn eq(&self, other: &Self) -> bool {
            match (self.typ(), other.typ()) {
                (Idle, Idle) => unsafe {
                    self.cast_unchecked::<StateAiTaskIdle>() == other.cast_unchecked::<StateAiTaskIdle>()
                },
                (Patrol, Patrol) => unsafe {
                    self.cast_unchecked::<StateAiTaskPatrol>() == other.cast_unchecked::<StateAiTaskPatrol>()
                },
                _ => false,
            }
        }
    }

    impl LayoutRaw for dyn StateAiTaskAny {
        fn layout_raw(metadata: DynMetadata<dyn StateAiTaskAny>) -> Result<Layout, LayoutError> {
            unsafe {
                let null = ptr::from_raw_parts::<dyn StateAiTaskAny>(ptr::null() as *const u8, metadata);
                Ok((*null).layout())
            }
        }
    }

    unsafe impl Pointee for dyn StateAiTaskAny {
        type Metadata = DynMetadata<dyn StateAiTaskAny>;
    }

    unsafe impl Pointee for dyn ArchivedStateAiTaskAny {
        type Metadata = DynMetadata<dyn ArchivedStateAiTaskAny>;
    }

    unsafe impl Portable for dyn ArchivedStateAiTaskAny {}

    unsafe impl NoUndef for StateAiTaskAnyMetadata {}

    impl ArchivePointee for dyn ArchivedStateAiTaskAny {
        type ArchivedMetadata = StateAiTaskAnyMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let typ = AiTaskType::try_from(archived.0).expect("Invalid AiTaskType");
            let archived_ref: &Self = unsafe {
                match typ {
                    Idle => mem::transmute_copy::<usize, &ArchivedStateAiTaskIdle>(&0),
                    Patrol => mem::transmute_copy::<usize, &ArchivedStateAiTaskPatrol>(&0),
                    _ => unreachable!("pointer_metadata() Invalid AiTaskType"),
                }
            };
            ptr::metadata(archived_ref)
        }
    }

    impl ArchiveUnsized for dyn StateAiTaskAny {
        type Archived = dyn ArchivedStateAiTaskAny;

        fn archived_metadata(&self) -> ArchivedMetadata<Self> {
            StateAiTaskAnyMetadata(self.typ().into())
        }
    }

    impl<S> SerializeUnsized<S> for dyn StateAiTaskAny
    where
        S: Fallible + Allocator + Writer + ?Sized,
        S::Error: Source,
    {
        fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
            #[inline(always)]
            fn serialize<T, S>(
                state_any: &(dyn StateAiTaskAny + 'static),
                serializer: &mut S,
            ) -> Result<usize, S::Error>
            where
                T: StateAiTaskAny + Serialize<S> + 'static,
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
                Idle => serialize::<StateAiTaskIdle, _>(self, serializer),
                Patrol => serialize::<StateAiTaskPatrol, _>(self, serializer),
                _ => unreachable!("serialize_unsized() Invalid AiTaskType"),
            }
        }
    }

    impl<D> DeserializeUnsized<dyn StateAiTaskAny, D> for dyn ArchivedStateAiTaskAny
    where
        D: Fallible + ?Sized,
        D::Error: Source,
    {
        unsafe fn deserialize_unsized(
            &self,
            deserializer: &mut D,
            out: *mut dyn StateAiTaskAny,
        ) -> Result<(), D::Error> {
            #[inline(always)]
            fn deserialize<T, D>(
                archived_any: &(dyn ArchivedStateAiTaskAny + 'static),
                deserializer: &mut D,
                out: *mut dyn StateAiTaskAny,
            ) -> Result<(), D::Error>
            where
                T: StateAiTaskAny + Archive + 'static,
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
                Idle => deserialize::<StateAiTaskIdle, _>(self, deserializer, out),
                Patrol => deserialize::<StateAiTaskPatrol, _>(self, deserializer, out),
                _ => unreachable!("deserialize_unsized() Invalid AiTaskType"),
            }
        }

        fn deserialize_metadata(&self) -> DynMetadata<dyn StateAiTaskAny> {
            let value_ref: &dyn StateAiTaskAny = unsafe {
                match self.typ() {
                    Idle => mem::transmute_copy::<usize, &StateAiTaskIdle>(&0),
                    Patrol => mem::transmute_copy::<usize, &StateAiTaskPatrol>(&0),
                    _ => unreachable!("deserialize_metadata() Invalid AiTaskType"),
                }
            };
            ptr::metadata(value_ref)
        }
    }
};

macro_rules! impl_state_ai_task {
    ($typ:ty, $state_enum:ident, $serde_tag:expr) => {
        paste::paste! {
            #[typetag::serde(name = $serde_tag)]
            unsafe impl $crate::logic::ai_task::StateAiTaskAny for $typ {
                #[inline]
                fn id(&self) -> u32 {
                    self._base.id
                }

                #[inline]
                fn typ(&self) -> $crate::utils::AiTaskType {
                    debug_assert_eq!(
                        self._base.typ,
                        $crate::utils::AiTaskType::$state_enum
                    );
                    $crate::utils::AiTaskType::$state_enum
                }

                #[inline]
                fn layout(&self) -> std::alloc::Layout {
                    std::alloc::Layout::new::<Self>()
                }
            }

            impl $crate::logic::ai_task::ArchivedStateAiTaskAny for [<Archived $typ>] {
                #[inline]
                fn id(&self) -> u32 {
                    self._base.id.to_native()
                }

                #[inline]
                fn typ(&self) -> $crate::utils::AiTaskType {
                    debug_assert_eq!(
                        self._base.typ,
                        $crate::utils::AiTaskType::$state_enum
                    );
                    $crate::utils::AiTaskType::$state_enum
                }
            }
        }
    };
}
pub(crate) use impl_state_ai_task;

//
// LogicAiTaskAny & LogicAiTaskBase
//

pub unsafe trait LogicAiTaskAny: Debug + Any {
    fn typ(&self) -> AiTaskType;
    fn save(&self) -> Box<dyn StateAiTaskAny>;
    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()>;

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicAiTaskBase) };
        base.start(ctx, ctxt)?;
        Ok(AiTaskReturn::default())
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn>;

    fn stop(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicAiTaskBase) };
        base.stop(ctx, ctxt)
    }

    fn finalize(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<()> {
        let (ptr, _) = (self as *mut Self).to_raw_parts();
        let base = unsafe { &mut *(ptr as *mut LogicAiTaskBase) };
        base.finalize(ctx, ctxt)
    }
}

pub struct ContextAiTask<'a> {
    pub(crate) chara_id: NumID,
    pub(crate) chara_ctrl: &'a LogicCharaControl,
    pub(crate) chara_phy: &'a LogicCharaPhysics,
    pub(crate) zone: &'a LogicZone,
    pub(crate) time_speed: f32,
    pub(crate) time_step: f32,
    pub(crate) frac_1_time_step: f32,
}

impl<'a> ContextAiTask<'a> {
    pub(crate) fn new(
        chara_id: NumID,
        chara_ctrl: &'a LogicCharaControl,
        chara_phy: &'a LogicCharaPhysics,
        zone: &'a LogicZone,
        mut time_speed: f32,
    ) -> ContextAiTask<'a> {
        let time_step;
        let frac_1_time_step;
        if abs_diff_eq!(time_speed, 0.0, epsilon = 1e-4) {
            time_speed = 0.0;
            time_step = 0.0;
            frac_1_time_step = 0.0;
        }
        else {
            time_step = SPF * time_speed;
            frac_1_time_step = 1.0 / time_step;
        }

        ContextAiTask {
            chara_id,
            chara_ctrl,
            chara_phy,
            zone,
            time_speed,
            time_step,
            frac_1_time_step,
        }
    }
}

#[derive(Debug, Default)]
pub struct AiTaskReturn {
    pub next_action: Option<Rc<dyn InstActionAny>>,
    pub view_dir_2d: Vec2xz,
    pub world_move: WorldMoveState,
    pub quick_switch: bool,
}

impl AiTaskReturn {
    #[inline]
    pub fn view_dir_3d(&self) -> Vec3A {
        Vec3A::new(self.view_dir_2d.x, 0.0, self.view_dir_2d.z)
    }
}

#[derive(Debug)]
pub struct LogicAiTaskBase {
    pub id: u32,
    pub inst: Rc<dyn InstAiTaskAny>,
    pub status: LogicAiTaskStatus,
    pub first_frame: u32,
    pub last_frame: u32,
}

interface!(LogicAiTaskAny, LogicAiTaskBase);

impl LogicAiTaskBase {
    pub fn new(id: u32, inst: Rc<dyn InstAiTaskAny>) -> LogicAiTaskBase {
        LogicAiTaskBase {
            id,
            inst,
            status: LogicAiTaskStatus::Starting,
            first_frame: 0,
            last_frame: u32::MAX,
        }
    }

    pub fn save(&self, typ: AiTaskType) -> StateAiTaskBase {
        StateAiTaskBase {
            id: self.id,
            tmpl_id: TmplID::default(),
            typ,
            status: self.status,
            first_frame: self.first_frame,
            last_frame: self.last_frame,
        }
    }

    pub fn restore(&mut self, state: &StateAiTaskBase) {
        self.status = state.status;
        self.first_frame = state.first_frame;
        self.last_frame = state.last_frame;
    }

    pub fn start(&mut self, ctx: &ContextUpdate, _ctxt: &mut ContextAiTask) -> XResult<()> {
        self.status = LogicAiTaskStatus::Running;
        self.first_frame = ctx.frame;
        Ok(())
    }

    pub fn update(&mut self, _ctx: &ContextUpdate, _ctxt: &mut ContextAiTask) -> XResult<()> {
        Ok(())
    }

    pub fn stop(&mut self, _ctx: &ContextUpdate, _ctxt: &mut ContextAiTask) -> XResult<()> {
        self.status = LogicAiTaskStatus::Stopping;
        Ok(())
    }

    pub fn finalize(&mut self, ctx: &ContextUpdate, _ctxt: &mut ContextAiTask) -> XResult<()> {
        self.status = LogicAiTaskStatus::Finalized;
        self.last_frame = ctx.frame;
        Ok(())
    }

    #[inline]
    pub fn is_starting(&self) -> bool {
        self.status == LogicAiTaskStatus::Starting
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        self.status == LogicAiTaskStatus::Running
    }

    #[inline]
    pub fn is_stopping(&self) -> bool {
        self.status == LogicAiTaskStatus::Stopping
    }

    #[inline]
    pub fn is_finalized(&self) -> bool {
        self.status == LogicAiTaskStatus::Finalized
    }
}

//
// Others
//

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize, CsEnum)]
pub enum LogicAiTaskStatus {
    Starting,
    Running,
    Stopping,
    Finalized,
}

rkyv_self!(LogicAiTaskStatus);
