use critical_point_csgen::CsEnum;
use enum_iterator::{cardinality, Sequence};
use std::alloc::Layout;
use std::any::Any;
use std::{fmt, mem};

use crate::utils::{rkyv_self, xres, Castable, IdentityState, TmplID, XError, XResult};

pub type TmplHashMap<V> = std::collections::HashMap<TmplID, V, IdentityState>;
pub type TmplHashSet = std::collections::HashSet<TmplID, IdentityState>;

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

    Zone,

    ActionEmpty,
    ActionIdle,
    ActionMove,
    ActionGeneral,
    ActionDodge,
    ActionGuard,
    ActionAim,
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
    fn try_from(val: u16) -> XResult<Self> {
        if val as usize >= cardinality::<TmplType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, TmplType>(val) })
    }
}

impl From<TmplType> for rkyv::primitive::ArchivedU16 {
    #[inline]
    fn from(val: TmplType) -> Self {
        unsafe { mem::transmute::<TmplType, u16>(val) }.into()
    }
}

impl TryFrom<rkyv::primitive::ArchivedU16> for TmplType {
    type Error = XError;

    #[inline]
    fn try_from(val: rkyv::primitive::ArchivedU16) -> XResult<Self> {
        if val.to_native() as usize >= cardinality::<TmplType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, TmplType>(val.to_native()) })
    }
}

#[typetag::serde(tag = "T")]
pub trait TmplAny: fmt::Debug + Any {
    fn id(&self) -> TmplID;
    fn typ(&self) -> TmplType;
    fn layout(&self) -> Layout;
}

pub trait ArchivedTmplAny: fmt::Debug + Any {
    fn id(&self) -> TmplID;
    fn typ(&self) -> TmplType;
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, rkyv::Portable)]
pub struct TmplAnyMetadata(rkyv::primitive::ArchivedU16);

impl Default for TmplAnyMetadata {
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

    use super::accessory::{ArchivedTmplAccessory, ArchivedTmplAccessoryPool, TmplAccessory, TmplAccessoryPool};
    use super::action::{
        ArchivedTmplActionGeneral, ArchivedTmplActionIdle, ArchivedTmplActionMove, TmplActionGeneral, TmplActionIdle,
        TmplActionMove,
    };
    use super::character::{ArchivedTmplCharacter, ArchivedTmplStyle, TmplCharacter, TmplStyle};
    use super::entry::{ArchivedTmplEntry, TmplEntry};
    use super::equipment::{ArchivedTmplEquipment, TmplEquipment};
    use super::jewel::{ArchivedTmplJewel, TmplJewel};
    use super::perk::{ArchivedTmplPerk, TmplPerk};
    use super::zone::{ArchivedTmplZone, TmplZone};
    use TmplType::*;

    impl LayoutRaw for dyn TmplAny {
        fn layout_raw(metadata: DynMetadata<dyn TmplAny>) -> Result<Layout, LayoutError> {
            unsafe {
                let null = ptr::from_raw_parts::<dyn TmplAny>(ptr::null() as *const u8, metadata);
                Ok((*null).layout())
            }
        }
    }

    unsafe impl Pointee for dyn TmplAny {
        type Metadata = DynMetadata<dyn TmplAny>;
    }

    unsafe impl Pointee for dyn ArchivedTmplAny {
        type Metadata = DynMetadata<dyn ArchivedTmplAny>;
    }

    unsafe impl Portable for dyn ArchivedTmplAny {}

    unsafe impl NoUndef for TmplAnyMetadata {}

