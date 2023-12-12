use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::mutator::types::Type;

use super::error::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct Crash {
    pub target_function: String,
    pub inputs: Vec<Type>,
    pub error: Error,
}

impl Crash {
    pub fn new(target_function: &str, inputs: &Vec<Type>, error: &Error) -> Self {
        Self {
            target_function: target_function.to_string(),
            inputs: inputs.clone(),
            error: error.clone(),
        }
    }
}

impl Hash for Crash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.error.hash(state);
        self.target_function.hash(state);
    }
}

impl PartialEq for Crash {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error && self.target_function == other.target_function
    }
}

impl Eq for Crash {}
