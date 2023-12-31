use crate::fuzzer::coverage::Coverage;
use crate::fuzzer::error::Error;
use crate::mutator::types::Type;

pub trait Runner {
    /// Runs the selected target
    fn execute(&mut self, inputs: Vec<Type>) -> Result<Option<Coverage>, (Coverage, Error)>;
    /// Returns the target parameters
    fn get_target_parameters(&self) -> Vec<Type>;
}
