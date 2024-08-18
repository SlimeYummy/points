mod collection;
mod error;
mod extend;
mod id;
mod key;
mod math;
mod ptr;
mod rc_cell;
mod symbol;

pub use collection::*;
pub use error::*;
pub use id::*;
pub use key::*;
pub use math::*;
pub use ptr::*;
pub use rc_cell::{RcCell, RcCellError, RcCellRef, RcCellRefMut};
pub use symbol::*;

pub(crate) use extend::{extend, interface};

pub type Num = f64;

pub const FPS: u32 = 15;
pub const MAX_PLAYER: usize = 8;
