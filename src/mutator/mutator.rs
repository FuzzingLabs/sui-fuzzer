pub trait Mutator {

    fn mutate(&mut self, input: &Vec<u8>, nb_mutation: usize) -> Vec<u8>;

}
