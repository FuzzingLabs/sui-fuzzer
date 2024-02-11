use crate::mutator::mutator::Mutator;
use basic_mutator::{self, EmptyDatabase};

use super::{rng::Rng, types::Type};

pub struct SuiMutator {
    seed: u64,
    mutator: basic_mutator::Mutator,
}

impl SuiMutator {
    pub fn new(seed: u64, max_input_size: usize) -> Self {
        let mutator = basic_mutator::Mutator::new()
            .seed(seed)
            .max_input_size(max_input_size);
        SuiMutator { seed, mutator }
    }
}

impl Mutator for SuiMutator {
    fn mutate(&mut self, inputs: &Vec<Type>, nb_mutation: usize) -> Vec<Type> {
        let mut res = vec![];

        for input in inputs {
            self.mutator.input.clear();
            match input {
                Type::U8(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U16(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U32(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U64(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U128(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::Bool(b) => self
                    .mutator
                    .input
                    .extend_from_slice(&[if *b { 1 } else { 0 }]),
                Type::Vector(_, vec) => {
                    let buffer: Vec<u8> = vec
                        .iter()
                        .map(|v| {
                            if let Type::U8(a) = v {
                                a.to_owned()
                            } else {
                                todo!()
                            }
                        })
                        .collect();
                    self.mutator.input.extend_from_slice(&buffer);
                }
                Type::Struct(_) => (),
                Type::Reference(_, _) => (),
                _ => unimplemented!(),
            }

            self.mutator.mutate(nb_mutation, &EmptyDatabase);

            // The size of the input needs to be the right size
            res.push(match input {
                Type::U8(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(1, 0);

                    Type::U8(u8::from_be_bytes(v[0..1].try_into().unwrap()))
                }
                Type::U16(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(2, 0);

                    Type::U16(u16::from_be_bytes(v[0..2].try_into().unwrap()))
                }
                Type::U32(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(4, 0);

                    Type::U32(u32::from_be_bytes(v[0..4].try_into().unwrap()))
                }
                Type::U64(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(8, 0);

                    Type::U64(u64::from_be_bytes(v[0..8].try_into().unwrap()) % 1000)
                }
                Type::U128(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(16, 0);

                    Type::U128(u128::from_be_bytes(v[0..16].try_into().unwrap()))
                }
                Type::Bool(_) => Type::Bool(self.mutator.input[0] != 0),
                Type::Vector(_, _) => Type::Vector(
                    Box::new(Type::U8(0)),
                    self.mutator
                        .input
                        .iter()
                        .map(|a| Type::U8(a.to_owned()))
                        .collect(),
                ),
                Type::Struct(types) => Type::Struct(self.mutate(types, nb_mutation)),
                Type::Reference(_, _) => input.clone(),
                _ => unimplemented!(),
            });
        }
        res
    }

    fn generate_number(&self, min: u64, max: u64) -> u64 {
        let mut rng = Rng {
            seed: self.seed,
            exp_disabled: false,
        };
        rng.rand(min.try_into().unwrap(), max.try_into().unwrap())
            .try_into()
            .unwrap()
    }
}
