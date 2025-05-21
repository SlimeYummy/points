use jolt_physics_rs::JoltError;
use ozz_animation_rs::OzzError;
use std::error::Error;
use std::fmt;

use crate::utils::id::TmplID;

const EMPTY_ERROR: &'static str = "";

#[derive(Debug)]
pub(crate) struct StaticError<T> {
    src: T,
    pos: &'static &'static str,
}

#[derive(Debug)]
pub(crate) struct DynamicError<T> {
    src: T,
    pos: &'static &'static str,
    msg: String,
}

#[allow(private_interfaces)]
#[derive(Debug)]
pub enum MixedError<T> {
    Static(StaticError<T>),
    Dynamic(Box<DynamicError<T>>),
}

impl<T> MixedError<T> {
    pub fn from_static(src: T, pos: &'static &'static str) -> Self {
        MixedError::Static(StaticError { src, pos })
    }

    pub fn from_dynamic(src: T, pos: &'static &'static str, msg: String) -> Self {
        MixedError::Dynamic(Box::new(DynamicError { src, pos, msg }))
    }

    pub(crate) fn pos(&self) -> &'static str {
        match self {
            MixedError::Static(e) => *e.pos,
            MixedError::Dynamic(e) => *e.pos,
        }
    }

    pub(crate) fn msg(&self) -> &str {
        match self {
            MixedError::Static(_) => EMPTY_ERROR,
            MixedError::Dynamic(e) => e.msg.as_str(),
        }
    }

    pub(crate) fn src(&self) -> &T {
        match self {
            MixedError::Static(e) => &e.src,
            MixedError::Dynamic(e) => &e.src,
        }
    }

    fn set_pos(mut self, pos: &'static &'static str) -> Self {
        match self {
            MixedError::Static(ref mut e) => {
                e.pos = pos;
            }
            MixedError::Dynamic(ref mut e) => {
                e.pos = pos;
            }
        };
        self
    }

    fn set_msg(mut self, msg: String) -> Self {
        match self {
            MixedError::Static(e) => MixedError::from_dynamic(e.src, e.pos, msg),
            MixedError::Dynamic(ref mut e) => {
                e.msg = msg;
                self
            }
        }
    }
}

trait ToString {
    fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result;
}

impl ToString for MixedError<()> {
    fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
        match self {
            MixedError::Static(e) => write!(f, "{}  [P]: {}", name, e.pos),
            MixedError::Dynamic(e) => write!(f, "{}  [P]: {}  [M]: {}", name, e.pos, e.msg),
        }
    }
}

macro_rules! val_to_string {
    ($typ:path) => {
        impl ToString for MixedError<$typ> {
            fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
                match self {
                    MixedError::Static(e) => write!(f, "{}({})  [P]: {}", name, e.src, e.pos),
                    MixedError::Dynamic(e) => write!(f, "{}({})  [P]: {}  [M]: {}", name, e.src, e.pos, e.msg),
                }
            }
        }
    };
}

val_to_string!(u64);
val_to_string!(TmplID);

macro_rules! err_to_string {
    ($err:path, $err_name:expr) => {
        impl ToString for MixedError<$err> {
            fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
                match self {
                    MixedError::Static(e) => write!(f, "{}({})  [P]: {}  [S]: {}", name, $err_name, e.pos, e.src),
                    MixedError::Dynamic(e) => write!(
                        f,
                        "{}({})  [P]: {}  [M]: {}  [S]: {}",
                        name, $err_name, e.pos, e.msg, e.src
                    ),
                }
            }
        }
    };
}

err_to_string!(std::io::Error, "io::Error");
err_to_string!(std::str::Utf8Error, "str::Utf8Error");
err_to_string!(serde_json::Error, "serde_json::Error");
err_to_string!(JoltError, "JoltError");
err_to_string!(OzzError, "OzzError");

#[derive(Debug)]
pub enum XError {
    Unexpected(MixedError<()>),

    BadArgument(MixedError<()>),
    NotFound(MixedError<()>),
    BadType(MixedError<()>),
    BadOperation(MixedError<()>),
    Overflow(MixedError<()>),

    SymbolTooLong(MixedError<()>),
    SymbolNotFound(MixedError<()>),
    SymbolNotPreloaded(MixedError<()>),

