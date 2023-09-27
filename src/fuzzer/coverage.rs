use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Coverage {
    pub input: Vec<u8>,
    pub data: Vec<CoverageData>
}

#[derive(Hash, Eq, PartialEq, Debug)]
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
