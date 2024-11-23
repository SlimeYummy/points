use cirtical_point_csgen::{CsEnum, CsOut};
use enum_iterator::{cardinality, Sequence};
use std::fmt::Debug;
use std::mem;

use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{interface, Castable, NumID, XError, XResult};

//
// LogicType & LogicAny
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
pub enum LogicType {
    Game,
    Stage,
    Player,
    Npc,
}

impl From<LogicType> for u16 {
    #[inline]
    fn from(val: LogicType) -> Self {
        unsafe { mem::transmute::<LogicType, u16>(val) }
    }
}

impl TryFrom<u16> for LogicType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> Result<Self, XError> {
        if value as usize >= cardinality::<LogicType>() {
            return Err(XError::overflow("LogicType::try_from()"));
        }
        Ok(unsafe { mem::transmute::<u16, LogicType>(value) })
    }
}

pub trait LogicAny: Debug {
    fn typ(&self) -> LogicType;
    fn id(&self) -> NumID;
    fn spawn_frame(&self) -> u32;
    fn death_frame(&self) -> u32;

    fn update(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()>;
    fn restore(&mut self, ctx: &ContextRestore) -> XResult<()>;

    #[inline]
    fn is_alive(&self) -> bool {
        self.death_frame() == u32::MAX
    }
}

//
// StateAny
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
pub enum StateType {
    GameInit,
    GameUpdate,
    StageInit,
    StageUpdate,
    PlayerInit,
    PlayerUpdate,
    NpcInit,
    NpcUpdate,
}

impl StateType {
    #[inline]
    pub fn logic_typ(&self) -> LogicType {
        match self {
            StateType::GameInit | StateType::GameUpdate => LogicType::Game,
            StateType::StageInit | StateType::StageUpdate => LogicType::Stage,
            StateType::PlayerInit | StateType::PlayerUpdate => LogicType::Player,
            StateType::NpcInit | StateType::NpcUpdate => LogicType::Npc,
        }
    }
}

impl From<StateType> for u16 {
    #[inline]
    fn from(val: StateType) -> Self {
        unsafe { mem::transmute::<StateType, u16>(val) }
    }
}

impl TryFrom<u16> for StateType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> Result<Self, XError> {
        if value as usize >= cardinality::<StateType>() {
            return Err(XError::overflow("StateType::try_from()"));
        }
        Ok(unsafe { mem::transmute::<u16, StateType>(value) })
    }
}

pub unsafe trait StateAny
where
    Self: Debug + Send + Sync,
{
    fn typ(&self) -> StateType;
    fn id(&self) -> NumID;

    fn logic_typ(&self) -> LogicType {
        self.typ().logic_typ()
    }
}

#[repr(C)]
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateAnyBase {
    pub id: NumID,
    pub typ: StateType,
    pub logic_typ: LogicType,
}

#[cfg(debug_assertions)]
impl Drop for StateAnyBase {
    fn drop(&mut self) {
        println!("StateAnyBase drop() {} {:?}", self.id, self.typ);
    }
}

interface!(StateAny, StateAnyBase);

impl StateAnyBase {
    pub fn new(id: NumID, typ: StateType, logic_typ: LogicType) -> Self {
        Self { id, typ, logic_typ }
    }
}

pub trait ArchivedStateAny: Debug {
    fn typ(&self) -> StateType;

    fn logic_typ(&self) -> LogicType {
        self.typ().logic_typ()
    }
}

impl Castable for dyn ArchivedStateAny {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateAnyMetadata {
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

    use crate::logic::character::{
        ArchivedStateNpcInit, ArchivedStateNpcUpdate, ArchivedStatePlayerInit, ArchivedStatePlayerUpdate,
        StateCharaPhysics, StateNpcInit, StateNpcUpdate, StatePlayerInit, StatePlayerUpdate,
    };
    use crate::logic::game::{ArchivedStateGameInit, ArchivedStateGameUpdate, StateGameInit, StateGameUpdate};
    use crate::logic::stage::{ArchivedStateStageInit, ArchivedStateStageUpdate, StateStageInit, StateStageUpdate};
    use crate::utils::CastRef;
    use StateType::*;

    impl Pointee for dyn StateAny {
        type Metadata = DynMetadata<dyn StateAny>;
    }

    impl Pointee for dyn ArchivedStateAny {
        type Metadata = DynMetadata<dyn ArchivedStateAny>;
    }

