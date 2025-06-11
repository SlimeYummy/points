use static_assertions::const_assert_eq;
use std::{fmt, mem};

use crate::utils::TmplID;

#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[serde(untagged)]
pub enum TmplVar<T>
where
    T: Clone + Copy + Default,
{
    Value(T),
    Values(#[rkyv(with = rkyv::with::AsBox)] TmplVarValues<T>),
}

impl<T> TmplVar<T>
where
    T: Clone + Copy + Default,
{
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
    pub fn get(&self, idx: usize) -> T {
        match self {
            TmplVar::Value(val) => *val,
            TmplVar::Values(list) => match list.values.get(idx) {
                Some(val) => *val,
                None => *list.values.last().unwrap_or(&T::default()),
            },
        }
    }
}

impl<T> fmt::Debug for ArchivedTmplVar<T>
where
    T: Clone + Copy + Default + rkyv::Archive,
    T::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchivedTmplVar::Value(val) => write!(f, "Value({val:?})"),
            ArchivedTmplVar::Values(list) => write!(f, "Values({list:?})"),
        }
    }
}

const_assert_eq!(mem::size_of::<ArchivedTmplVar<bool>>(), 8);
const_assert_eq!(mem::size_of::<ArchivedTmplVar<f32>>(), 8);
const_assert_eq!(mem::size_of::<ArchivedTmplVar<u64>>(), 16);

impl<T> ArchivedTmplVar<T>
where
    T: Clone + Copy + Default + rkyv::Archive,
    T::Archived: Clone + Copy + Default,
{
    #[inline]
    pub fn is_value(&self) -> bool {
        matches!(self, ArchivedTmplVar::Value(_))
    }

    #[inline]
    pub fn is_values(&self) -> bool {
        matches!(self, ArchivedTmplVar::Values(_))
    }

    #[inline]
    pub fn value(&self) -> Option<T::Archived> {
        match self {
            ArchivedTmplVar::Value(val) => Some(*val),
            ArchivedTmplVar::Values(_) => None,
        }
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
    pub fn values(&self) -> Option<&[T::Archived]> {
        match self {
            ArchivedTmplVar::Value(_) => None,
            ArchivedTmplVar::Values(list) => Some(&list.values),
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> T::Archived {
        match self {
            ArchivedTmplVar::Value(val) => *val,
            ArchivedTmplVar::Values(list) => match list.values.get(idx) {
                Some(val) => *val,
                None => *list.values.last().unwrap_or(&T::Archived::default()),
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
pub struct TmplVarValues<T>
where
    T: Clone + Copy + Default,
{
    pub id: TmplID,
    pub values: Vec<T>,
}

impl<T> TmplVarValues<T>
where
    T: Clone + Copy + Default,
{
    #[inline]
    pub fn get(&self, idx: usize) -> T {
        match self.values.get(idx) {
            Some(val) => *val,
            None => *self.values.last().unwrap_or(&T::default()),
        }
    }
}

impl<T> ArchivedTmplVarValues<T>
where
    T: Clone + Copy + Default + rkyv::Archive,
    T::Archived: Clone + Copy + Default,
{
    #[inline]
    pub fn get(&self, idx: usize) -> T::Archived {
        match self.values.get(idx) {
            Some(val) => *val,
            None => *self.values.last().unwrap_or(&T::Archived::default()),
        }
    }
}

impl<T> fmt::Debug for ArchivedTmplVarValues<T>
where
    T: Clone + Copy + Default + rkyv::Archive,
    T::Archived: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchivedTmplVarValues")
            .field("id", &self.id)
            .field("values", &self.values)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::id;

    #[test]
    fn test_tmpl_var_json() {
        let var1 = TmplVar::Value(1);
        let json1 = serde_json::to_string(&var1).unwrap();
        let var11 = serde_json::from_str(&json1).unwrap();
        assert_eq!(var1, var11);

        let var2 = TmplVar::Values(TmplVarValues {
            id: id!("#.Aaa.Bbb"),
            values: vec![1, 2, 3],
        });
        let json2 = serde_json::to_string(&var2).unwrap();
        let var22 = serde_json::from_str(&json2).unwrap();
        assert_eq!(var2, var22);
    }

    #[test]
    fn test_tmpl_var_rkyv() {
        use rkyv::rancor::Error;

        let var1 = TmplVar::Value(1);
        let buf1 = rkyv::to_bytes::<Error>(&var1).unwrap();
        let var11 = unsafe { rkyv::access_unchecked::<ArchivedTmplVar<i32>>(buf1.as_ref()) };
        assert!(var11.is_value());
        assert_eq!(var11.value().unwrap(), 1);

        let var2 = TmplVar::Values(TmplVarValues {
            id: id!("#.Aaa.Bbb"),
            values: vec![1, 2, 3],
        });
        let buf2 = rkyv::to_bytes::<Error>(&var2).unwrap();
        let var22 = unsafe { rkyv::access_unchecked::<ArchivedTmplVar<i32>>(buf2.as_ref()) };
        assert!(var22.is_values());
        assert_eq!(var22.id().unwrap(), id!("#.Aaa.Bbb"));
        assert_eq!(var22.values().unwrap(), &[1, 2, 3]);
    }
}
