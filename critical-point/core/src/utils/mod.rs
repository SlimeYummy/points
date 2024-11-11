mod collection;
mod error;
mod extend;
mod id;
mod key;
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

pub(crate) use extend::{extend, interface};

pub type Num = f64;
