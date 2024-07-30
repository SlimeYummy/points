mod collection;
mod error;
mod extend;
mod id;
mod key;
mod math;
mod rc_cell;
mod rc_ptr;
mod symbol;

pub use collection::*;
pub use error::*;
pub use extend::*;
pub use id::*;
pub use key::*;
pub use math::*;
pub use rc_cell::{RcCell, RcCellError, RcCellRef, RcCellRefMut};
pub use rc_ptr::{
    const_ptr, mut_ptr, size_of_array, size_of_type, CastArc, CastRc, CastRef, Xcast, Xrc, Xweak,
};
pub use symbol::*;

pub type Num = f64;

pub const FPS: u64 = 15;
