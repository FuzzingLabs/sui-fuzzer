use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::EnumVariantNames;

#[derive(Debug, Clone, EnumVariantNames, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Error {
    Abort { message: String },
    Runtime { message: String },
    OutOfBound { message: String },
    OutOfGas { message: String },
    ArithmeticError { message: String },
    MemoryLimitExceeded { message: String },
    Unknown { message: String },
    // TODO Add more errors
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Abort { message  } => write!(f, "Abort - {}", message),
            Error::OutOfBound { message: _ } => write!(f, "OutOfBound"),
            Error::OutOfGas { message: _ } => write!(f, "OutOfGas"),
            Error::ArithmeticError { message: _ } => write!(f, "ArithmeticError"),
            Error::MemoryLimitExceeded { message: _ } => write!(f, "MemoryLimitExceeded"),
            Error::Unknown { message } => write!(f, "Unknown - {}", message),
            Error::Runtime { message } => write!(f, "Runtime - {}", message),
        }
    }
}
