use super::types::Type;

pub trait Mutator {
    fn mutate(&mut self, inputs: &Vec<Type>, nb_mutation: usize) -> Vec<Type>;
    fn generate_number(&self, min: u64, max: u64) -> u64;
}
