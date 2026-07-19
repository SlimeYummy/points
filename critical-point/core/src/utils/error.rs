use jolt_physics_rs::JoltError;
use ozz_animation_rs::OzzError;
use recastnavigation_rs::RNError;
use std::error::Error;
use std::fmt;

const EMPTY_STR: &'static str = "";

#[derive(Debug)]
pub(crate) struct StaticError<T> {
    src: T, // source
    pos: &'static &'static str,
}

#[derive(Debug)]
pub(crate) struct DynamicError<T> {
    src: T, // source
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

    fn pos(&self) -> &'static str {
        match self {
            MixedError::Static(e) => *e.pos,
            MixedError::Dynamic(e) => *e.pos,
        }
    }

    fn msg(&self) -> &str {
        match self {
            MixedError::Static(_) => EMPTY_STR,
            MixedError::Dynamic(e) => e.msg.as_str(),
        }
    }

    fn src(&self) -> &T {
        match self {
            MixedError::Static(e) => &e.src,
            MixedError::Dynamic(e) => &e.src,
        }
    }

    fn set_pos(mut self, pos: &'static &'static str) -> Self {
        match self {
            MixedError::Static(ref mut e) => e.pos = pos,
            MixedError::Dynamic(ref mut e) => e.pos = pos,
        }
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

fn fmt_pos(s: &str) -> &str {
    match s {
        "" => "-",
        _ => s,
    }
}

trait ToString {
    fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result;
}

impl ToString for MixedError<()> {
    fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
        match self {
            MixedError::Static(e) => write!(f, "{}  [P]: {}", name, fmt_pos(e.pos)),
            MixedError::Dynamic(e) => write!(f, "{}  [P]: {}  [M]: {}", name, fmt_pos(e.pos), e.msg),
        }
    }
}

// macro_rules! val_to_string {
//     ($typ:path) => {
//         impl ToString for MixedError<$typ> {
//             fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
//                 match self {
//                     MixedError::Static(e) => write!(f, "{}({})  [P]: {}", name, e.src, fmt_pos(e.pos)),
//                     MixedError::Dynamic(e) => write!(f, "{}({})  [P]: {}  [M]: {}", name, e.src, fmt_pos(e.pos), e.msg),
//                 }
//             }
//         }
//     };
// }

// val_to_string!(NumID);
// val_to_string!(TmplID);

macro_rules! err_to_string {
    ($err:path, $err_name:expr) => {
        impl ToString for MixedError<$err> {
            fn to_string(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
                match self {
                    MixedError::Static(e) => write!(
                        f,
                        "{}({})  [P]: {}  [S]: {}",
                        name,
                        $err_name,
                        fmt_pos(e.pos),
                        e.src
                    ),
                    MixedError::Dynamic(e) => {
                        write!(
                            f,
                            "{}({})  [P]: {}  [M]: {}  [S]: {}",
                            name,
                            $err_name,
                            fmt_pos(e.pos),
                            e.msg,
                            e.src
                        )
                    }
                }
            }
        }
    };
}

err_to_string!(std::io::Error, "io::Error");
err_to_string!(std::str::Utf8Error, "str::Utf8Error");
err_to_string!(serde_json::Error, "serde_json::Error");
err_to_string!(wasmtime::Error, "wasmtime::Error");
err_to_string!(JoltError, "JoltError");
err_to_string!(OzzError, "OzzError");
err_to_string!(RNError, "RNError");

#[derive(Debug)]
pub enum XError {
    Unexpected(MixedError<()>),
    OutOfMemory(MixedError<()>),
    Overflow(MixedError<()>),
    NotFound(MixedError<()>),

    BadArgument(MixedError<()>),
    BadType(MixedError<()>),
    BadOperation(MixedError<()>),

    UninitedTmplID(MixedError<()>),
    InvalidTmplID(MixedError<()>),
    InvalidSymbol(MixedError<()>),

    BadParameter(MixedError<()>),
    BadAttribute(MixedError<()>),
    BadAsset(MixedError<()>),
    BadAction(MixedError<()>),

    AssetNotFound(MixedError<()>),
    TmplNotFound(MixedError<()>),
    InstNotFound(MixedError<()>),

    LogicNotFound(MixedError<()>),
    LogicBadState(MixedError<()>),
    LogicIDMismatch(MixedError<()>),
    LogicException(MixedError<()>),

