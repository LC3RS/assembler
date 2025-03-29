use std::{fmt, io, num, result::Result as StdResult};

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    IOError,
    ParseConstantError,
    ParseOpCodeError,
    ParseRegisterError,
    ParseDirectiveError,
    InvalidTokenError,
    MissingLabelError,
    UnexpectedEof,
    SyntaxError,
    JibbyError,
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: kind.as_str().to_owned(),
        }
    }
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::IOError => "io error",
            ErrorKind::ParseConstantError => "parse constant error",
            ErrorKind::ParseOpCodeError => "parse op code error",
            ErrorKind::ParseRegisterError => "parse register error",
            ErrorKind::ParseDirectiveError => "parse directive error",
            ErrorKind::InvalidTokenError => "encountered invalid token while parsing",
            ErrorKind::UnexpectedEof => "unexpectedly reached EOF",
            ErrorKind::SyntaxError => "invalid syntax",
            ErrorKind::MissingLabelError => "missing label",
            ErrorKind::JibbyError => "invalid value",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self {
            kind: ErrorKind::IOError,
            message: error.to_string(),
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(error: num::ParseIntError) -> Self {
        Self {
            kind: ErrorKind::ParseConstantError,
            message: error.to_string(),
        }
    }
}

pub type Result<T, E = Error> = StdResult<T, E>;
