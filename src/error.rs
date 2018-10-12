use std::{error, fmt, result};

#[derive(Debug)]
pub struct Error {
    msg: String
}

impl Error {
    pub fn new(msg: String) -> Self {
        Error { msg }
    }
}

pub trait ResultContext<T, E> {
    fn context(self, s: String) -> Result<T, Error>;
}

impl<T, E> ResultContext<T, E> for result::Result<T, E>
        where E: error::Error
{
    fn context(self, s: String) -> Result<T, Error> {
        self.map_err(|e| Error::new(format!("{}: {}", s, e.description())))
    }
}

impl<E: error::Error> From<E> for Error {
    fn from(e: E) -> Self {
        Error::new(e.description().to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
