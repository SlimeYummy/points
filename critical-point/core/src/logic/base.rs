use cirtical_point_csgen::CsGen;
use enum_iterator::{cardinality, Sequence};
use std::fmt::Debug;
use std::mem;

use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{interface, Castable, NumID, XError, XResult};

//
// LogicClass & LogicAny
//

#[repr(u8)]
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
    CsGen
)]
#[archive_attr(derive(Debug))]
pub enum LogicClass {
    Game,
    Stage,
    Player,
    Enemy,
}

impl From<LogicClass> for u8 {
    #[inline]
    fn from(val: LogicClass) -> Self {
        unsafe { mem::transmute::<LogicClass, u8>(val) }
    }
}

impl TryFrom<u8> for LogicClass {
    type Error = XError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, XError> {
        if value as usize >= cardinality::<LogicClass>() {
            return Err(XError::overflow("LogicClass::try_from()"));
        }
        Ok(unsafe { mem::transmute::<u8, LogicClass>(value) })
    }
}

pub trait LogicAny: Debug {
    fn class(&self) -> LogicClass;
    fn id(&self) -> NumID;
    fn spawn_frame(&self) -> u32;
    fn dead_frame(&self) -> u32;

    fn update(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()>;
    fn restore(&mut self, ctx: &ContextRestore) -> XResult<()>;

    #[inline]
    fn is_alive(&self) -> bool {
        self.dead_frame() == u32::MAX
    }
}

//
// StateAny
//

#[repr(u8)]
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
    CsGen,
)]
#[archive_attr(derive(Debug))]
pub enum StateClass {
    GameInit,
    GameUpdate,
    StageInit,
    StageUpdate,
    PlayerInit,
    PlayerUpdate,
    EnemyInit,
    EnemyUpdate,
}

impl StateClass {
    #[inline]
    pub fn logic_class(&self) -> LogicClass {
        match self {
            StateClass::GameInit | StateClass::GameUpdate => LogicClass::Game,
            StateClass::StageInit | StateClass::StageUpdate => LogicClass::Stage,
            StateClass::PlayerInit | StateClass::PlayerUpdate => LogicClass::Player,
            StateClass::EnemyInit | StateClass::EnemyUpdate => LogicClass::Enemy,
        }
    }
}

impl From<StateClass> for u8 {
    #[inline]
    fn from(val: StateClass) -> Self {
        unsafe { mem::transmute::<StateClass, u8>(val) }
    }
}

impl TryFrom<u8> for StateClass {
    type Error = XError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, XError> {
        if value as usize >= cardinality::<StateClass>() {
            return Err(XError::overflow("StateClass::try_from()"));
        }
        Ok(unsafe { mem::transmute::<u8, StateClass>(value) })
    }
}

pub unsafe trait StateAny
where
    Self: Debug + Send + Sync,
{
    fn class(&self) -> StateClass;
    fn id(&self) -> NumID;

    fn logic_class(&self) -> LogicClass {
        self.class().logic_class()
    }
}

#[repr(C)]
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen)]
#[archive_attr(derive(Debug))]
#[cs_attr(Rs, Ref)]
pub struct StateAnyBase {
    pub id: NumID,
    pub class: StateClass,
    pub logic_class: LogicClass,
}

interface!(StateAny, StateAnyBase);

impl StateAnyBase {
    pub fn new(id: NumID, class: StateClass, logic_class: LogicClass) -> Self {
        Self {
            id,
            class,
            logic_class,
        }
    }
}

pub trait ArchivedStateAny: Debug {
    fn class(&self) -> StateClass;

    fn logic_class(&self) -> LogicClass {
        self.class().logic_class()
    }
}

impl Castable for dyn ArchivedStateAny {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateAnyMetadata {
    pub class: rkyv::Archived<u8>,
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