    impl ArchivePointee for dyn ArchivedStateAny {
        type ArchivedMetadata = StateAnyMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let typ = StateType::try_from(archived.typ).expect("Invalid StateType");
            let archived_ref: &dyn ArchivedStateAny = unsafe {
                match typ {
                    GameInit => mem::transmute_copy::<usize, &ArchivedStateGameInit>(&0),
                    GameUpdate => mem::transmute_copy::<usize, &ArchivedStateGameUpdate>(&0),
                    StageInit => mem::transmute_copy::<usize, &ArchivedStateStageInit>(&0),
                    StageUpdate => mem::transmute_copy::<usize, &ArchivedStateStageUpdate>(&0),
                    PlayerInit => mem::transmute_copy::<usize, &ArchivedStatePlayerInit>(&0),
                    PlayerUpdate => mem::transmute_copy::<usize, &ArchivedStatePlayerUpdate>(&0),
                    NpcInit => mem::transmute_copy::<usize, &ArchivedStateNpcInit>(&0),
                    NpcUpdate => mem::transmute_copy::<usize, &ArchivedStateNpcUpdate>(&0),
                }
            };
            ptr::metadata(archived_ref)
        }
    }

    impl ArchiveUnsized for dyn StateAny {
        type Archived = dyn ArchivedStateAny;
        type MetadataResolver = ();

        unsafe fn resolve_metadata(
            &self,
            _pos: usize,
            _resolver: Self::MetadataResolver,
            out: *mut ArchivedMetadata<Self>,
        ) {
            let typ = to_archived!(self.typ().into());
            out.write(StateAnyMetadata { typ });
        }
    }

