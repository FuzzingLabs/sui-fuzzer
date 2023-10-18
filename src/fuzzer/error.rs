#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    Abort{message: String},
    OutOfBound{message: String},
    Unknown{message: String},
    // TODO Add more errors
}