    use crate::logic::enemy::{ArchivedStateEnemyInit, ArchivedStateEnemyUpdate, StateEnemyInit, StateEnemyUpdate};
    use crate::logic::game::{ArchivedStateGameInit, ArchivedStateGameUpdate, StateGameInit, StateGameUpdate};
    use crate::logic::player::{
        ArchivedStatePlayerInit, ArchivedStatePlayerUpdate, StatePlayerInit, StatePlayerUpdate,
    };
    use crate::logic::stage::{ArchivedStateStageInit, ArchivedStateStageUpdate, StateStageInit, StateStageUpdate};
    use crate::utils::CastRef;
    use StateClass::*;

    impl Pointee for dyn StateAny {
        type Metadata = DynMetadata<dyn StateAny>;
    }

    impl Pointee for dyn ArchivedStateAny {
        type Metadata = DynMetadata<dyn ArchivedStateAny>;
    }

    impl ArchivePointee for dyn ArchivedStateAny {
        type ArchivedMetadata = StateAnyMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let class = StateClass::try_from(archived.class).expect("Invalid StateClass");
            let archived_ref: &dyn ArchivedStateAny = unsafe {
                match class {
                    GameInit => mem::transmute_copy::<usize, &ArchivedStateGameInit>(&0),
                    GameUpdate => mem::transmute_copy::<usize, &ArchivedStateGameUpdate>(&0),
                    StageInit => mem::transmute_copy::<usize, &ArchivedStateStageInit>(&0),
                    StageUpdate => mem::transmute_copy::<usize, &ArchivedStateStageUpdate>(&0),
                    PlayerInit => mem::transmute_copy::<usize, &ArchivedStatePlayerInit>(&0),
                    PlayerUpdate => mem::transmute_copy::<usize, &ArchivedStatePlayerUpdate>(&0),
                    EnemyInit => mem::transmute_copy::<usize, &ArchivedStateEnemyInit>(&0),
                    EnemyUpdate => mem::transmute_copy::<usize, &ArchivedStateEnemyUpdate>(&0),
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
            let class = to_archived!(self.class().into());
            out.write(StateAnyMetadata { class });
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

            match self.class() {
                GameInit => serialize::<StateGameInit, _>(self, serializer),
                GameUpdate => serialize::<StateGameUpdate, _>(self, serializer),
                StageInit => serialize::<StateStageInit, _>(self, serializer),
                StageUpdate => serialize::<StateStageUpdate, _>(self, serializer),
                PlayerInit => serialize::<StatePlayerInit, _>(self, serializer),
                PlayerUpdate => serialize::<StatePlayerUpdate, _>(self, serializer),
                EnemyInit => serialize::<StateEnemyInit, _>(self, serializer),
                EnemyUpdate => serialize::<StateEnemyUpdate, _>(self, serializer),
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

            match self.class() {
                GameInit => deserialize::<StateGameInit, _>(self, deserializer, alloc),
                GameUpdate => deserialize::<StateGameUpdate, _>(self, deserializer, alloc),
                StageInit => deserialize::<StateStageInit, _>(self, deserializer, alloc),
                StageUpdate => deserialize::<StateStageUpdate, _>(self, deserializer, alloc),
                PlayerInit => deserialize::<StatePlayerInit, _>(self, deserializer, alloc),
                PlayerUpdate => deserialize::<StatePlayerUpdate, _>(self, deserializer, alloc),
                EnemyInit => deserialize::<StateEnemyInit, _>(self, deserializer, alloc),
                EnemyUpdate => deserialize::<StateEnemyUpdate, _>(self, deserializer, alloc),
            }
        }

        fn deserialize_metadata(&self, _deserializer: &mut D) -> Result<DynMetadata<dyn StateAny>, D::Error> {
            let value_ref: &dyn StateAny = unsafe {
                match self.class() {
                    GameInit => mem::transmute_copy::<usize, &StateGameInit>(&0),
                    GameUpdate => mem::transmute_copy::<usize, &StateGameUpdate>(&0),
                    StageInit => mem::transmute_copy::<usize, &StateStageInit>(&0),
                    StageUpdate => mem::transmute_copy::<usize, &StateStageUpdate>(&0),
                    PlayerInit => mem::transmute_copy::<usize, &StatePlayerInit>(&0),
                    PlayerUpdate => mem::transmute_copy::<usize, &StatePlayerUpdate>(&0),
                    EnemyInit => mem::transmute_copy::<usize, &StateEnemyInit>(&0),
                    EnemyUpdate => mem::transmute_copy::<usize, &StateEnemyUpdate>(&0),
                }
            };
            Ok(ptr::metadata(value_ref))
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::enemy::{StateEnemyInit, StateEnemyUpdate};
    use crate::logic::game::{StateGameInit, StateGameUpdate};
    use crate::logic::player::{StatePlayerInit, StatePlayerUpdate};
    use crate::logic::stage::{StateStageInit, StateStageUpdate};
    use crate::utils::{s, CastPtr};
    use anyhow::Result;
    use glam::Vec3;
    use rkyv::ser::serializers::AllocSerializer;
    use rkyv::ser::Serializer;
    use rkyv::{Deserialize, Infallible};

    fn test_rkyv(state: Box<dyn StateAny>, class: StateClass, logic_class: LogicClass) -> Result<Box<dyn StateAny>> {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(&state)?;
        let buffer = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<Box<dyn StateAny>>(&buffer) };
        assert_eq!(archived.class(), class);
        assert_eq!(archived.logic_class(), logic_class);

        let mut deserializer = Infallible;
        let result: Box<dyn StateAny> = archived.deserialize(&mut deserializer)?;
        assert_eq!(result.class(), class);
        assert_eq!(result.logic_class(), logic_class);

        Ok(result)
    }

    #[test]
    fn test_rkyv_state_game() {
        let state_game_new = test_rkyv(
            Box::new(StateGameInit {
                _base: StateAnyBase::new(123, StateClass::GameInit, LogicClass::Game),
            }),
            StateClass::GameInit,
            LogicClass::Game,
        )
        .unwrap();
        assert_eq!(state_game_new.id(), 123);
        let state_game_new = state_game_new.cast_as::<StateGameInit>().unwrap();
        assert_eq!(state_game_new.id, 123);
        assert_eq!(state_game_new.class, StateClass::GameInit);
        assert_eq!(state_game_new.logic_class, LogicClass::Game);

        let state_game_update = test_rkyv(
            Box::new(StateGameUpdate {
                _base: StateAnyBase::new(456, StateClass::GameUpdate, LogicClass::Game),
                frame: 90,
                id_gen_counter: 1,
            }),
            StateClass::GameUpdate,
            LogicClass::Game,
        )
        .unwrap();
        assert_eq!(state_game_update.id(), 456);
        let state_game_update = state_game_update.cast_as::<StateGameUpdate>().unwrap();
        assert_eq!(state_game_update.id, 456);
        assert_eq!(state_game_update.frame, 90);
        assert_eq!(state_game_update.id_gen_counter, 1);
        assert_eq!(state_game_update.class, StateClass::GameUpdate);
        assert_eq!(state_game_update.logic_class, LogicClass::Game);
    }

    #[test]
    fn test_rkyv_state_stage() {
        let state_stage_new = test_rkyv(
            Box::new(StateStageInit {
                _base: StateAnyBase::new(4321, StateClass::StageInit, LogicClass::Stage),
            }),
            StateClass::StageInit,
            LogicClass::Stage,
        )
        .unwrap();
        assert_eq!(state_stage_new.id(), 4321);
        let state_stage_new = state_stage_new.cast_as::<StateStageInit>().unwrap();
        assert_eq!(state_stage_new.id, 4321);
        assert_eq!(state_stage_new.class, StateClass::StageInit);
        assert_eq!(state_stage_new.logic_class, LogicClass::Stage);

        let state_stage_update = test_rkyv(
            Box::new(StateStageUpdate {
                _base: StateAnyBase::new(8765, StateClass::StageUpdate, LogicClass::Stage),
            }),
            StateClass::StageUpdate,
            LogicClass::Stage,
        )
        .unwrap();
        assert_eq!(state_stage_update.id(), 8765);
        let state_stage_update = state_stage_update.cast_as::<StateStageUpdate>().unwrap();
        assert_eq!(state_stage_update.id, 8765);
        assert_eq!(state_stage_update.class, StateClass::StageUpdate);
        assert_eq!(state_stage_update.logic_class, LogicClass::Stage);
    }

    #[test]
    fn test_rkyv_state_player() {
        let state_player_new = test_rkyv(
            Box::new(StatePlayerInit {
                _base: StateAnyBase::new(1110, StateClass::PlayerInit, LogicClass::Player),
                skeleton_file: s!("skeleton_file.ozz"),
                animation_files: vec![s!("animation_file_1.ozz"), s!("animation_file_2.ozz")],
            }),
            StateClass::PlayerInit,
            LogicClass::Player,
        )
        .unwrap();
        assert_eq!(state_player_new.id(), 1110);
        let state_player_new = state_player_new.cast_as::<StatePlayerInit>().unwrap();
        assert_eq!(state_player_new.id, 1110);
        assert_eq!(state_player_new.class, StateClass::PlayerInit);
        assert_eq!(state_player_new.logic_class, LogicClass::Player);
        assert_eq!(state_player_new.skeleton_file, s!("skeleton_file.ozz"));
        assert_eq!(
            state_player_new.animation_files,
            vec![s!("animation_file_1.ozz"), s!("animation_file_2.ozz")]
        );

        let state_player_update = test_rkyv(
            Box::new(StatePlayerUpdate {
                _base: StateAnyBase::new(2220, StateClass::PlayerUpdate, LogicClass::Player),
                position: Vec3::new(1.0, 2.0, 3.0),
                direction: Vec3::new(-1.0, 1.0, 1.0),
                actions: Vec::new(),
            }),
            StateClass::PlayerUpdate,
            LogicClass::Player,
        )
        .unwrap();
        assert_eq!(state_player_update.id(), 2220);
        let state_player_update = state_player_update.cast_as::<StatePlayerUpdate>().unwrap();
        assert_eq!(state_player_update.id, 2220);
        assert_eq!(state_player_update.class, StateClass::PlayerUpdate);
        assert_eq!(state_player_update.logic_class, LogicClass::Player);
        assert_eq!(state_player_update.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(state_player_update.direction, Vec3::new(-1.0, 1.0, 1.0));
        assert_eq!(state_player_update.actions.len(), 0);
    }

    #[test]
    fn test_rkyv_state_enemy() {
        let state_enemy_new = test_rkyv(
            Box::new(StateEnemyInit {
                _base: StateAnyBase::new(1111, StateClass::EnemyInit, LogicClass::Enemy),
            }),
            StateClass::EnemyInit,
            LogicClass::Enemy,
        )
        .unwrap();
        assert_eq!(state_enemy_new.id(), 1111);
        let state_enemy_new = state_enemy_new.cast_as::<StateEnemyInit>().unwrap();
        assert_eq!(state_enemy_new.id, 1111);
        assert_eq!(state_enemy_new.class, StateClass::EnemyInit);
        assert_eq!(state_enemy_new.logic_class, LogicClass::Enemy);

        let state_enemy_update = test_rkyv(
            Box::new(StateEnemyUpdate {
                _base: StateAnyBase::new(2222, StateClass::EnemyUpdate, LogicClass::Enemy),
            }),
            StateClass::EnemyUpdate,
            LogicClass::Enemy,
        )
        .unwrap();
        assert_eq!(state_enemy_update.id(), 2222);
        let state_enemy_update = state_enemy_update.cast_as::<StateEnemyUpdate>().unwrap();
        assert_eq!(state_enemy_update.id, 2222);
        assert_eq!(state_enemy_update.class, StateClass::EnemyUpdate);
        assert_eq!(state_enemy_update.logic_class, LogicClass::Enemy);
    }
}
