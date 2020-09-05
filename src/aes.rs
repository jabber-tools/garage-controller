use crate::errors::{Error, Result};
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::{aes, blockmodes, buffer};
use hex;
use rand::RngCore;

/// for implementation details see https://github.com/DaGenix/rust-crypto/blob/master/examples/symmetriccipher.rs#L17
fn encrypt_impl(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let mut encryptor =
        aes::cbc_encryptor(aes::KeySize::KeySize256, key, iv, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;

        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

/// for implementation details see https://github.com/DaGenix/rust-crypto/blob/master/examples/symmetriccipher.rs#L82
fn decrypt_impl(encrypted_data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    let mut decryptor =
        aes::cbc_decryptor(aes::KeySize::KeySize256, key, iv, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

pub fn encrypt(text: &str, encryption_key: &str) -> Result<String> {
    let key = encryption_key.as_bytes();
    let data_to_encrypt = text.as_bytes();
    let mut iv: [u8; 16] = [0; 16];

    let mut rng = rand::rngs::OsRng::default();
    rng.fill_bytes(&mut iv);

    let encrypted_data = encrypt_impl(data_to_encrypt, &key, &iv)?;
    let strigified_data = hex::encode(encrypted_data);
    let iv = hex::encode(&iv);
    Ok(format!("{}:{}", iv, strigified_data))
}

pub fn decrypt(text: &str, encryption_key: &str) -> Result<String> {
    let split: Vec<&str> = text.split(":").collect();
    if split.len() != 2 {
        return Err(Error::new(
            "Wrong format of encrypted data, must be <<iv>>:<<data>>!".to_owned(),
        ));
    }
    let iv = hex::decode(split[0])?;
    let encrypted_text = hex::decode(split[1])?;

    let decrytped_text = decrypt_impl(&encrypted_text[..], encryption_key.as_bytes(), &iv[..])?;
    let stringified_data = String::from_utf8(decrytped_text)?;
    Ok(stringified_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    // cargo test -- --show-output test_encrypt_impl_decrypt_impl
    #[test]
    fn test_encrypt_impl_decrypt_impl() -> Result<()> {
        let message = "Hello World!";

        let mut key: [u8; 32] = [0; 32];
        let mut iv: [u8; 16] = [0; 16];

        let mut rng = rand::rngs::OsRng::default();

        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut iv);

        let encrypted_data = encrypt_impl(message.as_bytes(), &key, &iv).ok().unwrap();
        let decrypted_data = decrypt_impl(&encrypted_data[..], &key, &iv).ok().unwrap();

        assert_eq!(message.as_bytes(), &decrypted_data[..]);

        Ok(())
    }

    // cargo test -- --show-output test_encrypt_decrypt
    #[test]
    fn test_encrypt_decrypt() -> Result<()> {
        let data_to_encrypt = "Adam was here!";
        let secret = "546191f3-ac70-43c3-b9ad-a26d8fds";

        let encrypted = encrypt(data_to_encrypt, secret)?;
        let decrypted = decrypt(&encrypted, secret)?;

        assert_eq!(data_to_encrypt, decrypted);

        Ok(())
    }
}
