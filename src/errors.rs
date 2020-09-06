use crypto::symmetriccipher;
use hex;
use std::result;
use toml;

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
            message: format!("jsonwebtoken::errors::Error: {}", error),
        }
    }
}

impl From<symmetriccipher::SymmetricCipherError> for Error {
    fn from(error: symmetriccipher::SymmetricCipherError) -> Error {
        Error {
            message: format!("symmetriccipher::SymmetricCipherError: {:#?}", error),
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Error {
        Error {
            message: format!("std::string::FromUtf8Error: {}", error),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error {
            message: format!("std::str::Utf8Error: {}", error),
        }
    }
}

impl From<hex::FromHexError> for Error {
    fn from(error: hex::FromHexError) -> Error {
        Error {
            message: format!("hex::FromHexError: {}", error),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Error {
        Error {
            message: format!("toml::de::Error: {}", error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error {
            message: format!("std::io::Error: {}", error),
        }
    }
}

impl From<mqtt_async_client::Error> for Error {
    fn from(error: mqtt_async_client::Error) -> Error {
        Error {
            message: format!("mqtt_async_client:Error: {}", error),
        }
    }
}
