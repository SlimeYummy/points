use crate::template::base::TmplSwitch;
use crate::utils::{KeyCode, StrID, Symbol};

pub const LEVEL_FREE: u16 = 0;
pub const LEVEL_ATTACK: u16 = 100;
pub const LEVEL_SKILL: u16 = 200;
pub const LEVEL_SUPER_SKILL: u16 = 300;
pub const LEVEL_PROGRESSING: u16 = 400;
pub const LEVEL_UNCONTROLABLE: u16 = 500;
pub const LEVEL_UNBREAKABLE: u16 = 600;

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplAnimation {
    pub file: Symbol,
    #[with(RkyvNonZero)]
    pub duration: u32,
    pub times: u32,
    pub additive: bool,
    pub body_progress: Option<u32>,
}

impl TmplAnimation {
    #[inline]
    pub fn is_infinity(&self) -> bool {
        self.times == 0
    }

    #[inline]
    pub fn ratio(&self, frame: u32) -> f32 {
        if self.times == 0 || frame < self.duration * self.times {
            let norm_frame = frame % self.duration;
            (norm_frame as f32) / (self.duration as f32)
        } else {
            1.0
        }
    }
}

struct RkyvNonZero;

const _: () = {
    use rkyv::with::{ArchiveWith, DeserializeWith, SerializeWith};
    use rkyv::{Archive, Archived, Deserialize, Fallible, Resolver, Serialize};

    impl ArchiveWith<i32> for RkyvNonZero {
        type Archived = Archived<i32>;
        type Resolver = Resolver<i32>;

        #[inline]
        unsafe fn resolve_with(field: &i32, pos: usize, _: (), out: *mut Self::Archived) {
            field.resolve(pos, (), out);
        }
    }

    impl<S: Fallible + ?Sized> SerializeWith<i32, S> for RkyvNonZero
    where
        i32: Serialize<S>,
    {
        #[inline]
        fn serialize_with(field: &i32, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            field.serialize(serializer)
        }
    }

    impl<D: Fallible + ?Sized> DeserializeWith<Archived<i32>, i32, D> for RkyvNonZero
    where
        Archived<i32>: Deserialize<i32, D>,
    {
        #[inline]
        fn deserialize_with(field: &Archived<i32>, deserializer: &mut D) -> Result<i32, D::Error> {
            let value = field.deserialize(deserializer)?;
            Ok(value.max(1))
        }
    }
};

#[derive(Debug, Clone, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplDeriveAction {
    pub key: KeyCode,
    pub enabled: TmplSwitch,
    pub action: StrID,
}

// #[derive(
//     Debug,
//     Clone,
//     Copy,
//     PartialEq,
//     Eq,
//     Hash,
//     serde::Deserialize,
//     rkyv::Archive,
//     rkyv::Serialize,
//     rkyv::Deserialize,
// )]
// pub enum TmplHitType {
//     Normal,
//     Skill,
//     Passive,
//     Other,
// }

// #[derive(
//     Debug,
//     Clone,
//     Copy,
//     PartialEq,
//     Eq,
//     Hash,
//     serde::Deserialize,
//     rkyv::Archive,
//     rkyv::Serialize,
//     rkyv::Deserialize,
// )]
// pub enum TmplHitRange {
//     Melee,
//     Shoot,
//     Area,
//     Other,
// }

// #[derive(Debug, Clone, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
// pub struct TmplHit {
//     #[serde(rename = "type")]
//     typ: TmplHitType,
// }
