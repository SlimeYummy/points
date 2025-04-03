use crate::template2::id::TmplID;
use crate::utils::Num;

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplVar {
    Value(Num),
    Values(#[with(rkyv::with::AsBox)] TmplVarValues),
}

impl TmplVar {
    #[inline]
    pub fn is_value(&self) -> bool {
        matches!(self, TmplVar::Value(_))
    }

    #[inline]
    pub fn is_values(&self) -> bool {
        matches!(self, TmplVar::Values(_))
    }

    #[inline]
    pub fn id(&self) -> Option<TmplID> {
        match self {
            TmplVar::Value(_) => None,
            TmplVar::Values(list) => Some(list.id.clone()),
        }
    }
    
    #[inline]
    pub fn len(&self) -> Option<usize> {
        match self {
            TmplVar::Value(_) => None,
            TmplVar::Values(list) => Some(list.values.len()),
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Num {
        match self {
            TmplVar::Value(val) => *val,
            TmplVar::Values(list) => {
                match list.values.get(idx) {
                    Some(val) => *val,
                    None => *list.values.last().unwrap_or(&0.0),
                }
            },
        }
    }
}

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
)]
pub struct TmplVarValues {
    pub id: TmplID,
    pub values: Vec<Num>,
}

impl ArchivedTmplVar {
    #[inline]
    pub fn is_value(&self) -> bool {
        matches!(self, ArchivedTmplVar::Value(_))
    }

    #[inline]
    pub fn is_values(&self) -> bool {
        matches!(self, ArchivedTmplVar::Values(_))
    }

    #[inline]
    pub fn id(&self) -> Option<TmplID> {
        match self {
            ArchivedTmplVar::Value(_) => None,
            ArchivedTmplVar::Values(list) => Some(list.id.clone()),
        }
    }
    
    #[inline]
    pub fn len(&self) -> Option<usize> {
        match self {
            ArchivedTmplVar::Value(_) => None,
            ArchivedTmplVar::Values(list) => Some(list.values.len()),
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Num {
        match self {
            ArchivedTmplVar::Value(val) => *val,
            ArchivedTmplVar::Values(list) => {
                match list.values.get(idx) {
                    Some(val) => *val,
                    None => *list.values.last().unwrap_or(&0.0),
                }
            },
        }
    }
}
