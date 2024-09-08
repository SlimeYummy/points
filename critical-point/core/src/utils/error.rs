use ozz_animation_rs::OzzError;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XError {
    #[error("Unexpected error {0}")]
    Unexpected(XMessage),

    #[error("Bad argument {0}")]
    BadArgument(XMessage),
    #[error("Not found {0}")]
    NotFound(XMessage),
    #[error("Bad type")]
    BadType,
    #[error("Invalid operation {0}")]
    InvalidOperation(XMessage),
    #[error("Overflow {0}")]
    Overflow(XMessage),
    #[error("ID miss match")]
    IDMissMatch,

    #[error("Symbol too long")]
    SymbolTooLong,
    #[error("Symbol not found")]
    SymbolNotFound,
    #[error("Symbol not preloaded")]
    SymbolNotPreloaded,

    #[error("Bad parameter {0}")]
    BadParameter(XMessage),
    #[error("Bad attribute {0}")]
    BadAttribute(XMessage),
    #[error("Bad template {0}")]
    BadTemplate(XMessage),
    #[error("Bad script {0}")]
    BadScript(XMessage),
    #[error("Bad action {0}")]
    BadAction(XMessage),

    #[error("Script hook not found")]
    ScriptNoHook,
    #[error("Script out of range")]
    ScriptOutOfRange,
    #[error("Script bad command")]
    ScriptBadCommand,
    #[error("Script stack overflow")]
    ScriptStackOverflow,

    #[error("Physic body failed")]
    PhysicBodyFailed,
    #[error("Physic shape not found")]
    PhysicShapeNotFound,

    #[error("IO error {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON error {0}")]
    JSON(#[from] serde_json::Error),
    #[error("Ozz {0}")]
    Ozz(#[from] OzzError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum XMessage {
    None,
    Str(Box<String>),
}

impl XMessage {
    #[inline]
    pub fn new<S: ToString>(s: S) -> XMessage {
        let string = s.to_string();
        if string.is_empty() {
            return XMessage::None;
        }
        XMessage::Str(Box::new(string))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        return match self {
            XMessage::None => "",
            XMessage::Str(s) => s.as_str(),
        };
    }
}

impl Default for XMessage {
    #[inline]
    fn default() -> Self {
        XMessage::None
    }
}

impl fmt::Display for XMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XMessage::None => f.write_str("(...)"),
            XMessage::Str(s) => write!(f, "({})", s),
        }
    }
}

macro_rules! constructor {
    ($func:ident, $enum:path) => {
        #[inline]
        pub fn $func<S: ToString>(s: S) -> XError {
            return $enum(XMessage::new(s));
        }
    };
}

impl XError {
    constructor!(unexpected, XError::Unexpected);
    constructor!(bad_argument, XError::BadArgument);
    constructor!(invalid_operation, XError::InvalidOperation);
    constructor!(overflow, XError::Overflow);
    constructor!(not_found, XError::NotFound);
    constructor!(bad_parameter, XError::BadParameter);
    constructor!(bad_attribute, XError::BadAttribute);
    constructor!(bad_template, XError::BadTemplate);
    constructor!(bad_script, XError::BadScript);
    constructor!(bad_action, XError::BadAction);
}

pub type XResult<T> = Result<T, XError>;
