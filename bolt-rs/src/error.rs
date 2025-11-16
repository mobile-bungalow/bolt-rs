use std::ffi::NulError;

use thiserror::Error;

use crate::types::value::ValueType;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "Could not convert argument to CString - make sure not to pass strings with `nul` characters."
    )]
    StringConversion(#[from] NulError),
    #[error("{msg}")]
    BoltError { msg: String },
}

impl Error {
    pub fn bolt(msg: &str) -> Self {
        Self::BoltError {
            msg: msg.to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum ArgError {
    TypeGuard {
        expected: ValueType,
        actual: ValueType,
    },
    TypeGuardEnum {
        actual: ValueType,
    },
    IndexOutOfBounds {
        idx: u8,
        len: u8,
    },
}

#[derive(Debug)]
pub enum ModuleError {
    InvalidName(String),
    AlreadyRegistered(String),
    NotFound(String),
}
