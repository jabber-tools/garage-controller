/// TBD: let's try this https://docs.rs/openssl/0.9.24/openssl/aes/index.html
use crate::errors::Result;
use aes::Aes256;

fn encrypt(test: &str, encryption_key: &str) -> String {
    "".to_owned()
}

fn decrypt(test: &str, encryption_key: &str) -> String {
    "".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // cargo test -- --show-output test_encrypt
    #[test]
    fn test_encrypt() -> Result<()> {
        let encrypted = encrypt("hello there", "tbd...");
        Ok(())
    }
}
