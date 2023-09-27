use crate::mutator::mutator::Mutator;
use basic_mutator::{self, EmptyDatabase};

pub struct SuiMutator {
    mutator: basic_mutator::Mutator
}

impl SuiMutator {

    pub fn new(seed: u64, max_input_size: usize) -> Self {
        let mutator = basic_mutator::Mutator::new().seed(seed)
            .max_input_size(max_input_size).printable(true);
        SuiMutator {
            mutator
        }
    }

}

impl Mutator for SuiMutator {

    fn mutate(&mut self, input: &Vec<u8>, nb_mutation: usize) -> Vec<u8> {
        self.mutator.input.clear();
        self.mutator.input.extend_from_slice(input);

        self.mutator.mutate(nb_mutation, &EmptyDatabase);

        self.mutator.input.clone()
    }

}
