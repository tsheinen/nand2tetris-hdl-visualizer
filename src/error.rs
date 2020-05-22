use nand2tetris_hdl_parser::HDLParseError;
use std::error::Error;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct GenericError {
    details: String
}

impl GenericError {
    fn new(msg: &str) -> GenericError {
        GenericError { details: msg.to_string() }
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
