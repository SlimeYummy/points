#![allow(unused_imports)]
#![allow(unused_macros)]

pub use anyhow::{Error, Result, anyhow};

#[macro_export]
macro_rules! xerr {
    ($variant:ident) => {
        anyhow!(
            "{}  [P]: {}:{}",
            stringify!($variant),
            file!(),
            line!()
        )
    };
    ($variant:ident, $payload:expr) => {
        anyhow!(
            "{}({})  [P]: {}:{}",
            stringify!($variant),
            $payload,
            file!(),
            line!()
        )
    };
    ($variant:ident; $extra:expr) => {
        anyhow!(
            "{}  [P]: {}:{} [M]: {}",
            stringify!($variant),
            file!(),
            line!(),
            $extra
        )
    };
    ($variant:ident, $payload:expr; $extra:expr) => {
        anyhow!(
            "{}({})  [P]: {}:{} [M]: {}",
            stringify!($variant),
            $payload,
            file!(),
            line!()
            $extra
        )
    };
}
pub use xerr;

#[macro_export]
macro_rules! xerrf {
    ($variant:ident; $($args:tt)*) => {
        anyhow!(
            "{}  [P]: {}:{} [M]: {}",
            stringify!($variant),
            file!(),
            line!(),
            format!($($args)*)
        )
    };
    ($variant:ident, $payload:expr; $($args:tt)*) => {
        anyhow!(
            "{}({})  [P]: {}:{} [M]: {}",
            stringify!($variant),
            $payload,
            file!(),
            line!(),
            format!($($args)*)
        )
    };
}
pub use xerrf;

#[macro_export]
macro_rules! xres {
    ($variant:ident) => {
        Err($crate::xerr!($variant))
    };
    ($variant:ident; $extra:expr) => {
        Err($crate::xerr!($variant; $extra))
    };
    ($variant:ident, $payload:expr) => {
        Err($crate::xerr!($variant, $payload))
    };
    ($variant:ident, $payload:expr; $extra:expr) => {
        Err($crate::xerr!($variant, $payload; $extra))
    };
}
pub use xres;

#[macro_export]
macro_rules! xresf {
    ($variant:ident; $($args:tt)*) => {
        Err($crate::xerrf!($variant; $($args)*))
    };
    ($variant:ident, $payload:expr; $($args:tt)*) => {
        Err($crate::xerrf!($variant, $payload; $($args)*))
    };
}
pub use xresf;

macro_rules! xfrom {
    () => {
        |e| anyhow!("Error  [P] {}:{} [S]: {}", file!(), line!(), e).into()
    };
    ($extra:expr) => {
        |e| anyhow!("Error  [P] {}:{} [M]: {} [S]: {}", file!(), line!(), $extra, e).into()
    };
}
pub(crate) use xfrom;

macro_rules! xfromf {
    ($($args:tt)*) => {
        |e| {
            let msg = format!($($args)*);
            anyhow!("Error  [P] {}:{} [M]: {} [S]: {}", file!(), line!(), msg, e).into()
        }
    };
}
pub(crate) use xfromf;

//
// error buffer
//

static mut HOST_ERROR: [u8; 1024] = [0u8; 1024];

pub struct HostError;

impl HostError {
    #[inline(always)]
    pub fn buffer() -> &'static mut [u8] {
        #[allow(static_mut_refs)]
        unsafe {
            &mut HOST_ERROR
        }
    }

    pub fn read_string(len: usize) -> Option<String> {
        if len == 0 {
            return None;
        }
        let len = usize::min(len, 1024);
        Some(String::from_utf8_lossy(&Self::buffer()[..len]).to_string())
    }

    pub fn read_error(len: usize) -> Option<Error> {
        match Self::read_string(len) {
            Some(msg) => Some(anyhow!(msg)),
            None => None,
        }
    }

    pub fn read_result(len: usize) -> Result<()> {
        match Self::read_string(len) {
            Some(msg) => Err(anyhow!(msg)),
            None => Ok(()),
        }
    }

    pub fn write_string(s: &str) -> usize {
        let len = usize::min(s.len(), 1024);
        unsafe { HOST_ERROR[..len].copy_from_slice(&s.as_bytes()[..len]) }
        len
    }

    pub fn write_error(e: Error) -> usize {
        Self::write_string(&e.to_string())
    }
}
