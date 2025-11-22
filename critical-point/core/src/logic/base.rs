use critical_point_csgen::{CsEnum, CsOut};
use enum_iterator::{cardinality, Sequence};
use std::alloc::Layout;
use std::any::Any;
use std::fmt::Debug;
use std::mem;

use crate::utils::{interface, rkyv_self, xres, NumID, XError, XResult};

//
// LogicType & LogicAny
//

#[repr(u16)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Sequence, serde::Serialize, serde::Deserialize, CsEnum,
)]
pub enum LogicType {
    Game,
    Zone,
    Player,
    Npc,
}

rkyv_self!(LogicType);

impl From<LogicType> for u16 {
    #[inline]
    fn from(val: LogicType) -> Self {
        unsafe { mem::transmute::<LogicType, u16>(val) }
    }
}

impl TryFrom<u16> for LogicType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> XResult<Self> {
        if value as usize >= cardinality::<LogicType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, LogicType>(value) })
    }
}

pub trait LogicAny: Debug {
    fn typ(&self) -> LogicType;
    fn id(&self) -> NumID;
    fn spawn_frame(&self) -> u32;
    fn death_frame(&self) -> u32;

    // fn state(&mut self) -> XResult<Box<dyn StateAny>>;
    // fn restore(&mut self, ctx: &ContextRestore) -> XResult<()>;

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
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Sequence, serde::Serialize, serde::Deserialize, CsEnum,
)]
pub enum StateType {
    GameInit,
    GameUpdate,
    ZoneInit,
    ZoneUpdate,
    PlayerInit,
    PlayerUpdate,
    NpcInit,
    NpcUpdate,
}

rkyv_self!(StateType);

impl StateType {
    #[inline]
    pub fn logic_typ(&self) -> LogicType {
        match self {
            StateType::GameInit | StateType::GameUpdate => LogicType::Game,
            StateType::ZoneInit | StateType::ZoneUpdate => LogicType::Zone,
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
    fn try_from(value: u16) -> XResult<Self> {
        if value as usize >= cardinality::<StateType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, StateType>(value) })
    }
}

impl From<StateType> for rkyv::primitive::ArchivedU16 {
    #[inline]
    fn from(val: StateType) -> Self {
        unsafe { mem::transmute::<StateType, u16>(val) }.into()
    }
}

impl TryFrom<rkyv::primitive::ArchivedU16> for StateType {
    type Error = XError;

    #[inline]
    fn try_from(val: rkyv::primitive::ArchivedU16) -> XResult<Self> {
        if val.to_native() as usize >= cardinality::<StateType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, StateType>(val.to_native()) })
    }
}

#[typetag::serde(tag = "T")]
pub unsafe trait StateAny: Debug + Any + Send + Sync {
    fn id(&self) -> NumID;
    fn typ(&self) -> StateType;
    fn logic_typ(&self) -> LogicType;
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
pub struct StateBase {
    pub id: NumID,
    pub typ: StateType,
    pub logic_typ: LogicType,
}

#[cfg(feature = "debug-print")]
impl Drop for StateBase {
    fn drop(&mut self) {
        log::debug!("StateBase::drop() id={}  typ={:?}", self.id, self.typ);
    }
}

interface!(StateAny, StateBase);

impl StateBase {
    pub fn new(id: NumID, typ: StateType, logic_typ: LogicType) -> Self {
        Self { id, typ, logic_typ }
    }
}

pub trait ArchivedStateAny: Debug + Any {
    fn id(&self) -> NumID;
    fn typ(&self) -> StateType;
    fn logic_typ(&self) -> LogicType;
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, rkyv::Portable)]
pub struct StateAnyMetadata(rkyv::primitive::ArchivedU16);

impl Default for StateAnyMetadata {
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

    use crate::logic::character::{
        ArchivedStateNpcInit, ArchivedStateNpcUpdate, ArchivedStatePlayerInit, ArchivedStatePlayerUpdate, StateNpcInit,
        StateNpcUpdate, StatePlayerInit, StatePlayerUpdate,
    };
    use crate::logic::game::{ArchivedStateGameInit, ArchivedStateGameUpdate, StateGameInit, StateGameUpdate};
    use crate::logic::zone::{ArchivedStateZoneInit, ArchivedStateZoneUpdate, StateZoneInit, StateZoneUpdate};
    use crate::utils::Castable;
    use StateType::*;