    impl ArchivePointee for dyn ArchivedTmplAny {
        type ArchivedMetadata = TmplAnyMetadata;

        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
            let typ = TmplType::try_from(archived.0).expect("Invalid TmplType");
            let archived_ref: &dyn ArchivedTmplAny = unsafe {
                match typ {
                    Character => mem::transmute_copy::<usize, &ArchivedTmplCharacter>(&0),
                    Style => mem::transmute_copy::<usize, &ArchivedTmplStyle>(&0),
                    Equipment => mem::transmute_copy::<usize, &ArchivedTmplEquipment>(&0),
                    Entry => mem::transmute_copy::<usize, &ArchivedTmplEntry>(&0),
                    Perk => mem::transmute_copy::<usize, &ArchivedTmplPerk>(&0),
                    Accessory => mem::transmute_copy::<usize, &ArchivedTmplAccessory>(&0),
                    AccessoryPool => mem::transmute_copy::<usize, &ArchivedTmplAccessoryPool>(&0),
                    Jewel => mem::transmute_copy::<usize, &ArchivedTmplJewel>(&0),
                    Zone => mem::transmute_copy::<usize, &ArchivedTmplZone>(&0),
                    ActionIdle => mem::transmute_copy::<usize, &ArchivedTmplActionIdle>(&0),
                    ActionMove => mem::transmute_copy::<usize, &ArchivedTmplActionMove>(&0),
                    ActionGeneral => mem::transmute_copy::<usize, &ArchivedTmplActionGeneral>(&0),
                    _ => unreachable!("pointer_metadata() Invalid TmplType"),
                }
            };
            ptr::metadata(archived_ref)
        }
    }

    impl ArchiveUnsized for dyn TmplAny {
        type Archived = dyn ArchivedTmplAny;

        fn archived_metadata(&self) -> ArchivedMetadata<Self> {
            TmplAnyMetadata(self.typ().into())
        }
    }

    impl<S> SerializeUnsized<S> for dyn TmplAny
    where
        S: Fallible + Allocator + Writer + ?Sized,
        S::Error: Source,
    {
        fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
            #[inline(always)]
            fn serialize<T, S>(state_any: &(dyn TmplAny + 'static), serializer: &mut S) -> Result<usize, S::Error>
            where
                T: TmplAny + Serialize<S> + 'static,
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
                Character => serialize::<TmplCharacter, _>(self, serializer),
                Style => serialize::<TmplStyle, _>(self, serializer),
                Equipment => serialize::<TmplEquipment, _>(self, serializer),
                Entry => serialize::<TmplEntry, _>(self, serializer),
                Perk => serialize::<TmplPerk, _>(self, serializer),
                Accessory => serialize::<TmplAccessory, _>(self, serializer),
                AccessoryPool => serialize::<TmplAccessoryPool, _>(self, serializer),
                Jewel => serialize::<TmplJewel, _>(self, serializer),
                Zone => serialize::<TmplZone, _>(self, serializer),
                ActionIdle => serialize::<TmplActionIdle, _>(self, serializer),
                ActionMove => serialize::<TmplActionMove, _>(self, serializer),
                ActionGeneral => serialize::<TmplActionGeneral, _>(self, serializer),
                _ => unreachable!("serialize_unsized() Invalid TmplType"),
            }
        }
    }

    impl<D> DeserializeUnsized<dyn TmplAny, D> for dyn ArchivedTmplAny
    where
        D: Fallible + ?Sized,
        D::Error: Source,
    {
        unsafe fn deserialize_unsized(&self, deserializer: &mut D, out: *mut dyn TmplAny) -> Result<(), D::Error> {
            #[inline(always)]
            fn deserialize<T, D>(
                archived_any: &(dyn ArchivedTmplAny + 'static),
                deserializer: &mut D,
                out: *mut dyn TmplAny,
            ) -> Result<(), D::Error>
            where
                T: TmplAny + Archive + 'static,
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
                Character => deserialize::<TmplCharacter, _>(self, deserializer, out),
                Style => deserialize::<TmplStyle, _>(self, deserializer, out),
                Equipment => deserialize::<TmplEquipment, _>(self, deserializer, out),
                Entry => deserialize::<TmplEntry, _>(self, deserializer, out),
                Perk => deserialize::<TmplPerk, _>(self, deserializer, out),
                Accessory => deserialize::<TmplAccessory, _>(self, deserializer, out),
                AccessoryPool => deserialize::<TmplAccessoryPool, _>(self, deserializer, out),
                Jewel => deserialize::<TmplJewel, _>(self, deserializer, out),
                Zone => deserialize::<TmplZone, _>(self, deserializer, out),
                ActionIdle => deserialize::<TmplActionIdle, _>(self, deserializer, out),
                ActionMove => deserialize::<TmplActionMove, _>(self, deserializer, out),
                ActionGeneral => deserialize::<TmplActionGeneral, _>(self, deserializer, out),
                _ => unreachable!("deserialize_unsized() Invalid TmplType"),
            }
        }

        fn deserialize_metadata(&self) -> DynMetadata<dyn TmplAny> {
            let value_ref: &dyn TmplAny = unsafe {
                match self.typ() {
                    Character => mem::transmute_copy::<usize, &TmplCharacter>(&0),
                    Style => mem::transmute_copy::<usize, &TmplStyle>(&0),
                    Equipment => mem::transmute_copy::<usize, &TmplEquipment>(&0),
                    Entry => mem::transmute_copy::<usize, &TmplEntry>(&0),
                    Perk => mem::transmute_copy::<usize, &TmplPerk>(&0),
                    Accessory => mem::transmute_copy::<usize, &TmplAccessory>(&0),
                    AccessoryPool => mem::transmute_copy::<usize, &TmplAccessoryPool>(&0),
                    Jewel => mem::transmute_copy::<usize, &TmplJewel>(&0),
                    Zone => mem::transmute_copy::<usize, &TmplZone>(&0),
                    ActionIdle => mem::transmute_copy::<usize, &TmplActionIdle>(&0),
                    ActionMove => mem::transmute_copy::<usize, &TmplActionMove>(&0),
                    ActionGeneral => mem::transmute_copy::<usize, &TmplActionGeneral>(&0),
                    _ => unreachable!("deserialize_metadata() Invalid TmplType"),
                }
            };
            ptr::metadata(value_ref)
        }
    }
};

macro_rules! impl_tmpl {
    ($typ:ty, $tmpl_enum:ident, $tmpl_tag:expr) => {
        paste::paste! {
            #[typetag::serde(name = $tmpl_tag)]
            impl $crate::template::TmplAny for $typ {
                #[inline]
                fn id(&self) -> $crate::utils::TmplID {
                    self.id
                }

                #[inline]
                fn typ(&self) -> $crate::template::TmplType {
                    $crate::template::TmplType::$tmpl_enum
                }

                #[inline]
                fn layout(&self) -> std::alloc::Layout {
                    std::alloc::Layout::new::<Self>()
                }
            }

            impl $crate::template::ArchivedTmplAny for [<Archived $typ>] {
                #[inline]
                fn id(&self) -> crate::utils::TmplID {
                    self.id
                }

                #[inline]
                fn typ(&self) -> $crate::template::TmplType {
                    $crate::template::TmplType::$tmpl_enum
                }
            }
        }
    };
}
pub(crate) use impl_tmpl;
