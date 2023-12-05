use crate::mutator::mutator::Mutator;
use basic_mutator::{self, EmptyDatabase};

use super::types::Type;

pub struct SuiMutator {
    mutator: basic_mutator::Mutator,
}

impl SuiMutator {
    pub fn new(seed: u64, max_input_size: usize) -> Self {
        let mutator = basic_mutator::Mutator::new()
            .seed(seed)
            .max_input_size(max_input_size)
            .printable(true);
        SuiMutator { mutator }
    }
}

impl Mutator for SuiMutator {
    fn mutate(&mut self, inputs: &Vec<Type>, nb_mutation: usize) -> Vec<Type> {
        let mut res = vec![];

        for input in inputs {
            self.mutator.input.clear();
            match input {
                Type::U8(v) => self.mutator.input.extend_from_slice(&v.to_le_bytes()),
                Type::U16(v) => self.mutator.input.extend_from_slice(&v.to_le_bytes()),
                Type::U32(v) => self.mutator.input.extend_from_slice(&v.to_le_bytes()),
                Type::U64(v) => self.mutator.input.extend_from_slice(&v.to_le_bytes()),
                Type::U128(v) => self.mutator.input.extend_from_slice(&v.to_le_bytes()),
                Type::Bool(b) => self
                    .mutator
                    .input
                    .extend_from_slice(&[if *b { 1 } else { 0 }]),
                Type::Vector(_, _) => todo!(),
            }

            self.mutator.mutate(nb_mutation, &EmptyDatabase);

            res.push(match input {
                Type::U8(_) => Type::U8(u8::from_le_bytes(self.mutator.input.try_into().unwrap())),
                Type::U16(_) => {
                    Type::U16(u16::from_be_bytes(self.mutator.input.try_into().unwrap()))
                }
                Type::U32(_) => {
                    Type::U32(u32::from_be_bytes(self.mutator.input.try_into().unwrap()))
                }
                Type::U64(_) => {
                    Type::U64(u64::from_be_bytes(self.mutator.input.try_into().unwrap()))
                }
                Type::U128(_) => {
                    Type::U128(u128::from_be_bytes(self.mutator.input.try_into().unwrap()))
                }
                Type::Bool(_) => Type::Bool(self.mutator.input[0] != 0),
                Type::Vector(_, _) => todo!(),
            });
        }
        res
    }
}
