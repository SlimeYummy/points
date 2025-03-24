use cirtical_point_csgen::CsEnum;
use enum_iterator::{cardinality, Sequence};
use std::fmt::Debug;
use std::mem;

use super::id::TmplID;
use crate::utils::{rkyv_self, serde_by, xres, Castable, Symbol, XError, XResult};

//
// TmplType & TmplAny
//

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, serde::Serialize, serde::Deserialize, CsEnum)]
pub enum TmplType {
    Character,
    Style,
    Equipment,
    Entry,
    Perk,
    AccessoryPool,
    Accessory,
    Jewel,

    ActionGeneral,
    ActionIdle,
    ActionMove,
    ActionDodge,
    ActionGuard,
    ActionAim,

    Zone,
}

rkyv_self!(TmplType);

impl From<TmplType> for u16 {
    #[inline]
    fn from(val: TmplType) -> Self {
        unsafe { mem::transmute::<TmplType, u16>(val) }
    }
}

impl TryFrom<u16> for TmplType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> XResult<Self> {
        if value as usize >= cardinality::<TmplType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, TmplType>(value) })
    }
}

#[typetag::deserialize(tag = "T")]
pub trait TmplAny: Debug {
    fn id(&self) -> TmplID;
    fn typ(&self) -> TmplType;
}

impl Castable for dyn TmplAny {}

pub trait ArchivedTmplAny: Debug {
    fn id(&self) -> TmplID;
    fn typ(&self) -> TmplType;
}

impl Castable for dyn ArchivedTmplAny {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TmplAnyMetadata {
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

    use super::accessory::{ArchivedTmplAccessory, ArchivedTmplAccessoryPool, TmplAccessory, TmplAccessoryPool};
    use super::character::{ArchivedTmplCharacter, ArchivedTmplStyle, TmplCharacter, TmplStyle};
    use super::entry::{ArchivedTmplEntry, TmplEntry};
    use super::equipment::{ArchivedTmplEquipment, TmplEquipment};
    use super::jewel::{ArchivedTmplJewel, TmplJewel};
    use super::zone::{ArchivedTmplZone, TmplZone};
    use crate::utils::CastRef;
    use TmplType::*;

    impl Pointee for dyn TmplAny {
        type Metadata = DynMetadata<dyn TmplAny>;
    }

    impl Pointee for dyn ArchivedTmplAny {
        type Metadata = DynMetadata<dyn ArchivedTmplAny>;
    }

    impl ArchivePointee for dyn ArchivedTmplAny {
        type ArchivedMetadata = TmplAnyMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let typ = TmplType::try_from(archived.typ).expect("Invalid TmplType");
            let archived_ref: &dyn ArchivedTmplAny = unsafe {
                match typ {
                    Character => mem::transmute_copy::<usize, &ArchivedTmplCharacter>(&0),
                    Style => mem::transmute_copy::<usize, &ArchivedTmplStyle>(&0),
                    Equipment => mem::transmute_copy::<usize, &ArchivedTmplEquipment>(&0),
                    Entry => mem::transmute_copy::<usize, &ArchivedTmplEntry>(&0),
                    Accessory => mem::transmute_copy::<usize, &ArchivedTmplAccessory>(&0),
                    AccessoryPool => mem::transmute_copy::<usize, &ArchivedTmplAccessoryPool>(&0),
                    Jewel => mem::transmute_copy::<usize, &ArchivedTmplJewel>(&0),
                    Stage => mem::transmute_copy::<usize, &ArchivedTmplZone>(&0),
                    _ => unreachable!("pointer_metadata() Invalid TmplType"),
                }
            };
            ptr::metadata(archived_ref)
        }
    }

    impl ArchiveUnsized for dyn TmplAny {
        type Archived = dyn ArchivedTmplAny;
        type MetadataResolver = ();

        unsafe fn resolve_metadata(
            &self,
            _pos: usize,
            _resolver: Self::MetadataResolver,
            out: *mut ArchivedMetadata<Self>,
        ) {
            let typ = to_archived!(self.typ().into());
            out.write(TmplAnyMetadata { typ });
        }
    }

