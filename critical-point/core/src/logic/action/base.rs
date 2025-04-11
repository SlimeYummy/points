use cirtical_point_csgen::{CsEnum, CsOut};
use enum_iterator::{cardinality, Sequence};
use glam::{Quat, Vec2, Vec3A};
use std::fmt::Debug;
use std::mem;

use crate::consts::{MAX_ACTION_ANIMATION, WEIGHT_THRESHOLD};
use crate::logic::character::LogicCharaPhysics;
use crate::logic::game::ContextUpdate;
use crate::logic::system::input::InputVariables;
use crate::template::TmplType;
use crate::utils::{
    calc_ratio_clamp, interface, to_euler_degree, xres, AStrID, ASymbol, Castable, NumID, StrID, XError, XResult,
};

//
// StateAction & StateActionBase
//

#[repr(u16)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Hash,
    Sequence,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsEnum,
)]
#[archive_attr(derive(Debug))]
pub enum StateActionType {
    Idle,
    Move,
}

impl StateActionType {
    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        match self {
            StateActionType::Idle => TmplType::ActionIdle,
            StateActionType::Move => TmplType::ActionMove,
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

pub unsafe trait StateAction
where
    Self: Debug + Send + Sync,
{
    fn typ(&self) -> StateActionType;
    fn tmpl_typ(&self) -> TmplType;
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionBase {
    pub id: NumID,
    pub tmpl_id: AStrID,
    pub typ: StateActionType,
    pub tmpl_typ: TmplType,

    pub spawn_frame: u32,
    pub death_frame: u32,
    pub enter_progress: u32,
    pub is_leaving: bool,

    pub event_idx: u64,
    pub derive_level: u16,
    pub antibreak_level: u16,

    pub body_ratio: f32,
    pub animations: [StateActionAnimation; MAX_ACTION_ANIMATION],
}

interface!(StateAction, StateActionBase);

#[cfg(feature = "debug-print")]
impl Drop for StateActionBase {
    fn drop(&mut self) {
        println!("StateActionBase drop() {} {}", self.id, self.tmpl_id);
    }
}

impl StateActionBase {
    pub fn new(typ: StateActionType, tmpl_typ: TmplType) -> StateActionBase {
        StateActionBase {
            id: 0,
            tmpl_id: AStrID::default(),
            typ,
            tmpl_typ,

            spawn_frame: 0,
            death_frame: 0,
            enter_progress: 0,
            is_leaving: false,

            event_idx: 0,
            derive_level: 0,
            antibreak_level: 0,

            body_ratio: 0.0,
            animations: Default::default(),
        }
    }
}

pub trait ArchivedStateAction: Debug {
    fn typ(&self) -> StateActionType;
    fn tmpl_typ(&self) -> TmplType;
}

impl Castable for dyn ArchivedStateAction {}

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
pub struct StateActionAnimation {
    pub file: ASymbol,
    pub animation_id: u32,
    pub ratio: f32,
    pub weight: f32,
}

impl StateActionAnimation {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.file.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateActionMetadata {
    pub typ: rkyv::Archived<u16>,
}

const _: () = {
    use ptr_meta::Pointee;
    use rkyv::ser::{ScratchSpace, Serializer};
    use rkyv::{
        to_archived, Archive, ArchivePointee, ArchiveUnsized, Archived, ArchivedMetadata, Deserialize,
        DeserializeUnsized, Fallible, Serialize, SerializeUnsized,
    };
    use std::alloc::Layout;
    use std::ptr::DynMetadata;
    use std::{mem, ptr};

    use crate::logic::action::idle::{ArchivedStateActionIdle, StateActionIdle};
    use crate::logic::action::r#move::{ArchivedStateActionMove, StateActionMove};
    use crate::utils::CastRef;
    use StateActionType::*;

    impl PartialEq for dyn StateAction {
        fn eq(&self, other: &Self) -> bool {
            match (self.typ(), other.typ()) {
                (StateActionType::Idle, StateActionType::Idle) => unsafe {
                    self.cast_ref_unchecked::<StateActionIdle>() == other.cast_ref_unchecked::<StateActionIdle>()
                },
                (StateActionType::Move, StateActionType::Move) => unsafe {
                    self.cast_ref_unchecked::<StateActionMove>() == other.cast_ref_unchecked::<StateActionMove>()
                },
                _ => false,
            }
        }
    }

    impl Pointee for dyn StateAction {
        type Metadata = DynMetadata<dyn StateAction>;
    }

    impl Pointee for dyn ArchivedStateAction {
        type Metadata = DynMetadata<dyn ArchivedStateAction>;
    }

    impl ArchivePointee for dyn ArchivedStateAction {
        type ArchivedMetadata = StateActionMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let typ = StateActionType::try_from(archived.typ).expect("Invalid StateActionType");
            let archived_ref: &Self = unsafe {
                match typ {
                    Idle => mem::transmute_copy::<usize, &ArchivedStateActionIdle>(&0),
                    Move => mem::transmute_copy::<usize, &ArchivedStateActionMove>(&0),
                    // _ => panic!("Invalid StateActionType"),
                }
            };
            ptr::metadata(archived_ref)
        }
    }

    impl ArchiveUnsized for dyn StateAction {
        type Archived = dyn ArchivedStateAction;
        type MetadataResolver = ();

        unsafe fn resolve_metadata(
            &self,
            _pos: usize,
            _resolver: Self::MetadataResolver,
            out: *mut ArchivedMetadata<Self>,
        ) {
            let typ = to_archived!(self.typ().into());
            out.write(StateActionMetadata { typ });
        }
    }

    impl<S> SerializeUnsized<S> for dyn StateAction
    where
        S: Serializer + ScratchSpace + ?Sized,
    {
        fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
            #[inline(always)]
            fn serialize<T, S>(state_any: &(dyn StateAction + 'static), serializer: &mut S) -> Result<usize, S::Error>
            where
                T: StateAction + Serialize<S> + 'static,
                S: Serializer + ScratchSpace + ?Sized,
            {
                let state_ref = unsafe { state_any.cast_ref_unchecked::<T>() };
                let resolver = state_ref.serialize(serializer)?;
                serializer.align_for::<T>()?;
                Ok(unsafe { serializer.resolve_aligned(state_ref, resolver)? })
            }

            match self.typ() {
                Idle => serialize::<StateActionIdle, _>(self, serializer),
                Move => serialize::<StateActionMove, _>(self, serializer),
                // _ => panic!("Invalid StateActionType"),
            }
        }

        fn serialize_metadata(&self, _serializer: &mut S) -> Result<Self::MetadataResolver, S::Error> {
            Ok(())
        }
    }

    impl<D> DeserializeUnsized<dyn StateAction, D> for dyn ArchivedStateAction
    where
        D: Fallible + ?Sized,
    {
        unsafe fn deserialize_unsized(
            &self,
            deserializer: &mut D,
            alloc: impl FnMut(Layout) -> *mut u8,
        ) -> Result<*mut (), D::Error> {
            #[inline(always)]
            fn deserialize<T, D>(
                archived_any: &(dyn ArchivedStateAction + 'static),
                deserializer: &mut D,
                mut alloc: impl FnMut(Layout) -> *mut u8,
            ) -> Result<*mut (), D::Error>
            where
                T: StateAction + Archive + 'static,
                D: Fallible + ?Sized,
                Archived<T>: Deserialize<T, D>,
            {
                let pointer = alloc(Layout::new::<T>()) as *mut T;
                let archived_ref: &Archived<T> = unsafe { archived_any.cast_ref_unchecked() };
                let value: T = archived_ref.deserialize(deserializer)?;
                unsafe { pointer.write(value) };
                Ok(pointer as *mut ())
            }

            match self.typ() {
                Idle => deserialize::<StateActionIdle, _>(self, deserializer, alloc),
                Move => deserialize::<StateActionMove, _>(self, deserializer, alloc),
                // _ => panic!("Invalid TmplType"),
            }
        }

        fn deserialize_metadata(&self, _deserializer: &mut D) -> Result<DynMetadata<dyn StateAction>, D::Error> {
            let value_ref: &dyn StateAction = unsafe {
                match self.typ() {
                    Idle => mem::transmute_copy::<usize, &StateActionIdle>(&0),
                    Move => mem::transmute_copy::<usize, &StateActionMove>(&0),
                    // _ => panic!("Invalid TmplType"),
                }
            };
            Ok(ptr::metadata(value_ref))
        }
    }
};

//
// LogicAction & LogicActionBase
//

pub unsafe trait LogicAction
where
    Self: Debug,
{
    fn typ(&self) -> StateActionType;
    fn tmpl_typ(&self) -> TmplType;
    fn restore(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()>;
    fn update(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        ctxa: &mut ContextAction<'_>,
    ) -> XResult<Option<Box<dyn StateAction>>>;
}

#[derive(Debug)]
pub struct LogicActionBase {
    pub id: NumID,
    pub tmpl_id: StrID,

    pub spawn_frame: u32,
    pub death_frame: u32,
    pub enter_progress: u32,
    pub is_leaving: bool,

    pub event_idx: u64,
    pub derive_level: u16,
    pub antibreak_level: u16,

    pub body_ratio: f32,
}

interface!(LogicAction, LogicActionBase);

impl LogicActionBase {
    pub fn new(id: NumID, tmpl_id: StrID, spawn_frame: u32) -> LogicActionBase {
        LogicActionBase {
            id,
            tmpl_id,
            spawn_frame,
            death_frame: u32::MAX,
            enter_progress: 0,
            is_leaving: false,
            event_idx: 0,
            derive_level: 0,
            antibreak_level: 0,
            body_ratio: 0.0,
        }
    }

    pub fn reuse(&mut self, id: NumID, tmpl_id: StrID, spawn_frame: u32) -> XResult<()> {
        self.id = id;
        self.tmpl_id = tmpl_id;

        self.spawn_frame = spawn_frame;
        self.death_frame = u32::MAX;
        self.enter_progress = 0;
        self.is_leaving = false;

        self.derive_level = 0;
        self.antibreak_level = 0;

        self.body_ratio = 0.0;
        Ok(())
    }

    pub fn save(&self, typ: StateActionType, tmpl_typ: TmplType) -> StateActionBase {
        StateActionBase {
            id: self.id,
            tmpl_id: ASymbol::from(&self.tmpl_id),
            typ,
            tmpl_typ,

            spawn_frame: self.spawn_frame,
            death_frame: self.death_frame,
            enter_progress: self.enter_progress,
            is_leaving: self.is_leaving,

            event_idx: self.event_idx,
            derive_level: self.derive_level,
            antibreak_level: self.antibreak_level,

            body_ratio: self.body_ratio,
            animations: Default::default(),
        }
    }

    pub fn restore(&mut self, state: &StateActionBase) {
        self.spawn_frame = state.spawn_frame;
        self.death_frame = state.death_frame;
        self.enter_progress = state.enter_progress;
        self.is_leaving = state.is_leaving;

        self.event_idx = state.event_idx;
        self.derive_level = state.derive_level;
        self.antibreak_level = state.antibreak_level;

        self.body_ratio = state.body_ratio;
    }

    // Return None if self action finished
    pub fn handle_enter_leave(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        ctxa: &mut ContextAction<'_>,
        enter_time: u32,
    ) -> Option<f32> {
        let local_weight = match ctxa.prev_action {
            None => 1.0,
            Some(_) => {
                self.enter_progress += 1;
                calc_ratio_clamp(self.enter_progress, enter_time)
            }
        };
        let real_weight = ctxa.apply_weight(local_weight);
        self.is_leaving = self.is_leaving || ctxa.next_action.is_some();
        if self.is_leaving && real_weight < WEIGHT_THRESHOLD {
            self.death_frame = ctx.frame; // action finished
            return None;
        }
        Some(real_weight)
    }
}

//
// ContextAction
//

pub struct ContextAction<'t> {
    pub player_id: NumID,
    pub chara_physics: &'t LogicCharaPhysics,
    pub next_action: Option<&'t (dyn LogicAction + 'static)>,
    pub prev_action: Option<&'t (dyn LogicAction + 'static)>,
    pub unused_weight: f32,
    pub input_vars: InputVariables,
    pub new_velocity: Vec3A,
    pub new_rotation: Quat,
}

impl<'t> ContextAction<'t> {
    pub fn new(
        player_id: NumID,
        chara_physics: &'t LogicCharaPhysics,
        input_vars: InputVariables,
    ) -> ContextAction<'t> {
        ContextAction {
            player_id,
            chara_physics,
            next_action: None,
            prev_action: None,
            unused_weight: 1.0,
            input_vars,
            new_velocity: Vec3A::ZERO,
            new_rotation: Quat::IDENTITY,
        }
    }

    // return real weight for callee action
    #[inline]
    pub fn apply_weight(&mut self, weight: f32) -> f32 {
        let normalized_weight = weight.clamp(0.0, 1.0);
        let real_weight = normalized_weight * self.unused_weight;
        self.unused_weight *= 1.0 - normalized_weight;
        real_weight
    }

    #[inline]
    pub fn set_new_velocity(&mut self, move_dir: Vec2) {
        self.new_velocity = Vec3A::new(move_dir.x, 0.0, move_dir.y);
    }

    #[inline]
    pub fn set_new_rotation(&mut self, chara_dir: Vec2) {
        let rot = Quat::from_rotation_arc_2d(Vec2::Y, chara_dir);
        self.new_rotation = Quat::from_xyzw(0.0, -rot.z, 0.0, rot.w);
        println!("self.new_rotation {:?}", to_euler_degree(self.new_rotation));
    }
}

//
// utils
//

#[macro_export]
macro_rules! continue_to {
    ($mode:expr, $next:expr) => {{
        $mode = $next;
        continue;
    }};
}
pub(crate) use continue_to;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::action::idle::{ActionIdleMode, StateActionIdle};
    use crate::utils::{asb, CastPtr};
    use anyhow::Result;
    use rkyv::ser::serializers::AllocSerializer;
    use rkyv::ser::Serializer;
    use rkyv::{Deserialize, Infallible};

    fn test_rkyv(state: Box<dyn StateAction>, typ: StateActionType) -> Result<Box<dyn StateAction>> {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(&state)?;
        let buffer = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<Box<dyn StateAction>>(&buffer) };
        assert_eq!(archived.typ(), typ);

        let mut deserializer = Infallible;
        let result: Box<dyn StateAction> = archived.deserialize(&mut deserializer)?;
        assert_eq!(result.typ(), typ);

        Ok(result)
    }