    impl PartialEq for dyn StateAny {
        fn eq(&self, other: &Self) -> bool {
            match (self.typ(), other.typ()) {
                (GameInit, GameInit) => unsafe {
                    self.cast_unchecked::<StateGameInit>() == other.cast_unchecked::<StateGameInit>()
                },
                (GameUpdate, GameUpdate) => unsafe {
                    self.cast_unchecked::<StateGameUpdate>() == other.cast_unchecked::<StateGameUpdate>()
                },
                (ZoneInit, ZoneInit) => unsafe {
                    self.cast_unchecked::<StateZoneInit>() == other.cast_unchecked::<StateZoneInit>()
                },
                (ZoneUpdate, ZoneUpdate) => unsafe {
                    self.cast_unchecked::<StateZoneUpdate>() == other.cast_unchecked::<StateZoneUpdate>()
                },
                (PlayerInit, PlayerInit) => unsafe {
                    self.cast_unchecked::<StatePlayerInit>() == other.cast_unchecked::<StatePlayerInit>()
                },
                (PlayerUpdate, PlayerUpdate) => unsafe {
                    self.cast_unchecked::<StatePlayerUpdate>() == other.cast_unchecked::<StatePlayerUpdate>()
                },
                (NpcInit, NpcInit) => unsafe {
                    self.cast_unchecked::<StateNpcInit>() == other.cast_unchecked::<StateNpcInit>()
                },
                (NpcUpdate, NpcUpdate) => unsafe {
                    self.cast_unchecked::<StateNpcUpdate>() == other.cast_unchecked::<StateNpcUpdate>()
                },
                _ => false,
            }
        }
    }

    impl LayoutRaw for dyn StateAny {
        fn layout_raw(metadata: DynMetadata<dyn StateAny>) -> Result<Layout, LayoutError> {
            unsafe {
                let null = ptr::from_raw_parts::<dyn StateAny>(ptr::null() as *const u8, metadata);
                Ok((*null).layout())
            }
        }
    }

    unsafe impl Pointee for dyn StateAny {
        type Metadata = DynMetadata<dyn StateAny>;
    }

    unsafe impl Pointee for dyn ArchivedStateAny {
        type Metadata = DynMetadata<dyn ArchivedStateAny>;
    }

    unsafe impl Portable for dyn ArchivedStateAny {}

    unsafe impl NoUndef for StateAnyMetadata {}

