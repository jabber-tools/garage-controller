use crypto::symmetriccipher;
use hex;
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

impl From<symmetriccipher::SymmetricCipherError> for Error {
    fn from(error: symmetriccipher::SymmetricCipherError) -> Error {
        Error {
            message: format!("{:#?}", error),
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Error {
        Error {
            message: format!("{}", error),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error {
            message: format!("{}", error),
        }
    }
}

impl From<hex::FromHexError> for Error {
    fn from(error: hex::FromHexError) -> Error {
        Error {
            message: format!("{}", error),
        }
    }
}
