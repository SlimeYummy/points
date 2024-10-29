use cirtical_point_csgen::CsGen;
use std::fmt::Debug;
use std::rc::Rc;
use std::u32;

use crate::instance::{InstAction, InstPlayer};
use crate::logic::game::ContextUpdate;
use crate::template::TmplClass;
use crate::utils::{interface, Castable, NumID, StrID, Symbol, XResult};

pub const MAX_ACTION_ANIMATION: usize = 4;
pub const WEIGHT_THRESHOLD: f32 = 0.01;

//
// StateAction & StateActionBase
//

pub unsafe trait StateAction
where
    Self: Debug + Send + Sync,
{
    fn class(&self) -> TmplClass;
}

#[repr(C)]
#[derive(Debug, Default, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen)]
#[archive_attr(derive(Debug))]
#[cs_attr(Rs, Ref)]
pub struct StateActionBase {
    pub id: NumID,
    pub tmpl_id: StrID,
    pub spawn_frame: u32,
    pub dead_frame: u32,
    pub derive_level: u16,
    pub antibreak_level: u16,
    pub blend_weight: f32,
    pub body_ratio: f32,
    pub animations: [StateActionAnimation; MAX_ACTION_ANIMATION],
}

interface!(StateAction, StateActionBase);

pub trait ArchivedStateAction: Debug {
    fn class(&self) -> TmplClass;
}

impl Castable for dyn ArchivedStateAction {}

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen)]
#[archive_attr(derive(Debug))]
#[cs_attr(Rs)]
pub struct StateActionAnimation {
    pub file: Symbol,
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
    pub class: rkyv::Archived<u16>,
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
    use crate::utils::CastRef;
    use TmplClass::*;

    impl Pointee for dyn StateAction {
        type Metadata = DynMetadata<dyn StateAction>;
    }

    impl Pointee for dyn ArchivedStateAction {
        type Metadata = DynMetadata<dyn ArchivedStateAction>;
    }