    impl ArchivePointee for dyn ArchivedStateAny {
        type ArchivedMetadata = StateAnyMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let typ = StateType::try_from(archived.0).expect("Invalid StateType");
            let archived_ref: &dyn ArchivedStateAny = unsafe {
                match typ {
                    GameInit => mem::transmute_copy::<usize, &ArchivedStateGameInit>(&0),
                    GameUpdate => mem::transmute_copy::<usize, &ArchivedStateGameUpdate>(&0),
                    ZoneInit => mem::transmute_copy::<usize, &ArchivedStateZoneInit>(&0),
                    ZoneUpdate => mem::transmute_copy::<usize, &ArchivedStateZoneUpdate>(&0),
                    PlayerInit => mem::transmute_copy::<usize, &ArchivedStatePlayerInit>(&0),
                    PlayerUpdate => mem::transmute_copy::<usize, &ArchivedStatePlayerUpdate>(&0),
                    NpcInit => mem::transmute_copy::<usize, &ArchivedStateNpcInit>(&0),
                    NpcUpdate => mem::transmute_copy::<usize, &ArchivedStateNpcUpdate>(&0),
                    _ => unreachable!("pointer_metadata() Invalid StateType"),
                }
            };
            ptr::metadata(archived_ref)
        }
    }

    impl ArchiveUnsized for dyn StateAny {
        type Archived = dyn ArchivedStateAny;

        fn archived_metadata(&self) -> ArchivedMetadata<Self> {
            StateAnyMetadata(self.typ().into())
        }
    }

    impl<S> SerializeUnsized<S> for dyn StateAny
    where
        S: Fallible + Allocator + Writer + ?Sized,
        S::Error: Source,
    {
        fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
            #[inline(always)]
            fn serialize<T, S>(state_any: &(dyn StateAny + 'static), serializer: &mut S) -> Result<usize, S::Error>
            where
                T: StateAny + Serialize<S> + 'static,
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
                GameInit => serialize::<StateGameInit, _>(self, serializer),
                GameUpdate => serialize::<StateGameUpdate, _>(self, serializer),
                ZoneInit => serialize::<StateZoneInit, _>(self, serializer),
                ZoneUpdate => serialize::<StateZoneUpdate, _>(self, serializer),
                PlayerInit => serialize::<StatePlayerInit, _>(self, serializer),
                PlayerUpdate => serialize::<StatePlayerUpdate, _>(self, serializer),
                NpcInit => serialize::<StateNpcInit, _>(self, serializer),
                NpcUpdate => serialize::<StateNpcUpdate, _>(self, serializer),
                _ => unreachable!("serialize_unsized() Invalid StateType"),
            }
        }
    }

    impl<D> DeserializeUnsized<dyn StateAny, D> for dyn ArchivedStateAny
    where
        D: Fallible + ?Sized,
        D::Error: Source,
    {
        unsafe fn deserialize_unsized(&self, deserializer: &mut D, out: *mut dyn StateAny) -> Result<(), D::Error> {
            #[inline(always)]
            fn deserialize<T, D>(
                archived_any: &(dyn ArchivedStateAny + 'static),
                deserializer: &mut D,
                out: *mut dyn StateAny,
            ) -> Result<(), D::Error>
            where
                T: StateAny + Archive + 'static,
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
                GameInit => deserialize::<StateGameInit, _>(self, deserializer, out),
                GameUpdate => deserialize::<StateGameUpdate, _>(self, deserializer, out),
                ZoneInit => deserialize::<StateZoneInit, _>(self, deserializer, out),
                ZoneUpdate => deserialize::<StateZoneUpdate, _>(self, deserializer, out),
                PlayerInit => deserialize::<StatePlayerInit, _>(self, deserializer, out),
                PlayerUpdate => deserialize::<StatePlayerUpdate, _>(self, deserializer, out),
                NpcInit => deserialize::<StateNpcInit, _>(self, deserializer, out),
                NpcUpdate => deserialize::<StateNpcUpdate, _>(self, deserializer, out),
                _ => unreachable!("deserialize_unsized() Invalid StateType"),
            }
        }

        fn deserialize_metadata(&self) -> DynMetadata<dyn StateAny> {
            let value_ref: &dyn StateAny = unsafe {
                match self.typ() {
                    GameInit => mem::transmute_copy::<usize, &StateGameInit>(&0),
                    GameUpdate => mem::transmute_copy::<usize, &StateGameUpdate>(&0),
                    ZoneInit => mem::transmute_copy::<usize, &StateZoneInit>(&0),
                    ZoneUpdate => mem::transmute_copy::<usize, &StateZoneUpdate>(&0),
                    PlayerInit => mem::transmute_copy::<usize, &StatePlayerInit>(&0),
                    PlayerUpdate => mem::transmute_copy::<usize, &StatePlayerUpdate>(&0),
                    NpcInit => mem::transmute_copy::<usize, &StateNpcInit>(&0),
                    NpcUpdate => mem::transmute_copy::<usize, &StateNpcUpdate>(&0),
                    _ => unreachable!("deserialize_metadata() Invalid StateType"),
                }
            };
            ptr::metadata(value_ref)
        }
    }
};

