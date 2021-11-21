use std::fmt;
use std::io;

#[derive(Debug)]
pub struct PatchError {
    pub message: String,
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<io::Error> for PatchError {
    fn from(error: io::Error) -> Self {
        PatchError {
            message: error.to_string(),
        }
    }
}