    BadParameter(MixedError<()>),
    BadAttribute(MixedError<()>),
    BadAsset(MixedError<()>),
    BadScript(MixedError<()>),
    BadAction(MixedError<()>),

    ScriptNoHook(MixedError<()>),
    ScriptOutOfRange(MixedError<()>),
    ScriptBadCommand(MixedError<()>),
    ScriptStackOverflow(MixedError<()>),

    TmplNotFound(MixedError<TmplID>),

    LogicNotFound(MixedError<u64>),
    LogicBadState(MixedError<()>),
    LogicIDMismatch(MixedError<()>),

    IO(MixedError<std::io::Error>),
    Utf8(MixedError<std::str::Utf8Error>),
    Json(MixedError<serde_json::Error>),
    Zip(MixedError<std::io::Error>),
    Jolt(MixedError<JoltError>),
    Ozz(MixedError<OzzError>),
    Rkyv(MixedError<()>), // no source here
}

impl Error for XError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            XError::IO(e) => Some(e.src()),
            XError::Utf8(e) => Some(e.src()),
            XError::Json(e) => Some(e.src()),
            XError::Zip(e) => Some(e.src()),
            XError::Jolt(e) => Some(e.src()),
            XError::Ozz(e) => Some(e.src()),
            _ => None,
        }
    }
}

macro_rules! switch_call {
    ($self:expr, $func:expr) => {
        match $self {
            XError::Unexpected(e) => $func(e),
            XError::BadArgument(e) => $func(e),
            XError::NotFound(e) => $func(e),
            XError::BadType(e) => $func(e),
            XError::BadOperation(e) => $func(e),
            XError::Overflow(e) => $func(e),
            XError::SymbolTooLong(e) => $func(e),
            XError::SymbolNotFound(e) => $func(e),
            XError::SymbolNotPreloaded(e) => $func(e),
            XError::BadParameter(e) => $func(e),
            XError::BadAttribute(e) => $func(e),
            XError::BadAsset(e) => $func(e),
            XError::BadScript(e) => $func(e),
            XError::BadAction(e) => $func(e),
            XError::ScriptNoHook(e) => $func(e),
            XError::ScriptOutOfRange(e) => $func(e),
            XError::ScriptBadCommand(e) => $func(e),
            XError::ScriptStackOverflow(e) => $func(e),
            XError::TmplNotFound(e) => $func(e),
            XError::LogicNotFound(e) => $func(e),
            XError::LogicBadState(e) => $func(e),
            XError::LogicIDMismatch(e) => $func(e),
            XError::IO(e) => $func(e),
            XError::Utf8(e) => $func(e),
            XError::Json(e) => $func(e),
            XError::Zip(e) => $func(e),
            XError::Jolt(e) => $func(e),
            XError::Ozz(e) => $func(e),
            XError::Rkyv(e) => $func(e),
        }
    };
}

macro_rules! switch_new {
    ($self:expr, $func:expr) => {
        match $self {
            XError::Unexpected(e) => XError::Unexpected($func(e)),
            XError::BadArgument(e) => XError::BadArgument($func(e)),
            XError::NotFound(e) => XError::NotFound($func(e)),
            XError::BadType(e) => XError::BadType($func(e)),
            XError::BadOperation(e) => XError::BadOperation($func(e)),
            XError::Overflow(e) => XError::Overflow($func(e)),
            XError::SymbolTooLong(e) => XError::SymbolTooLong($func(e)),
            XError::SymbolNotFound(e) => XError::SymbolNotFound($func(e)),
            XError::SymbolNotPreloaded(e) => XError::SymbolNotPreloaded($func(e)),
            XError::BadParameter(e) => XError::BadParameter($func(e)),
            XError::BadAttribute(e) => XError::BadAttribute($func(e)),
            XError::BadAsset(e) => XError::BadAsset($func(e)),
            XError::BadScript(e) => XError::BadScript($func(e)),
            XError::BadAction(e) => XError::BadAction($func(e)),
            XError::ScriptNoHook(e) => XError::ScriptNoHook($func(e)),
            XError::ScriptOutOfRange(e) => XError::ScriptOutOfRange($func(e)),
            XError::ScriptBadCommand(e) => XError::ScriptBadCommand($func(e)),
            XError::ScriptStackOverflow(e) => XError::ScriptStackOverflow($func(e)),
            XError::TmplNotFound(e) => XError::TmplNotFound($func(e)),
            XError::LogicNotFound(e) => XError::LogicNotFound($func(e)),
            XError::LogicBadState(e) => XError::LogicBadState($func(e)),
            XError::LogicIDMismatch(e) => XError::LogicIDMismatch($func(e)),
            XError::IO(e) => XError::IO($func(e)),
            XError::Utf8(e) => XError::Utf8($func(e)),
            XError::Json(e) => XError::Json($func(e)),
            XError::Zip(e) => XError::Zip($func(e)),
            XError::Jolt(e) => XError::Jolt($func(e)),
            XError::Ozz(e) => XError::Ozz($func(e)),
            XError::Rkyv(e) => XError::Rkyv($func(e)),
        }
    };
}