    impl<S> SerializeUnsized<S> for dyn StateAny
    where
        S: Serializer + ScratchSpace + ?Sized,
    {
        fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
            #[inline(always)]
            fn serialize<T, S>(state_any: &(dyn StateAny + 'static), serializer: &mut S) -> Result<usize, S::Error>
            where
                T: StateAny + Serialize<S> + 'static,
                S: Serializer + ScratchSpace + ?Sized,
            {
                let state_ref = unsafe { state_any.cast_ref_unchecked::<T>() };
                let resolver = state_ref.serialize(serializer)?;
                serializer.align_for::<T>()?;
                Ok(unsafe { serializer.resolve_aligned(state_ref, resolver)? })
            }

            match self.typ() {
                GameInit => serialize::<StateGameInit, _>(self, serializer),
                GameUpdate => serialize::<StateGameUpdate, _>(self, serializer),
                StageInit => serialize::<StateStageInit, _>(self, serializer),
                StageUpdate => serialize::<StateStageUpdate, _>(self, serializer),
                PlayerInit => serialize::<StatePlayerInit, _>(self, serializer),
                PlayerUpdate => serialize::<StatePlayerUpdate, _>(self, serializer),
                NpcInit => serialize::<StateNpcInit, _>(self, serializer),
                NpcUpdate => serialize::<StateNpcUpdate, _>(self, serializer),
            }
        }

        fn serialize_metadata(&self, _serializer: &mut S) -> Result<Self::MetadataResolver, S::Error> {
            Ok(())
        }
    }

    impl<D> DeserializeUnsized<dyn StateAny, D> for dyn ArchivedStateAny
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
                archived_any: &(dyn ArchivedStateAny + 'static),
                deserializer: &mut D,
                mut alloc: impl FnMut(Layout) -> *mut u8,
            ) -> Result<*mut (), D::Error>
            where
                T: StateAny + Archive + 'static,
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
                GameInit => deserialize::<StateGameInit, _>(self, deserializer, alloc),
                GameUpdate => deserialize::<StateGameUpdate, _>(self, deserializer, alloc),
                StageInit => deserialize::<StateStageInit, _>(self, deserializer, alloc),
                StageUpdate => deserialize::<StateStageUpdate, _>(self, deserializer, alloc),
                PlayerInit => deserialize::<StatePlayerInit, _>(self, deserializer, alloc),
                PlayerUpdate => deserialize::<StatePlayerUpdate, _>(self, deserializer, alloc),
                NpcInit => deserialize::<StateNpcInit, _>(self, deserializer, alloc),
                NpcUpdate => deserialize::<StateNpcUpdate, _>(self, deserializer, alloc),
            }
        }

        fn deserialize_metadata(&self, _deserializer: &mut D) -> Result<DynMetadata<dyn StateAny>, D::Error> {
            let value_ref: &dyn StateAny = unsafe {
                match self.typ() {
                    GameInit => mem::transmute_copy::<usize, &StateGameInit>(&0),
                    GameUpdate => mem::transmute_copy::<usize, &StateGameUpdate>(&0),
                    StageInit => mem::transmute_copy::<usize, &StateStageInit>(&0),
                    StageUpdate => mem::transmute_copy::<usize, &StateStageUpdate>(&0),
                    PlayerInit => mem::transmute_copy::<usize, &StatePlayerInit>(&0),
                    PlayerUpdate => mem::transmute_copy::<usize, &StatePlayerUpdate>(&0),
                    NpcInit => mem::transmute_copy::<usize, &StateNpcInit>(&0),
                    NpcUpdate => mem::transmute_copy::<usize, &StateNpcUpdate>(&0),
                }
            };
            Ok(ptr::metadata(value_ref))
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::character::{
        StateCharaPhysics, StateNpcInit, StateNpcUpdate, StatePlayerInit, StatePlayerUpdate,
    };
    use crate::logic::game::{StateGameInit, StateGameUpdate};
    use crate::logic::stage::{StateStageInit, StateStageUpdate};
    use crate::utils::{s, CastPtr};
    use anyhow::Result;
    use glam::{Quat, Vec3};
    use rkyv::ser::serializers::AllocSerializer;
    use rkyv::ser::Serializer;
    use rkyv::{Deserialize, Infallible};

    fn test_rkyv(state: Box<dyn StateAny>, typ: StateType, logic_typ: LogicType) -> Result<Box<dyn StateAny>> {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(&state)?;
        let buffer = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<Box<dyn StateAny>>(&buffer) };
        assert_eq!(archived.typ(), typ);
        assert_eq!(archived.logic_typ(), logic_typ);

        let mut deserializer = Infallible;
        let result: Box<dyn StateAny> = archived.deserialize(&mut deserializer)?;
        assert_eq!(result.typ(), typ);
        assert_eq!(result.logic_typ(), logic_typ);

        Ok(result)
    }

    #[test]
    fn test_rkyv_state_game() {
        let state_game_new = test_rkyv(
            Box::new(StateGameInit {
                _base: StateAnyBase::new(123, StateType::GameInit, LogicType::Game),
            }),
            StateType::GameInit,
            LogicType::Game,
        )
        .unwrap();
        assert_eq!(state_game_new.id(), 123);
        let state_game_new = state_game_new.cast_as::<StateGameInit>().unwrap();
        assert_eq!(state_game_new.id, 123);
        assert_eq!(state_game_new.typ, StateType::GameInit);
        assert_eq!(state_game_new.logic_typ, LogicType::Game);

        let state_game_update = test_rkyv(
            Box::new(StateGameUpdate {
                _base: StateAnyBase::new(456, StateType::GameUpdate, LogicType::Game),
                frame: 90,
                id_gen_counter: 1,
            }),
            StateType::GameUpdate,
            LogicType::Game,
        )
        .unwrap();
        assert_eq!(state_game_update.id(), 456);
        let state_game_update = state_game_update.cast_as::<StateGameUpdate>().unwrap();
        assert_eq!(state_game_update.id, 456);
        assert_eq!(state_game_update.frame, 90);
        assert_eq!(state_game_update.id_gen_counter, 1);
        assert_eq!(state_game_update.typ, StateType::GameUpdate);
        assert_eq!(state_game_update.logic_typ, LogicType::Game);
    }

    #[test]
    fn test_rkyv_state_stage() {
        let state_stage_new = test_rkyv(
            Box::new(StateStageInit {
                _base: StateAnyBase::new(4321, StateType::StageInit, LogicType::Stage),
                view_stage_file: "stage_file.tscn".to_string(),
            }),
            StateType::StageInit,
            LogicType::Stage,
        )
        .unwrap();
        assert_eq!(state_stage_new.id(), 4321);
        let state_stage_new = state_stage_new.cast_as::<StateStageInit>().unwrap();
        assert_eq!(state_stage_new.id, 4321);
        assert_eq!(state_stage_new.typ, StateType::StageInit);
        assert_eq!(state_stage_new.logic_typ, LogicType::Stage);
        assert_eq!(state_stage_new.view_stage_file, "stage_file.tscn");

        let state_stage_update = test_rkyv(
            Box::new(StateStageUpdate {
                _base: StateAnyBase::new(8765, StateType::StageUpdate, LogicType::Stage),
            }),
            StateType::StageUpdate,
            LogicType::Stage,
        )
        .unwrap();
        assert_eq!(state_stage_update.id(), 8765);
        let state_stage_update = state_stage_update.cast_as::<StateStageUpdate>().unwrap();
        assert_eq!(state_stage_update.id, 8765);
        assert_eq!(state_stage_update.typ, StateType::StageUpdate);
        assert_eq!(state_stage_update.logic_typ, LogicType::Stage);
    }

    #[test]
    fn test_rkyv_state_player() {
        let state_player_new = test_rkyv(
            Box::new(StatePlayerInit {
                _base: StateAnyBase::new(1110, StateType::PlayerInit, LogicType::Player),
                skeleton_file: s!("skeleton_file.ozz"),
                animation_files: vec![s!("animation_file_1.ozz"), s!("animation_file_2.ozz")],
                view_model: "model.vrm".to_string(),
            }),
            StateType::PlayerInit,
            LogicType::Player,
        )
        .unwrap();
        assert_eq!(state_player_new.id(), 1110);
        let state_player_new = state_player_new.cast_as::<StatePlayerInit>().unwrap();
        assert_eq!(state_player_new.id, 1110);
        assert_eq!(state_player_new.typ, StateType::PlayerInit);
        assert_eq!(state_player_new.logic_typ, LogicType::Player);
        assert_eq!(state_player_new.skeleton_file, s!("skeleton_file.ozz"));
        assert_eq!(
            state_player_new.animation_files,
            vec![s!("animation_file_1.ozz"), s!("animation_file_2.ozz")]
        );
        assert_eq!(state_player_new.view_model, "model.vrm");

        let state_player_update = test_rkyv(
            Box::new(StatePlayerUpdate {
                _base: StateAnyBase::new(2220, StateType::PlayerUpdate, LogicType::Player),
                physics: StateCharaPhysics {
                    position: Vec3::new(1.0, 2.0, 3.0),
                    rotation: Quat::IDENTITY,
                },
                actions: Vec::new(),
            }),
            StateType::PlayerUpdate,
            LogicType::Player,
        )
        .unwrap();
        assert_eq!(state_player_update.id(), 2220);
        let state_player_update = state_player_update.cast_as::<StatePlayerUpdate>().unwrap();
        assert_eq!(state_player_update.id, 2220);
        assert_eq!(state_player_update.typ, StateType::PlayerUpdate);
        assert_eq!(state_player_update.logic_typ, LogicType::Player);
        assert_eq!(state_player_update.physics.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(state_player_update.physics.rotation, Quat::IDENTITY);
        assert_eq!(state_player_update.actions.len(), 0);
    }

    #[test]
    fn test_rkyv_state_enemy() {
        let state_enemy_new = test_rkyv(
            Box::new(StateNpcInit {
                _base: StateAnyBase::new(1111, StateType::NpcInit, LogicType::Npc),
            }),
            StateType::NpcInit,
            LogicType::Npc,
        )
        .unwrap();
        assert_eq!(state_enemy_new.id(), 1111);
        let state_enemy_new = state_enemy_new.cast_as::<StateNpcInit>().unwrap();
        assert_eq!(state_enemy_new.id, 1111);
        assert_eq!(state_enemy_new.typ, StateType::NpcInit);
        assert_eq!(state_enemy_new.logic_typ, LogicType::Npc);

        let state_enemy_update = test_rkyv(
            Box::new(StateNpcUpdate {
                _base: StateAnyBase::new(2222, StateType::NpcUpdate, LogicType::Npc),
            }),
            StateType::NpcUpdate,
            LogicType::Npc,
        )
        .unwrap();
        assert_eq!(state_enemy_update.id(), 2222);
        let state_enemy_update = state_enemy_update.cast_as::<StateNpcUpdate>().unwrap();
        assert_eq!(state_enemy_update.id, 2222);
        assert_eq!(state_enemy_update.typ, StateType::NpcUpdate);
        assert_eq!(state_enemy_update.logic_typ, LogicType::Npc);
    }
}
