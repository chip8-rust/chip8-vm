//! `std:error:Error` implementations

use std::fmt;
use std::io;
use std::error::Error;
use std::convert::From;

/// `Error` variants for public errors in this crate
#[derive(Debug)]
pub enum Chip8Error {
    /// I/O error
    Io(&'static str, Option<io::Error>),
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

impl From<io::Error> for Chip8Error {
    fn from(err: io::Error) -> Chip8Error {
        Chip8Error::Io("I/O error", Some(err))
    }
}