    IO(MixedError<std::io::Error>),
    Utf8(MixedError<std::str::Utf8Error>),
    Json(MixedError<serde_json::Error>),
    Zip(MixedError<std::io::Error>),
    Wasmtime(MixedError<wasmtime::Error>),
    Jolt(MixedError<JoltError>),
    Ozz(MixedError<OzzError>),
    RcNav(MixedError<RNError>),
    Rkyv(MixedError<()>), // no source here

    Script(MixedError<()>),
    Custom(MixedError<()>),
}

impl Error for XError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            XError::IO(e) => Some(e.src()),
            XError::Utf8(e) => Some(e.src()),
            XError::Json(e) => Some(e.src()),
            XError::Zip(e) => Some(e.src()),
            XError::Wasmtime(e) => Some(e.src().as_ref()),
            XError::Jolt(e) => Some(e.src()),
            XError::Ozz(e) => Some(e.src()),
            XError::RcNav(e) => Some(e.src()),
            _ => None,
        }
    }
}

macro_rules! switch_call {
    ($self:expr, $func:expr) => {
        match $self {
            XError::Unexpected(e) => $func(e),
            XError::OutOfMemory(e) => $func(e),
            XError::Overflow(e) => $func(e),
            XError::NotFound(e) => $func(e),
            XError::BadArgument(e) => $func(e),
            XError::BadType(e) => $func(e),
            XError::BadOperation(e) => $func(e),
            XError::UninitedTmplID(e) => $func(e),
            XError::InvalidTmplID(e) => $func(e),
            XError::InvalidSymbol(e) => $func(e),
            XError::BadParameter(e) => $func(e),
            XError::BadAttribute(e) => $func(e),
            XError::BadAsset(e) => $func(e),
            XError::BadAction(e) => $func(e),
            XError::AssetNotFound(e) => $func(e),
            XError::TmplNotFound(e) => $func(e),
            XError::InstNotFound(e) => $func(e),
            XError::LogicNotFound(e) => $func(e),
            XError::LogicBadState(e) => $func(e),
            XError::LogicIDMismatch(e) => $func(e),
            XError::LogicException(e) => $func(e),
            XError::IO(e) => $func(e),
            XError::Utf8(e) => $func(e),
            XError::Json(e) => $func(e),
            XError::Zip(e) => $func(e),
            XError::Wasmtime(e) => $func(e),
            XError::Jolt(e) => $func(e),
            XError::Ozz(e) => $func(e),
            XError::RcNav(e) => $func(e),
            XError::Rkyv(e) => $func(e),
            XError::Script(e) => $func(e),
            XError::Custom(e) => $func(e),
        }
    };
}

