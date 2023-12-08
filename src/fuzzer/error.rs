use serde::{Serialize, Deserialize};
use strum::{EnumVariantNames, Display};

#[derive(Debug, Clone, Display ,EnumVariantNames, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum Error {
    Abort { message: String },
    OutOfBound { message: String },
    OutOfGas { message: String },
    ArithmeticError {message: String},
    MemoryLimitExceeded { message: String },
    Unknown { message: String },
    // TODO Add more errors
}
