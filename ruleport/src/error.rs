use std;
use std::fmt::{self, Display};

use serde::{de, ser};

use crate::Iterable;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    UnrecognizedSyntax(u32, u32),
    SequenceKeptOpen(Iterable, u32, u32),
    InvalidKey(u32, u32),
    InvalidValue(u32, u32),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

macro_rules! format_as_str {
    ($($arg:tt)*) => {
        format!($($arg)*).as_str()
    };
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::UnrecognizedSyntax(line, column) => formatter.write_str(format_as_str!(
                "syntax could not be properly parsed ({}:{})",
                line,
                column
            )),
            Error::SequenceKeptOpen(data, line, column) => formatter.write_str(format_as_str!(
                "the {} at location ({}:{}) was kept open",
                data,
                line,
                column
            )),
            Error::InvalidKey(line, column) => formatter.write_str(format_as_str!(
                "key at location ({}:{}) is invalid",
                line,
                column
            )),
            Error::InvalidValue(line, column) => formatter.write_str(format_as_str!(
                "value at location ({}:{}) was invalid",
                line,
                column
            )),
        }
    }
}

impl std::error::Error for Error {}