    #[test]
    fn test_rkyv_state_tmpl_idle() {
        let mut raw_state = Box::new(StateActionIdle {
            _base: StateActionBase::new(StateActionType::Idle, TmplType::ActionIdle),
            mode: ActionIdleMode::IdleToReady,
            idle_progress: 10,
            ready_progress: 20,
            idle_timer: 12,
            switch_progress: 5,
        });
        raw_state.id = 123;
        raw_state.tmpl_id = asb!("idle");
        raw_state.spawn_frame = 99;
        raw_state.death_frame = 1100;
        raw_state.is_leaving = false;
        raw_state.enter_progress = 57;
        raw_state.event_idx = 0;
        raw_state.derive_level = 1;
        raw_state.antibreak_level = 2;
        raw_state.body_ratio = 0.8;
        raw_state.animations[0] = StateActionAnimation {
            animation_id: 1,
            file: asb!("idle.ozz"),
            ratio: 0.5,
            weight: 0.5,
        };

        let state = test_rkyv(raw_state, StateActionType::Idle).unwrap();
        let state = state.cast_as::<StateActionIdle>().unwrap();

        assert_eq!(state.id, 123);
        assert_eq!(state.tmpl_id, asb!("idle"));
        assert_eq!(state.spawn_frame, 99);
        assert_eq!(state.death_frame, 1100);
        assert_eq!(state.derive_level, 1);
        assert_eq!(state.antibreak_level, 2);
        assert_eq!(state.body_ratio, 0.8);
        assert_eq!(state.animations[0].animation_id, 1);
        assert_eq!(state.animations[0].file, asb!("idle.ozz"));
        assert_eq!(state.animations[0].ratio, 0.5);
        assert_eq!(state.animations[0].weight, 0.5);
        assert_eq!(state.animations[1], StateActionAnimation::default());
        assert_eq!(state.animations[2], StateActionAnimation::default());
        assert_eq!(state.animations[3], StateActionAnimation::default());
        assert_eq!(state.event_idx, 0);
        assert_eq!(state.mode, ActionIdleMode::IdleToReady);
        assert!(!state.is_leaving);
        assert_eq!(state.enter_progress, 57);
        assert_eq!(state.idle_progress, 10);
        assert_eq!(state.ready_progress, 20);
        assert_eq!(state.idle_timer, 12);
        assert_eq!(state.switch_progress, 5);
    }
}