impl XError {
    pub fn pos(&self) -> &'static str {
        switch_call!(&self, |e: &MixedError<_>| e.pos())
    }

    pub fn msg<'t>(&'t self) -> &'t str {
        switch_call!(&self, |e: &'t MixedError<_>| e.msg())
    }

    pub(crate) fn set_pos(self, pos: &'static &'static str) -> XError {
        switch_new!(self, |e: MixedError<_>| e.set_pos(pos))
    }

    pub(crate) fn set_msg(self, msg: String) -> XError {
        switch_new!(self, |e: MixedError<_>| e.set_msg(msg))
    }
}

impl fmt::Display for XError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XError::Unexpected(e) => e.to_string(f, "Unexpected"),
            XError::BadArgument(e) => e.to_string(f, "BadArgument"),
            XError::NotFound(e) => e.to_string(f, "NotFound"),
            XError::BadType(e) => e.to_string(f, "BadType"),
            XError::BadOperation(e) => e.to_string(f, "BadOperation"),
            XError::Overflow(e) => e.to_string(f, "Overflow"),
            XError::SymbolTooLong(e) => e.to_string(f, "SymbolTooLong"),
            XError::SymbolNotFound(e) => e.to_string(f, "SymbolNotFound"),
            XError::SymbolNotPreloaded(e) => e.to_string(f, "SymbolNotPreloaded"),
            XError::BadParameter(e) => e.to_string(f, "BadParameter"),
            XError::BadAttribute(e) => e.to_string(f, "BadAttribute"),
            XError::BadAsset(e) => e.to_string(f, "BadAsset"),
            XError::BadScript(e) => e.to_string(f, "BadScript"),
            XError::BadAction(e) => e.to_string(f, "BadAction"),
            XError::ScriptNoHook(e) => e.to_string(f, "ScriptNoHook"),
            XError::ScriptOutOfRange(e) => e.to_string(f, "ScriptOutOfRange"),
            XError::ScriptBadCommand(e) => e.to_string(f, "ScriptBadCommand"),
            XError::ScriptStackOverflow(e) => e.to_string(f, "ScriptStackOverflow"),
            XError::TmplNotFound(e) => e.to_string(f, "TmplNotFound"),
            XError::LogicNotFound(e) => e.to_string(f, "LogicNotFound"),
            XError::LogicBadState(e) => e.to_string(f, "LogicBadState"),
            XError::LogicIDMismatch(e) => e.to_string(f, "LogicIDMismatch"),
            XError::IO(e) => e.to_string(f, "IO"),
            XError::Utf8(e) => e.to_string(f, "Utf8"),
            XError::Json(e) => e.to_string(f, "Json"),
            XError::Zip(e) => e.to_string(f, "Zip"),
            XError::Jolt(e) => e.to_string(f, "Jolt"),
            XError::Ozz(e) => e.to_string(f, "Ozz"),
            XError::Rkyv(e) => e.to_string(f, "Rkyv"),
        }
    }
}

impl From<std::io::Error> for XError {
    fn from(e: std::io::Error) -> Self {
        XError::IO(MixedError::from_static(e, &EMPTY_ERROR))
    }
}

impl From<std::str::Utf8Error> for XError {
    fn from(err: std::str::Utf8Error) -> Self {
        XError::Utf8(MixedError::from_static(err, &EMPTY_ERROR))
    }
}

impl From<serde_json::Error> for XError {
    fn from(e: serde_json::Error) -> Self {
        XError::Json(MixedError::from_static(e, &EMPTY_ERROR))
    }
}

