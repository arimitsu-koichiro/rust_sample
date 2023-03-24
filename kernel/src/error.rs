use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};
use strum::{Display, EnumString};

#[derive(Clone, Debug, Serialize)]
pub enum Error {
    BadRequest(Codes, Option<String>),
    Unauthorized(Codes, Option<String>),
    Forbidden(Codes, Option<String>),
    NotFound(Codes, Option<String>),
    Unexpected(Codes, Option<String>),
}

impl Error {
    pub fn bad_request(codes: Codes, s: impl Into<String>) -> Error {
        Error::BadRequest(codes, Some(s.into()))
    }
    pub fn unauthorized(codes: Codes, s: impl Into<String>) -> Error {
        Error::Unauthorized(codes, Some(s.into()))
    }
    pub fn forbidden(codes: Codes, s: impl Into<String>) -> Error {
        Error::Forbidden(codes, Some(s.into()))
    }
    pub fn not_found(codes: Codes, s: impl Into<String>) -> Error {
        Error::NotFound(codes, Some(s.into()))
    }
    pub fn unexpected(codes: Codes, s: impl Into<String>) -> Error {
        Error::Unexpected(codes, Some(s.into()))
    }
    #[must_use]
    pub fn with_codes(&self, codes: Codes) -> Error {
        match self {
            Error::BadRequest(_, detail) => Error::BadRequest(codes, detail.clone()),
            Error::Unauthorized(_, detail) => Error::Unauthorized(codes, detail.clone()),
            Error::Forbidden(_, detail) => Error::Forbidden(codes, detail.clone()),
            Error::NotFound(_, detail) => Error::NotFound(codes, detail.clone()),
            Error::Unexpected(_, detail) => Error::Unexpected(codes, detail.clone()),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadRequest(msg, detail) => {
                write!(f, "{msg:?}, {detail:?}")
            }
            Error::Unauthorized(msg, detail) => {
                write!(f, "{msg:?}, {detail:?}")
            }
            Error::Forbidden(msg, detail) => {
                write!(f, "{msg:?}, {detail:?}")
            }
            Error::NotFound(msg, detail) => write!(f, "{msg:?}, {detail:?}"),
            Error::Unexpected(msg, detail) => {
                write!(f, "{msg:?}, {detail:?}")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Serialize, Debug, Clone, EnumString, Display)]
pub enum Codes {
    // common error
    #[strum(to_string = "common/bad_request")]
    CommonBadRequest,
    #[strum(to_string = "common/unauthorized")]
    CommonUnauthorized,
    #[strum(to_string = "common/forbidden")]
    CommonForbidden,
    #[strum(to_string = "common/not_found")]
    CommonNotFound,
    #[strum(to_string = "common/unexpected")]
    CommonUnexpected,
    // special error
    #[strum(to_string = "auth/invalid_session")]
    InvalidSession,
    #[strum(to_string = "auth/invalid_email_or_password")]
    InvalidEmailOrPassword,
}

#[macro_export]
macro_rules! bad_request {
    ($msg:literal $(,)?) => {
        $crate::error::Error::bad_request($crate::error::Codes::CommonBadRequest, $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::bad_request(format!($crate::error::Codes::CommonBadRequest, $fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! unauthorized {
    ($msg:literal $(,)?) => {
        $crate::error::Error::unauthorized($crate::error::Codes::CommonUnauthorized, $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::unauthorized($crate::error::Codes::CommonUnauthorized, format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! forbidden {
    ($msg:literal $(,)?) => {
        $crate::error::Error::forbidden($crate::error::Codes::CommonForbidden, $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::forbidden($crate::error::Codes::CommonForbidden, format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! not_found {
    ($msg:literal $(,)?) => {
        $crate::error::Error::not_found($crate::error::Codes::CommonNotFound, $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::not_found($crate::error::Codes::CommonNotFound, format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! unexpected {
    ($msg:literal $(,)?) => {
        $crate::error::Error::unexpected($crate::error::Codes::CommonUnexpected, $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::unexpected($crate::error::Codes::CommonUnexpected, format!($fmt, $($arg)*))
    };
}
