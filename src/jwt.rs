use crate::errors::{Error, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct SmartHomeCommandPayload {
    /// 'lock' | 'unlock' | 'toggle' | 'status'
    command: String,

    /// request ID, should be returned in asynchronous reponse so that we can match the reponse to request
    id: String,
}

impl fmt::Display for SmartHomeCommandPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.command, self.id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: u64,
    iat: u64,
    command: SmartHomeCommandPayload,
}

/// JWTService can be used for:
///     signing, JWT tokens utilizing RS256.
///         in this case service should hold pair of keys, i.e. private & public key
///     verification of JWT tokens utilizing RS256
///         in this case only public key of JWT creator/signer is needed (hence private_key is Option)
pub struct JWTService {
    public_key: String,
    private_key: Option<String>,
}

impl JWTService {
    pub fn new(public_key: String, private_key: Option<String>) -> Self {
        JWTService {
            public_key,
            private_key,
        }
    }

    pub fn get_token_validity() -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let validity = now + 60;
        validity
    }

    pub fn sign(&self, claims: Claims) -> Result<String> {
        let token = encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(
                &self.private_key.as_ref().unwrap().to_owned().into_bytes(),
            )?,
        )?;
        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<Claims> {
        let mut aud = std::collections::HashSet::new();
        aud.insert("myhome-cc-smarthome-microcontroller".to_owned());

        let validation = Validation {
            iss: Some("myhome-cc-smarthome-aog".to_owned()),
            sub: Some("myhome-cc-smarthome-microcontroller-myhome".to_owned()),
            aud: Some(aud),
            validate_exp: true,
            leeway: 0,
            validate_nbf: false,
            algorithms: vec![Algorithm::RS256],
        };

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_rsa_pem(&self.public_key.to_owned().into_bytes())?,
            &validation,
        )?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PRIVATE_KEY: &str = r#"
-----BEGIN RSA PRIVATE KEY-----
MIICWwIBAAKBgQCC8fI2z6Aeppz9GMAO5iIJsuMF/+gYMLh8AaYT/4RI06ggEybB
fptsGryUIr1nDZ0BAefiudUC7QGc979ZtIOVOvNsd0+OO+0upjvbggic2r04NeNz
T6x+LYV0Qn+Gdpt2YLUSubnMX7DtDfYj9wnbmLxQyHo9ZifUH6e/0aSAtQIDAQAB
AoGAH1CvRU5oE4xy9NBLdgSxVTJzuoaVwds882MNjbDIuQXtKiaKWTHnB3ZpbN/V
/eQyjQAgrYcVmwqFHT3ehBx4bwdNQiLDTqQTLc51530cIGr2pvSzLigNskQ1FymM
0x5dWz42wvG+bIZ/LsHC9fDF/66ueeoa6dBPc67aHYc0Zc0CQQD3GyrpVmLv/DgL
Y6dHoophvUdn7NBhLMMD5xOSjftHUNsbZnNMSCkH8Y1Qqs//f+z29aNXxIVsb0Qx
LO9ZNlRbAkEAh6h49WRP+eZAEF8CBIk68kb6ThmcCPIgwrGDz5UVVUaxXG+Mlhvu
Gp1lPq3XltfzjbtdYPNUxsmp8kXBgOBMLwJAdXVEgIW/obOSMFe+PB7XCH6gYpX+
tzI/wKsmcpNqzgtxGyUnySrD1jLLqXyIQjrcuMcqTZ3sjg6Vq4pge1eH4QJANeWz
mRtvwwO99EMrFA5JwzR8AkeefNdmOLQ6gGDlBup9URJossMKCLz8GrkK7L2D4I3O
fdRvnENHKCCazs9OtQJAVHdMfvfP0Cl2K6tXfcxYwVrKqRksfyDrjtYmxEzkJU1q
ni/ZjR55vIi9Ynn3m3eETkMpJe0dm71Ou3aOrwtoEQ==
-----END RSA PRIVATE KEY-----
        "#;

    const SAMPLE_PUBLIC_KEY: &str = r#"
-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCC8fI2z6Aeppz9GMAO5iIJsuMF
/+gYMLh8AaYT/4RI06ggEybBfptsGryUIr1nDZ0BAefiudUC7QGc979ZtIOVOvNs
d0+OO+0upjvbggic2r04NeNzT6x+LYV0Qn+Gdpt2YLUSubnMX7DtDfYj9wnbmLxQ
yHo9ZifUH6e/0aSAtQIDAQAB
-----END PUBLIC KEY-----
        "#;

    // cargo test -- --show-output test_sign
    #[test]
    fn test_sign() -> Result<()> {
        let jwtSvc = JWTService::new(SAMPLE_PUBLIC_KEY.to_owned(), Some(SAMPLE_PRIVATE_KEY.to_owned()));

        Ok(())
    }

    // cargo test -- --show-output test_verify
    #[test]
    fn test_verify() -> Result<()> {
        let jwtSvc = JWTService::new(SAMPLE_PUBLIC_KEY.to_owned(), None);
        Ok(())
    }

    // cargo test -- --show-output test_both_sign_verify
    #[test]
    fn test_both_sign_verify() -> Result<()> {
        Ok(())
    }
}
