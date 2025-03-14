mod collection;
mod error;
mod id;
mod key;
mod macros;
mod math;
mod ptr;
mod shape;
mod symbol;

pub use collection::*;
pub use error::*;
pub use id::*;
pub use key::*;
pub use math::*;
pub use ptr::*;
pub use shape::*;
pub use symbol::*;

pub(crate) use macros::{extend, interface, rkyv_self, serde_by};
pub(crate) use math::near;

pub type Num = f64;
