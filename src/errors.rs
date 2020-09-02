use std::result;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Error { message }
    }
}

pub type Result<T> = result::Result<T, Error>;

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(error: jsonwebtoken::errors::Error) -> Error {
        Error {
            message: format!("{}", error),
        }
    }
}
