use crate::fuzzer::coverage::Coverage;

pub trait Runner {

    type InputType;

    // Runs the selected target
    fn execute(&mut self, input: Self::InputType) -> Result<Option<Coverage>, (Coverage, String)>;

}