    impl ArchivePointee for dyn ArchivedStateAction {
        type ArchivedMetadata = StateActionMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let class = TmplClass::try_from(archived.class).expect("Invalid TmplClass");
            let archived_ref: &Self = unsafe {
                match class {
                    ActionIdle => mem::transmute_copy::<usize, &ArchivedStateActionIdle>(&0),
                    _ => panic!("Invalid TmplClass"),
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
            let class = to_archived!(self.class().into());
            out.write(StateActionMetadata { class });
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

            match self.class() {
                ActionIdle => serialize::<StateActionIdle, _>(self, serializer),
                _ => panic!("Invalid TmplClass"),
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

            match self.class() {
                ActionIdle => deserialize::<StateActionIdle, _>(self, deserializer, alloc),
                _ => panic!("Invalid TmplClass"),
            }
        }

        fn deserialize_metadata(&self, _deserializer: &mut D) -> Result<DynMetadata<dyn StateAction>, D::Error> {
            let value_ref: &dyn StateAction = unsafe {
                match self.class() {
                    ActionIdle => mem::transmute_copy::<usize, &StateActionIdle>(&0),
                    _ => panic!("Invalid TmplClass"),
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
    fn class(&self) -> TmplClass;
    fn restore(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()>;
    fn next(&mut self, ctx: &mut ContextUpdate<'_>, ctx_an: &ContextActionNext) -> XResult<Option<Rc<dyn InstAction>>>;
    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctx_au: &mut ContextActionUpdate<'_>) -> XResult<()>;
}

#[derive(Debug)]
pub struct LogicActionBase {
    pub id: NumID,
    pub tmpl_id: StrID,
    pub spawn_frame: u32,
    pub dead_frame: u32,
    pub derive_level: u16,
    pub antibreak_level: u16,
    pub blend_weight: f32,
    pub body_ratio: f32,
}

interface!(LogicAction, LogicActionBase);

impl LogicActionBase {
    pub fn new(id: NumID, tmpl_id: StrID, spawn_frame: u32) -> LogicActionBase {
        LogicActionBase {
            id,
            tmpl_id,
            spawn_frame,
            dead_frame: u32::MAX,
            derive_level: 0,
            antibreak_level: 0,
            blend_weight: 0.0,
            body_ratio: 0.0,
        }
    }

    pub fn reuse(&mut self, id: NumID, tmpl_id: StrID, spawn_frame: u32) -> XResult<()> {
        self.id = id;
        self.tmpl_id = tmpl_id;
        self.spawn_frame = spawn_frame;
        self.dead_frame = u32::MAX;
        self.derive_level = 0;
        self.antibreak_level = 0;
        self.blend_weight = 0.0;
        self.body_ratio = 0.0;
        Ok(())
    }

    pub fn save(&self) -> StateActionBase {
        StateActionBase {
            id: self.id,
            tmpl_id: self.tmpl_id.clone(),
            spawn_frame: self.spawn_frame,
            dead_frame: self.dead_frame,
            derive_level: self.derive_level,
            antibreak_level: self.antibreak_level,
            blend_weight: self.blend_weight,
            body_ratio: self.body_ratio,
            animations: Default::default(),
        }
    }

    pub fn restore(&mut self, state: &StateActionBase) {
        self.spawn_frame = state.spawn_frame;
        self.dead_frame = state.dead_frame;
        self.derive_level = state.derive_level;
        self.antibreak_level = state.antibreak_level;
        self.blend_weight = state.blend_weight;
        self.body_ratio = state.body_ratio;
    }
}

//
// ContextActionNext
//

pub struct ContextActionNext {
    pub player_id: NumID,
    pub inst_player: Rc<InstPlayer>,
}

impl ContextActionNext {
    pub fn new(player_id: NumID, inst_player: Rc<InstPlayer>) -> ContextActionNext {
        ContextActionNext { player_id, inst_player }
    }
}

//
// ContextActionUpdate
//

pub struct ContextActionUpdate<'t> {
    pub player_id: NumID,
    pub inst_player: Rc<InstPlayer>,
    pub next_action: Option<&'t (dyn LogicAction + 'static)>,
    pub prev_action: Option<&'t (dyn LogicAction + 'static)>,
    pub is_idle: bool,
    pub unused_weight: f32,
    pub states: Vec<Box<dyn StateAction>>,
}

impl<'t> ContextActionUpdate<'t> {
    pub fn new(player_id: NumID, inst_player: Rc<InstPlayer>, states_cap: usize) -> ContextActionUpdate<'t> {
        ContextActionUpdate {
            player_id,
            inst_player,
            next_action: None,
            prev_action: None,
            is_idle: true,
            unused_weight: 1.0,
            states: Vec::with_capacity(states_cap),
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
    pub fn state(&mut self, state: Box<dyn StateAction>) {
        self.states.push(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::action::idle::{ActionIdleMode, StateActionIdle};
    use crate::utils::{s, CastPtr};
    use anyhow::Result;
    use rkyv::ser::serializers::AllocSerializer;
    use rkyv::ser::Serializer;
    use rkyv::{Deserialize, Infallible};

    fn test_rkyv(state: Box<dyn StateAction>, class: TmplClass) -> Result<Box<dyn StateAction>> {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(&state)?;
        let buffer = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<Box<dyn StateAction>>(&buffer) };
        assert_eq!(archived.class(), class);

        let mut deserializer = Infallible;
        let result: Box<dyn StateAction> = archived.deserialize(&mut deserializer)?;
        assert_eq!(result.class(), class);

        Ok(result)
    }

    #[test]
    fn test_rkyv_state_tmpl_idle() {
        let mut raw_state = Box::new(StateActionIdle {
            _base: StateActionBase::default(),
            event_idx: 0,
            mode: ActionIdleMode::IdleToReady,
            is_dying: false,
            enter_progress: 57,
            idle_progress: 10,
            ready_progress: 20,
            idle_timer: 12,
            switch_progress: 5,
        });
        raw_state.id = 123;
        raw_state.tmpl_id = s!("idle");
        raw_state.spawn_frame = 99;
        raw_state.dead_frame = 1100;
        raw_state.derive_level = 1;
        raw_state.antibreak_level = 2;
        raw_state.blend_weight = 0.5;
        raw_state.body_ratio = 0.8;
        raw_state.animations[0] = StateActionAnimation {
            animation_id: 1,
            file: s!("idle.ozz"),
            ratio: 0.5,
            weight: 0.5,
        };

        let state = test_rkyv(raw_state, TmplClass::ActionIdle).unwrap();
        let state = state.cast_as::<StateActionIdle>().unwrap();

        assert_eq!(state.id, 123);
        assert_eq!(state.tmpl_id, s!("idle"));
        assert_eq!(state.spawn_frame, 99);
        assert_eq!(state.dead_frame, 1100);
        assert_eq!(state.derive_level, 1);
        assert_eq!(state.antibreak_level, 2);
        assert_eq!(state.blend_weight, 0.5);
        assert_eq!(state.body_ratio, 0.8);
        assert_eq!(state.animations[0].animation_id, 1);
        assert_eq!(state.animations[0].file, s!("idle.ozz"));
        assert_eq!(state.animations[0].ratio, 0.5);
        assert_eq!(state.animations[0].weight, 0.5);
        assert_eq!(state.animations[1], StateActionAnimation::default());
        assert_eq!(state.animations[2], StateActionAnimation::default());
        assert_eq!(state.animations[3], StateActionAnimation::default());
        assert_eq!(state.event_idx, 0);
        assert_eq!(state.mode, ActionIdleMode::IdleToReady);
        assert!(!state.is_dying);
        assert_eq!(state.enter_progress, 57);
        assert_eq!(state.idle_progress, 10);
        assert_eq!(state.ready_progress, 20);
        assert_eq!(state.idle_timer, 12);
        assert_eq!(state.switch_progress, 5);
    }
}
