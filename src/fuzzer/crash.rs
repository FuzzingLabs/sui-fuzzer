use serde::{Serialize, Deserialize};

use crate::mutator::types::Type;

use super::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Crash {
    pub target_function: String,
    pub inputs: Vec<Type>,
    pub error: Error
}

impl Crash {

    pub fn new(target_function: &str, inputs: &Vec<Type>, error: &Error) -> Self {
        Self {
            target_function: target_function.to_string(),
            inputs: inputs.clone(),
            error: error.clone()
        }
    }

}