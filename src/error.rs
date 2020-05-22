use nand2tetris_hdl_parser::HDLParseError;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct GenericError {
    pub details: String
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for GenericError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<std::io::Error> for GenericError {
    fn from(x: std::io::Error) -> Self {
        GenericError {
            details: x.to_string()
        }
    }
}

impl From<HDLParseError> for GenericError {
    fn from(x: HDLParseError) -> Self {
        GenericError {
            details: x.to_string()
        }
    }
}
