use ozz_animation_rs::OzzError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XError {
    #[error("Unexpected error")]
    Unexpected,

    #[error("Bad argument")]
    BadArgument,
    #[error("Not found")]
    NotFound(Box<String>),
    #[error("Bad type")]
    BadType,

    #[error("Symbol too long")]
    SymbolTooLong,
    #[error("Symbol not found")]
    SymbolNotFound,
    #[error("Symbol not preloaded")]
    SymbolNotPreloaded,

    #[error("Bad parameter: {0}")]
    BadParameter(Box<String>),
    #[error("Bad attribute: {0}")]
    BadAttribute(Box<String>),
    #[error("Bad script: {0}")]
    BadScript(Box<String>),
    #[error("Bad action: {0}")]
    BadAction(Box<String>),

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

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JSON(#[from] serde_json::Error),
    #[error("Ozz {0}")]
    Ozz(#[from] OzzError),
}

impl XError {
    pub fn not_found<S: ToString>(s: S) -> XError {
        return XError::NotFound(Box::new(s.to_string()));
    }

    pub fn bad_parameter<S: ToString>(s: S) -> XError {
        return XError::BadParameter(Box::new(s.to_string()));
    }

    pub fn bad_attribute<S: ToString>(s: S) -> XError {
        return XError::BadAttribute(Box::new(s.to_string()));
    }

    pub fn bad_script<S: ToString>(s: S) -> XError {
        return XError::BadScript(Box::new(s.to_string()));
    }

    pub fn bad_action<S: ToString>(s: S) -> XError {
        return XError::BadAction(Box::new(s.to_string()));
    }
}

pub type XResult<T> = Result<T, XError>;