impl From<zip::result::ZipError> for XError {
    fn from(e: zip::result::ZipError) -> Self {
        let io_err: std::io::Error = e.into();
        XError::Zip(MixedError::from_static(io_err, &EMPTY_ERROR))
    }
}

impl From<JoltError> for XError {
    fn from(e: JoltError) -> Self {
        XError::Jolt(MixedError::from_static(e, &EMPTY_ERROR))
    }
}

impl From<OzzError> for XError {
    fn from(e: OzzError) -> Self {
        XError::Ozz(MixedError::from_static(e, &EMPTY_ERROR))
    }
}

pub type XResult<T> = Result<T, XError>;

macro_rules! xpos {
    () => {
        &const_format::formatcp!("{}:{}", file!(), line!())
    };
    ($extra:expr) => {
        &const_format::formatcp!("{}:{}({})", file!(), line!(), $extra)
    };
}
pub(crate) use xpos;

macro_rules! xerr {
    ($variant:ident) => {
        crate::utils::XError::$variant(crate::utils::MixedError::from_static(
            (),
            &const_format::formatcp!("{}:{}", file!(), line!()),
        ))
    };
    ($variant:ident, $source:expr) => {
        crate::utils::XError::$variant(crate::utils::MixedError::from_static(
            $source,
            &const_format::formatcp!("{}:{}", file!(), line!()),
        ))
    };
    ($variant:ident; $extra:expr) => {
        crate::utils::XError::$variant(crate::utils::MixedError::from_static(
            (),
            &const_format::formatcp!("{}:{} ({})", file!(), line!(), $extra),
        ))
    };
    ($variant:ident, $source:expr; $extra:expr) => {
        crate::utils::XError::$variant(crate::utils::MixedError::from_static(
            $source,
            &const_format::formatcp!("{}:{} ({})", file!(), line!(), $extra),
        ))
    };
}
pub(crate) use xerr;

macro_rules! xerrf {
    ($variant:ident; $($args:tt)*) => {
        crate::utils::XError::$variant(
            crate::utils::MixedError::from_static(
                (),
                &const_format::formatcp!("{}:{}", file!(), line!())
            )
        ).set_msg(format!($($args)*))
    };
    ($variant:ident, $source:expr; $($args:tt)*) => {
        crate::utils::XError::$variant(
            crate::utils::MixedError::from_static(
                $source,
                &const_format::formatcp!("{}:{}", file!(), line!())
            )
        ).set_msg(format!($($args)*))
    };
}
pub(crate) use xerrf;

macro_rules! xres {
    ($variant:ident) => {
        Err(crate::utils::xerr!($variant))
    };
    ($variant:ident, $source:expr) => {
        Err(crate::utils::xerr!($variant, $source))
    };
    ($variant:ident; $extra:expr) => {
        Err(crate::utils::xerr!($variant; $extra))
    };
    ($variant:ident, $source:expr; $extra:expr) => {
        Err(crate::utils::xerr!($variant, $source; $extra))
    };
}
pub(crate) use xres;

macro_rules! xresf {
    ($variant:ident; $($args:tt)*) => {
        Err(crate::utils::xerrf!($variant; $($args)*))
    };
    ($variant:ident, $source:expr; $($args:tt)*) => {
        Err(crate::utils::xerrf!($variant, $source; $($args)*))
    };
}
pub(crate) use xresf;

macro_rules! xfrom {
    () => {
        |e| crate::utils::XError::from(e).set_pos(crate::utils::xpos!())
    };
    ($extra:expr) => {
        |e| crate::utils::XError::from(e).set_pos(crate::utils::xpos!($extra))
    };
}
pub(crate) use xfrom;

macro_rules! xfromf {
    ($($args:tt)*) => {
        |e| crate::utils::XError::from(e)
            .set_pos(crate::utils::xpos!())
            .set_msg(format!($($args)*))
    };
}
pub(crate) use xfromf;

#[macro_export]
macro_rules! xerror {
    ($variant:ident, $msg:expr) => {
        cirtical_point_core::utils::XError::$variant(cirtical_point_core::utils::MixedError::from_dynamic(
            (),
            &const_format::formatcp!("{}:{}", file!(), line!()),
            $msg.to_string(),
        ))
    };
}
pub use xerror;