macro_rules! switch_new {
    ($self:expr, $func:expr) => {
        match $self {
            XError::Unexpected(e) => XError::Unexpected($func(e)),
            XError::OutOfMemory(e) => XError::OutOfMemory($func(e)),
            XError::Overflow(e) => XError::Overflow($func(e)),
            XError::NotFound(e) => XError::NotFound($func(e)),
            XError::BadArgument(e) => XError::BadArgument($func(e)),
            XError::BadType(e) => XError::BadType($func(e)),
            XError::BadOperation(e) => XError::BadOperation($func(e)),
            XError::UninitedTmplID(e) => XError::UninitedTmplID($func(e)),
            XError::InvalidTmplID(e) => XError::InvalidTmplID($func(e)),
            XError::InvalidSymbol(e) => XError::InvalidSymbol($func(e)),
            XError::BadParameter(e) => XError::BadParameter($func(e)),
            XError::BadAttribute(e) => XError::BadAttribute($func(e)),
            XError::BadAsset(e) => XError::BadAsset($func(e)),
            XError::BadAction(e) => XError::BadAction($func(e)),
            XError::AssetNotFound(e) => XError::AssetNotFound($func(e)),
            XError::TmplNotFound(e) => XError::TmplNotFound($func(e)),
            XError::InstNotFound(e) => XError::InstNotFound($func(e)),
            XError::LogicNotFound(e) => XError::LogicNotFound($func(e)),
            XError::LogicBadState(e) => XError::LogicBadState($func(e)),
            XError::LogicIDMismatch(e) => XError::LogicIDMismatch($func(e)),
            XError::LogicException(e) => XError::LogicException($func(e)),
            XError::IO(e) => XError::IO($func(e)),
            XError::Utf8(e) => XError::Utf8($func(e)),
            XError::Json(e) => XError::Json($func(e)),
            XError::Zip(e) => XError::Zip($func(e)),
            XError::Wasmtime(e) => XError::Wasmtime($func(e)),
            XError::Jolt(e) => XError::Jolt($func(e)),
            XError::Ozz(e) => XError::Ozz($func(e)),
            XError::RcNav(e) => XError::RcNav($func(e)),
            XError::Rkyv(e) => XError::Rkyv($func(e)),
            XError::Script(e) => XError::Script($func(e)),
            XError::Custom(e) => XError::Custom($func(e)),
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

    pub fn set_pos(self, pos: &'static &'static str) -> XError {
        switch_new!(self, |e: MixedError<_>| e.set_pos(pos))
    }

    pub fn set_msg(self, msg: String) -> XError {
        switch_new!(self, |e: MixedError<_>| e.set_msg(msg))
    }
}

impl fmt::Display for XError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XError::Unexpected(e) => e.to_string(f, "Unexpected"),
            XError::OutOfMemory(e) => e.to_string(f, "OutOfMemory"),
            XError::Overflow(e) => e.to_string(f, "Overflow"),
            XError::NotFound(e) => e.to_string(f, "NotFound"),
            XError::BadArgument(e) => e.to_string(f, "BadArgument"),
            XError::BadType(e) => e.to_string(f, "BadType"),
            XError::BadOperation(e) => e.to_string(f, "BadOperation"),
            XError::UninitedTmplID(e) => e.to_string(f, "UninitedTmplID"),
            XError::InvalidTmplID(e) => e.to_string(f, "InvalidTmplID"),
            XError::InvalidSymbol(e) => e.to_string(f, "InvalidSymbol"),
            XError::BadParameter(e) => e.to_string(f, "BadParameter"),
            XError::BadAttribute(e) => e.to_string(f, "BadAttribute"),
            XError::BadAsset(e) => e.to_string(f, "BadAsset"),
            XError::BadAction(e) => e.to_string(f, "BadAction"),
            XError::AssetNotFound(e) => e.to_string(f, "AssetNotFound"),
            XError::TmplNotFound(e) => e.to_string(f, "TmplNotFound"),
            XError::InstNotFound(e) => e.to_string(f, "InstNotFound"),
            XError::LogicNotFound(e) => e.to_string(f, "LogicNotFound"),
            XError::LogicBadState(e) => e.to_string(f, "LogicBadState"),
            XError::LogicIDMismatch(e) => e.to_string(f, "LogicIDMismatch"),
            XError::LogicException(e) => e.to_string(f, "LogicException"),
            XError::IO(e) => e.to_string(f, "IO"),
            XError::Utf8(e) => e.to_string(f, "Utf8"),
            XError::Json(e) => e.to_string(f, "Json"),
            XError::Zip(e) => e.to_string(f, "Zip"),
            XError::Wasmtime(e) => e.to_string(f, "Wasmtime"),
            XError::Jolt(e) => e.to_string(f, "Jolt"),
            XError::Ozz(e) => e.to_string(f, "Ozz"),
            XError::RcNav(e) => e.to_string(f, "RcNav"),
            XError::Rkyv(e) => e.to_string(f, "Rkyv"),
            XError::Script(e) => e.to_string(f, "Script"),
            XError::Custom(e) => e.to_string(f, "Custom"),
        }
    }
}

impl From<std::io::Error> for XError {
    fn from(e: std::io::Error) -> Self {
        XError::IO(MixedError::from_static(e, &EMPTY_STR))
    }
}

impl From<std::str::Utf8Error> for XError {
    fn from(err: std::str::Utf8Error) -> Self {
        XError::Utf8(MixedError::from_static(err, &EMPTY_STR))
    }
}

impl From<serde_json::Error> for XError {
    fn from(e: serde_json::Error) -> Self {
        XError::Json(MixedError::from_static(e, &EMPTY_STR))
    }
}

impl From<zip::result::ZipError> for XError {
    fn from(e: zip::result::ZipError) -> Self {
        let io_err: std::io::Error = e.into();
        XError::Zip(MixedError::from_static(io_err, &EMPTY_STR))
    }
}

impl From<wasmtime::Error> for XError {
    fn from(e: wasmtime::Error) -> Self {
        XError::Wasmtime(MixedError::from_static(e, &EMPTY_STR))
    }
}