macro_rules! impl_state {
    ($typ:ty, $logic_enum:ident, $state_enum:ident, $serde_tag:expr) => {
        paste::paste! {
            #[typetag::serde(name = $serde_tag)]
            unsafe impl $crate::logic::StateAny for $typ {
                #[inline]
                fn id(&self) -> $crate::utils::NumID {
                    self._base.id
                }

                #[inline]
                fn typ(&self) -> $crate::logic::StateType {
                    debug_assert_eq!(self._base.typ, $crate::logic::StateType::$state_enum);
                    $crate::logic::StateType::$state_enum
                }

                #[inline]
                fn logic_typ(&self) -> $crate::logic::LogicType {
                    debug_assert_eq!(self._base.logic_typ, $crate::logic::LogicType::$logic_enum);
                    $crate::logic::LogicType::$logic_enum
                }

                #[inline]
                fn layout(&self) -> std::alloc::Layout {
                    std::alloc::Layout::new::<Self>()
                }
            }

            impl $crate::logic::ArchivedStateAny for [<Archived $typ>] {
                #[inline]
                fn id(&self) -> crate::utils::NumID {
                    self._base.id.to_native()
                }

                #[inline]
                fn typ(&self) -> $crate::logic::StateType {
                    debug_assert_eq!(self._base.typ, $crate::logic::StateType::$state_enum);
                    $crate::logic::StateType::$state_enum
                }

                #[inline]
                fn logic_typ(&self) -> $crate::logic::LogicType {
                    debug_assert_eq!(self._base.logic_typ, $crate::logic::LogicType::$logic_enum);
                    $crate::logic::LogicType::$logic_enum
                }
            }
        }
    };
}
pub(crate) use impl_state;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::character::{
        StateCharaPhysics, StateNpcInit, StateNpcUpdate, StatePlayerInit, StatePlayerUpdate,
    };
    use crate::logic::game::{StateGameInit, StateGameUpdate};
    use crate::logic::zone::{StateZoneInit, StateZoneUpdate};
    use crate::utils::{sb, Castable, CsVec3A, AnimationFileMeta};
    use anyhow::Result;
    use glam::Vec3A;
    use glam_ext::Vec2xz;

    fn test_rkyv(state: Box<dyn StateAny>, typ: StateType, logic_typ: LogicType) -> Result<Box<dyn StateAny>> {
        use rkyv::rancor::Failure;
        use rkyv::Archived;

        let buffer = rkyv::to_bytes::<Failure>(&state)?;
        let archived = unsafe { rkyv::access_unchecked::<Archived<Box<dyn StateAny>>>(&buffer) };
        assert_eq!(archived.typ(), typ);
        assert_eq!(archived.logic_typ(), logic_typ);
        let result: Box<dyn StateAny> = rkyv::deserialize::<_, Failure>(archived)?;
        assert_eq!(result.typ(), typ);
        assert_eq!(result.logic_typ(), logic_typ);
        Ok(result)
    }

    #[test]
    fn test_rkyv_state_game() {
        let state_game_new = test_rkyv(
            Box::new(StateGameInit {
                _base: StateBase::new(123, StateType::GameInit, LogicType::Game),
            }),
            StateType::GameInit,
            LogicType::Game,
        )
        .unwrap();
        assert_eq!(state_game_new.id(), 123);
        let state_game_new = state_game_new.cast::<StateGameInit>().unwrap();
        assert_eq!(state_game_new.id, 123);
        assert_eq!(state_game_new.typ, StateType::GameInit);
        assert_eq!(state_game_new.logic_typ, LogicType::Game);

        let state_game_update = test_rkyv(
            Box::new(StateGameUpdate {
                _base: StateBase::new(456, StateType::GameUpdate, LogicType::Game),
                frame: 90,
                id_gen_counter: 1,
            }),
            StateType::GameUpdate,
            LogicType::Game,
        )
        .unwrap();
        assert_eq!(state_game_update.id(), 456);
        let state_game_update = state_game_update.cast::<StateGameUpdate>().unwrap();
        assert_eq!(state_game_update.id, 456);
        assert_eq!(state_game_update.frame, 90);
        assert_eq!(state_game_update.id_gen_counter, 1);
        assert_eq!(state_game_update.typ, StateType::GameUpdate);
        assert_eq!(state_game_update.logic_typ, LogicType::Game);
    }

    #[test]
    fn test_rkyv_state_zone() {
        let state_zone_new = test_rkyv(
            Box::new(StateZoneInit {
                _base: StateBase::new(4321, StateType::ZoneInit, LogicType::Zone),
                view_zone_file: "stage_file.tscn".into(),
            }),
            StateType::ZoneInit,
            LogicType::Zone,
        )
        .unwrap();
        assert_eq!(state_zone_new.id(), 4321);
        let state_zone_new = state_zone_new.cast::<StateZoneInit>().unwrap();
        assert_eq!(state_zone_new.id, 4321);
        assert_eq!(state_zone_new.typ, StateType::ZoneInit);
        assert_eq!(state_zone_new.logic_typ, LogicType::Zone);
        assert_eq!(state_zone_new.view_zone_file, "stage_file.tscn");

        let state_zone_update = test_rkyv(
            Box::new(StateZoneUpdate {
                _base: StateBase::new(8765, StateType::ZoneUpdate, LogicType::Zone),
            }),
            StateType::ZoneUpdate,
            LogicType::Zone,
        )
        .unwrap();
        assert_eq!(state_zone_update.id(), 8765);
        let state_zone_update = state_zone_update.cast::<StateZoneUpdate>().unwrap();
        assert_eq!(state_zone_update.id, 8765);
        assert_eq!(state_zone_update.typ, StateType::ZoneUpdate);
        assert_eq!(state_zone_update.logic_typ, LogicType::Zone);
    }

    #[test]
    fn test_rkyv_state_player() {
        let state_player_new = test_rkyv(
            Box::new(StatePlayerInit {
                _base: StateBase::new(1110, StateType::PlayerInit, LogicType::Player),
                skeleton_file: sb!("skeleton_file.ozz"),
                animation_metas: vec![
                    AnimationFileMeta::new(sb!("animation_file_1.ozz"), false, false),
                    AnimationFileMeta::new(sb!("animation_file_2.ozz"), true, true),
                ],
                view_model: sb!("model.vrm"),
                init_position: CsVec3A::new(1.0, 2.0, 3.0).into(),
                init_direction: Vec2xz::Z,
            }),
            StateType::PlayerInit,
            LogicType::Player,
        )
        .unwrap();
        assert_eq!(state_player_new.id(), 1110);
        let state_player_new = state_player_new.cast::<StatePlayerInit>().unwrap();
        assert_eq!(state_player_new.id, 1110);
        assert_eq!(state_player_new.typ, StateType::PlayerInit);
        assert_eq!(state_player_new.logic_typ, LogicType::Player);
        assert_eq!(state_player_new.skeleton_file, "skeleton_file.ozz");
        assert_eq!(state_player_new.animation_metas, vec![
            AnimationFileMeta::new(sb!("animation_file_1.ozz"), false, false),
            AnimationFileMeta::new(sb!("animation_file_2.ozz"), true, true),
        ]);
        assert_eq!(state_player_new.view_model, "model.vrm");

        let state_player_update = test_rkyv(
            Box::new(StatePlayerUpdate {
                _base: StateBase::new(2220, StateType::PlayerUpdate, LogicType::Player),
                physics: StateCharaPhysics {
                    velocity: Vec3A::ONE.into(),
                    position: Vec3A::new(1.0, 2.0, 3.0).into(),
                    direction: Vec2xz::X,
                },
                actions: Vec::new(),
            }),
            StateType::PlayerUpdate,
            LogicType::Player,
        )
        .unwrap();
        assert_eq!(state_player_update.id(), 2220);
        let state_player_update = state_player_update.cast::<StatePlayerUpdate>().unwrap();
        assert_eq!(state_player_update.id, 2220);
        assert_eq!(state_player_update.typ, StateType::PlayerUpdate);
        assert_eq!(state_player_update.logic_typ, LogicType::Player);
        assert_eq!(state_player_update.physics.velocity, Vec3A::ONE);
        assert_eq!(state_player_update.physics.position, Vec3A::new(1.0, 2.0, 3.0));
        assert_eq!(state_player_update.physics.direction, Vec2xz::X);
        assert_eq!(state_player_update.actions.len(), 0);
    }

    #[test]
    fn test_rkyv_state_enemy() {
        let state_enemy_new = test_rkyv(
            Box::new(StateNpcInit {
                _base: StateBase::new(1111, StateType::NpcInit, LogicType::Npc),
            }),
            StateType::NpcInit,
            LogicType::Npc,
        )
        .unwrap();
        assert_eq!(state_enemy_new.id(), 1111);
        let state_enemy_new = state_enemy_new.cast::<StateNpcInit>().unwrap();
        assert_eq!(state_enemy_new.id, 1111);
        assert_eq!(state_enemy_new.typ, StateType::NpcInit);
        assert_eq!(state_enemy_new.logic_typ, LogicType::Npc);

        let state_enemy_update = test_rkyv(
            Box::new(StateNpcUpdate {
                _base: StateBase::new(2222, StateType::NpcUpdate, LogicType::Npc),
            }),
            StateType::NpcUpdate,
            LogicType::Npc,
        )
        .unwrap();
        assert_eq!(state_enemy_update.id(), 2222);
        let state_enemy_update = state_enemy_update.cast::<StateNpcUpdate>().unwrap();
        assert_eq!(state_enemy_update.id, 2222);
        assert_eq!(state_enemy_update.typ, StateType::NpcUpdate);
        assert_eq!(state_enemy_update.logic_typ, LogicType::Npc);
    }
}
