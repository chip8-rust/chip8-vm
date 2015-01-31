use std::fmt; 
use std::old_io::{IoError};
use std::error::{Error, FromError};

pub enum Ch8Error {
    Io(&'static str, Option<IoError>),
}

impl fmt::Display for Ch8Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        return write!(fmt, "{}", self.description())
    }
}

impl Error for Ch8Error {
    fn description(&self) -> &str {
        use self::Ch8Error::*;
        match *self {
            Io(desc, _) => desc
        }
    }

    fn cause(&self) -> Option<&Error> {
        use self::Ch8Error::*;
        match *self {
            Io(_, Some(ref cause)) => Some(cause),
            _ => None
        }
    }
}

impl FromError<IoError> for Ch8Error {
    fn from_error(err: IoError) -> Ch8Error {
        Ch8Error::Io("Internal IO error: ", Some(err))
    }
}