impl From<JoltError> for XError {
    fn from(e: JoltError) -> Self {
        XError::Jolt(MixedError::from_static(e, &EMPTY_STR))
    }
}

impl From<OzzError> for XError {
    fn from(e: OzzError) -> Self {
        XError::Ozz(MixedError::from_static(e, &EMPTY_STR))
    }
}

impl From<RNError> for XError {
    fn from(e: RNError) -> Self {
        XError::RcNav(MixedError::from_static(e, &EMPTY_STR))
    }
}

impl From<String> for XError {
    fn from(s: String) -> Self {
        XError::Custom(MixedError::from_dynamic((), &EMPTY_STR, s))
    }
}

impl From<&str> for XError {
    fn from(s: &str) -> Self {
        XError::Custom(MixedError::from_dynamic((), &EMPTY_STR, s.to_string()))
    }
}

pub type XResult<T> = Result<T, XError>;

#[macro_export]
macro_rules! xpos {
    () => {
        &const_format::formatcp!("{}:{}", file!(), line!())
    };
    ($extra:expr) => {
        &const_format::formatcp!("{}:{}({})", file!(), line!(), $extra)
    };
}
pub use xpos;

// TODO: refactor error formats !!!!!!!!!!

#[macro_export]
macro_rules! xerr {
    ($variant:ident) => {
        $crate::utils::XError::$variant($crate::utils::MixedError::from_static(
            (),
            &const_format::formatcp!("{}:{}", file!(), line!()),
        ))
    };
    ($variant:ident, $payload:expr) => {
        $crate::utils::XError::$variant($crate::utils::MixedError::from_static(
            $payload,
            &const_format::formatcp!("{}:{}", file!(), line!()),
        ))
    };
    ($variant:ident; $extra:expr) => {
        $crate::utils::XError::$variant($crate::utils::MixedError::from_static(
            (),
            &const_format::formatcp!("{}:{} ({})", file!(), line!(), $extra),
        ))
    };
    ($variant:ident, $payload:expr; $extra:expr) => {
        $crate::utils::XError::$variant($crate::utils::MixedError::from_static(
            $payload,
            &const_format::formatcp!("{}:{} ({})", file!(), line!(), $extra),
        ))
    };
}
pub use xerr;

#[macro_export]
macro_rules! xerrf {
    ($variant:ident; $($args:tt)*) => {
        $crate::utils::XError::$variant(
            $crate::utils::MixedError::from_static(
                (),
                &const_format::formatcp!("{}:{}", file!(), line!())
            )
        ).set_msg(format!($($args)*))
    };
    ($variant:ident, $payload:expr; $($args:tt)*) => {
        $crate::utils::XError::$variant(
            $crate::utils::MixedError::from_static(
                $payload,
                &const_format::formatcp!("{}:{}", file!(), line!())
            )
        ).set_msg(format!($($args)*))
    };
}
pub use xerrf;

#[macro_export]
macro_rules! xres {
    ($variant:ident) => {
        Err($crate::utils::xerr!($variant))
    };
    ($variant:ident, $payload:expr) => {
        Err($crate::utils::xerr!($variant, $payload))
    };
    ($variant:ident; $extra:expr) => {
        Err($crate::utils::xerr!($variant; $extra))
    };
    ($variant:ident, $payload:expr; $extra:expr) => {
        Err($crate::utils::xerr!($variant, $payload; $extra))
    };
}
pub use xres;

#[macro_export]
macro_rules! xresf {
    ($variant:ident; $($args:tt)*) => {
        Err($crate::utils::xerrf!($variant; $($args)*))
    };
    ($variant:ident, $payload:expr; $($args:tt)*) => {
        Err($crate::utils::xerrf!($variant, $payload; $($args)*))
    };
}
pub use xresf;

#[macro_export]
macro_rules! xfrom {
    () => {
        |e| $crate::utils::XError::from(e).set_pos(&const_format::formatcp!("{}:{}", file!(), line!()))
    };
    ($extra:expr) => {
        |e| $crate::utils::XError::from(e).set_pos(&const_format::formatcp!("{}:{}({})"))
    };
}
pub use xfrom;

#[macro_export]
macro_rules! xfromf {
    ($($args:tt)*) => {
        |e| $crate::utils::XError::from(e)
            .set_pos(&const_format::formatcp!("{}:{}", file!(), line!()))
            .set_msg(format!($($args)*))
    };
}
pub use xfromf;