    impl<S> SerializeUnsized<S> for dyn TmplAny
    where
        S: Serializer + ScratchSpace + ?Sized,
    {
        fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
            #[inline(always)]
            fn serialize<T, S>(state_any: &(dyn TmplAny + 'static), serializer: &mut S) -> Result<usize, S::Error>
            where
                T: TmplAny + Serialize<S> + 'static,
                S: Serializer + ScratchSpace + ?Sized,
            {
                let state_ref = unsafe { state_any.cast_ref_unchecked::<T>() };
                let resolver = state_ref.serialize(serializer)?;
                serializer.align_for::<T>()?;
                Ok(unsafe { serializer.resolve_aligned(state_ref, resolver)? })
            }

            match self.typ() {
                Character => serialize::<TmplCharacter, _>(self, serializer),
                Style => serialize::<TmplStyle, _>(self, serializer),
                Equipment => serialize::<TmplEquipment, _>(self, serializer),
                Entry => serialize::<TmplEntry, _>(self, serializer),
                Accessory => serialize::<TmplAccessory, _>(self, serializer),
                AccessoryPool => serialize::<TmplAccessoryPool, _>(self, serializer),
                Jewel => serialize::<TmplJewel, _>(self, serializer),
                Stage => serialize::<TmplZone, _>(self, serializer),
                _ => unreachable!("serialize_unsized() Invalid TmplType"),
            }
        }

        fn serialize_metadata(&self, _serializer: &mut S) -> Result<Self::MetadataResolver, S::Error> {
            Ok(())
        }
    }

    impl<D> DeserializeUnsized<dyn TmplAny, D> for dyn ArchivedTmplAny
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
                archived_any: &(dyn ArchivedTmplAny + 'static),
                deserializer: &mut D,
                mut alloc: impl FnMut(Layout) -> *mut u8,
            ) -> Result<*mut (), D::Error>
            where
                T: TmplAny + Archive + 'static,
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
                Character => deserialize::<TmplCharacter, _>(self, deserializer, alloc),
                Style => deserialize::<TmplStyle, _>(self, deserializer, alloc),
                Equipment => deserialize::<TmplEquipment, _>(self, deserializer, alloc),
                Entry => deserialize::<TmplEntry, _>(self, deserializer, alloc),
                Accessory => deserialize::<TmplAccessory, _>(self, deserializer, alloc),
                AccessoryPool => deserialize::<TmplAccessoryPool, _>(self, deserializer, alloc),
                Jewel => deserialize::<TmplJewel, _>(self, deserializer, alloc),
                Stage => deserialize::<TmplZone, _>(self, deserializer, alloc),
                _ => unreachable!("deserialize_unsized() Invalid TmplType"),
            }
        }

        fn deserialize_metadata(&self, _deserializer: &mut D) -> Result<DynMetadata<dyn TmplAny>, D::Error> {
            let value_ref: &dyn TmplAny = unsafe {
                match self.typ() {
                    Character => mem::transmute_copy::<usize, &TmplCharacter>(&0),
                    Style => mem::transmute_copy::<usize, &TmplStyle>(&0),
                    Equipment => mem::transmute_copy::<usize, &TmplEquipment>(&0),
                    Entry => mem::transmute_copy::<usize, &TmplEntry>(&0),
                    Accessory => mem::transmute_copy::<usize, &TmplAccessory>(&0),
                    AccessoryPool => mem::transmute_copy::<usize, &TmplAccessoryPool>(&0),
                    Jewel => mem::transmute_copy::<usize, &TmplJewel>(&0),
                    Stage => mem::transmute_copy::<usize, &TmplZone>(&0),
                    _ => unreachable!("deserialize_metadata() Invalid TmplType"),
                }
            };
            Ok(ptr::metadata(value_ref))
        }
    }
};

//
// utils types
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplRare {
    Rare1 = 1,
    Rare2 = 2,
    Rare3 = 3,
}

rkyv_self!(TmplRare);

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[serde(untagged)]
pub enum TmplSwitch {
    Bool(bool),
    Symbol(Symbol),
}

impl Default for TmplSwitch {
    #[inline]
    fn default() -> Self {
        TmplSwitch::Bool(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TmplLevelRange {
    pub min: u32,
    pub max: u32,
}

rkyv_self!(TmplLevelRange);
serde_by!(TmplLevelRange, [u32; 2], TmplLevelRange::from, TmplLevelRange::to_array);

impl TmplLevelRange {
    #[inline]
    pub fn new(min: u32, max: u32) -> TmplLevelRange {
        TmplLevelRange { min, max }
    }

    #[inline]
    pub fn to_array(&self) -> [u32; 2] {
        [self.min, self.max]
    }

    #[inline]
    pub fn to_tuple(&self) -> (u32, u32) {
        (*self).into()
    }
}

impl From<[u32; 2]> for TmplLevelRange {
    #[inline]
    fn from(range: [u32; 2]) -> TmplLevelRange {
        TmplLevelRange::new(range[0], range[1])
    }
}

impl From<TmplLevelRange> for [u32; 2] {
    #[inline]
    fn from(val: TmplLevelRange) -> Self {
        [val.min, val.max]
    }
}

impl From<(u32, u32)> for TmplLevelRange {
    #[inline]
    fn from(range: (u32, u32)) -> TmplLevelRange {
        TmplLevelRange::new(range.0, range.1)
    }
}

impl From<TmplLevelRange> for (u32, u32) {
    #[inline]
    fn from(val: TmplLevelRange) -> Self {
        (val.min, val.max)
    }
}
