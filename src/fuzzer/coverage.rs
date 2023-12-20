use std::hash::{Hash, Hasher};

use serde::{Serialize, Deserialize};

use crate::mutator::types::Type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub inputs: Vec<Type>,
    pub data: Vec<CoverageData>
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct CoverageData {
    pub pc: u64
}

impl Hash for Coverage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl PartialEq for Coverage {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Coverage {}
