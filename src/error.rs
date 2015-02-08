//! `std:error:Error` implementations

use std::fmt;
use std::old_io::{IoError};
use std::error::{Error, FromError};

/// `Error` variants for public errors in this crate
pub enum Chip8Error {
    /// I/O error
    Io(&'static str, Option<IoError>),
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        return write!(fmt, "{}", self.description())
    }
}

impl Error for Chip8Error {
    fn description(&self) -> &str {
        match *self {
            Chip8Error::Io(desc, _) => desc,
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            Chip8Error::Io(_, Some(ref cause)) => Some(cause),
            _ => None,
        }
    }
}

impl FromError<IoError> for Chip8Error {
    fn from_error(err: IoError) -> Chip8Error {
        Chip8Error::Io("Internal IO error: ", Some(err))
    }
}
