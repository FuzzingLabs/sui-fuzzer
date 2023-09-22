use crate::fuzzer::coverage::Coverage;

pub trait Runner {

    // Runs the selected target
    fn execute(&mut self) -> Result<Option<Vec<Coverage>>, String>;

}
