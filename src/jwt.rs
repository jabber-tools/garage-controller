use crate::errors::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,

    /// 'lock' | 'unlock' | 'toggle' | 'status'
    pub command: String,

    /// request ID, should be returned in asynchronous response so that we can match the response to request
    pub id: String,
}

impl Default for Claims {
    fn default() -> Self {
        let iat_val = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let exp_val = iat_val + 60;

        Claims {
            iss: String::from("myhome-cc-smarthome-aog"),
            sub: String::from("myhome-cc-smarthome-microcontroller-myhome"),
            aud: String::from("myhome-cc-smarthome-microcontroller"),
            exp: exp_val,
            iat: iat_val,
            command: "".to_owned(),
            id: "".to_owned(),
        }
    }
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

    pub fn sign(&self, payload: Claims) -> Result<String> {
        let token = encode(
            &Header::new(Algorithm::RS256),
            &payload,
            &EncodingKey::from_rsa_pem(
                &self.private_key.as_ref().unwrap().to_owned().into_bytes(),
            )?,
        )?;
        Ok(token)
    }

    pub fn verify(&self, token: &str, validate_expiry: bool) -> Result<Claims> {
        let mut aud = std::collections::HashSet::new();
        aud.insert("myhome-cc-smarthome-microcontroller".to_owned());

        let validation = Validation {
            iss: Some("myhome-cc-smarthome-aog".to_owned()),
            sub: Some("myhome-cc-smarthome-microcontroller-myhome".to_owned()),
            aud: Some(aud),
            validate_exp: validate_expiry,
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
    use lazy_static::lazy_static;
    use std::fs;

    lazy_static! {
        pub static ref SMART_HOME_ACTION_PRIVATE_KEY: String =
            fs::read_to_string("./examples/testdata/smart-home-priv.pem").unwrap_or("".to_owned());
        pub static ref SMART_HOME_ACTION_PUBLIC_KEY: String =
            fs::read_to_string("./examples/testdata/smart-home-pub.pem").unwrap_or("".to_owned());
        pub static ref SMART_HOME_ACTION_REAL_TOKEN: String =
            fs::read_to_string("./examples/testdata/smart_home_token.txt").unwrap_or("".to_owned());
    }

    // https://travistidwell.com/jsencrypt/demo/
    // SAMPLE_PRIVATE_KEY and SAMPLE_PUBLIC_KEY are 1024 bit long
    // jsonwebtoken (and underlying crypto libraries) require
    // RSA_PKCS1_2048_8192_SHA256 for RS256 algorithm, shorter keys
    // will be always invalid!
    #[allow(dead_code)]
    const SAMPLE_PRIVATE_KEY_1024: &str = r#"
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

    #[allow(dead_code)]
    const SAMPLE_PUBLIC_KEY_1024: &str = r#"
-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCC8fI2z6Aeppz9GMAO5iIJsuMF
/+gYMLh8AaYT/4RI06ggEybBfptsGryUIr1nDZ0BAefiudUC7QGc979ZtIOVOvNs
d0+OO+0upjvbggic2r04NeNzT6x+LYV0Qn+Gdpt2YLUSubnMX7DtDfYj9wnbmLxQ
yHo9ZifUH6e/0aSAtQIDAQAB
-----END PUBLIC KEY-----
        "#;

    // sample keys taken from jwt.io. Seems longer than keys above
    // and it passes through signing process
    // private key above throws: Error { message: "InvalidRsaKey" }
    const SAMPLE_PRIVATE_KEY_2048: &str = r#"
-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAnzyis1ZjfNB0bBgKFMSvvkTtwlvBsaJq7S5wA+kzeVOVpVWw
kWdVha4s38XM/pa/yr47av7+z3VTmvDRyAHcaT92whREFpLv9cj5lTeJSibyr/Mr
m/YtjCZVWgaOYIhwrXwKLqPr/11inWsAkfIytvHWTxZYEcXLgAXFuUuaS3uF9gEi
NQwzGTU1v0FqkqTBr4B8nW3HCN47XUu0t8Y0e+lf4s4OxQawWD79J9/5d3Ry0vbV
3Am1FtGJiJvOwRsIfVChDpYStTcHTCMqtvWbV6L11BWkpzGXSW4Hv43qa+GSYOD2
QU68Mb59oSk2OB+BtOLpJofmbGEGgvmwyCI9MwIDAQABAoIBACiARq2wkltjtcjs
kFvZ7w1JAORHbEufEO1Eu27zOIlqbgyAcAl7q+/1bip4Z/x1IVES84/yTaM8p0go
amMhvgry/mS8vNi1BN2SAZEnb/7xSxbflb70bX9RHLJqKnp5GZe2jexw+wyXlwaM
+bclUCrh9e1ltH7IvUrRrQnFJfh+is1fRon9Co9Li0GwoN0x0byrrngU8Ak3Y6D9
D8GjQA4Elm94ST3izJv8iCOLSDBmzsPsXfcCUZfmTfZ5DbUDMbMxRnSo3nQeoKGC
0Lj9FkWcfmLcpGlSXTO+Ww1L7EGq+PT3NtRae1FZPwjddQ1/4V905kyQFLamAA5Y
lSpE2wkCgYEAy1OPLQcZt4NQnQzPz2SBJqQN2P5u3vXl+zNVKP8w4eBv0vWuJJF+
hkGNnSxXQrTkvDOIUddSKOzHHgSg4nY6K02ecyT0PPm/UZvtRpWrnBjcEVtHEJNp
bU9pLD5iZ0J9sbzPU/LxPmuAP2Bs8JmTn6aFRspFrP7W0s1Nmk2jsm0CgYEAyH0X
+jpoqxj4efZfkUrg5GbSEhf+dZglf0tTOA5bVg8IYwtmNk/pniLG/zI7c+GlTc9B
BwfMr59EzBq/eFMI7+LgXaVUsM/sS4Ry+yeK6SJx/otIMWtDfqxsLD8CPMCRvecC
2Pip4uSgrl0MOebl9XKp57GoaUWRWRHqwV4Y6h8CgYAZhI4mh4qZtnhKjY4TKDjx
QYufXSdLAi9v3FxmvchDwOgn4L+PRVdMwDNms2bsL0m5uPn104EzM6w1vzz1zwKz
5pTpPI0OjgWN13Tq8+PKvm/4Ga2MjgOgPWQkslulO/oMcXbPwWC3hcRdr9tcQtn9
Imf9n2spL/6EDFId+Hp/7QKBgAqlWdiXsWckdE1Fn91/NGHsc8syKvjjk1onDcw0
NvVi5vcba9oGdElJX3e9mxqUKMrw7msJJv1MX8LWyMQC5L6YNYHDfbPF1q5L4i8j
8mRex97UVokJQRRA452V2vCO6S5ETgpnad36de3MUxHgCOX3qL382Qx9/THVmbma
3YfRAoGAUxL/Eu5yvMK8SAt/dJK6FedngcM3JEFNplmtLYVLWhkIlNRGDwkg3I5K
y18Ae9n7dHVueyslrb6weq7dTkYDi3iOYRW8HRkIQh06wEdbxt0shTzAJvvCQfrB
jg/3747WSsf/zBTcHihTRBdAv6OmdhV4/dD5YBfLAkLrd+mX7iE=
-----END RSA PRIVATE KEY-----
        "#;

    const SAMPLE_PUBLIC_KEY_2048: &str = r#"
-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAnzyis1ZjfNB0bBgKFMSv
vkTtwlvBsaJq7S5wA+kzeVOVpVWwkWdVha4s38XM/pa/yr47av7+z3VTmvDRyAHc
aT92whREFpLv9cj5lTeJSibyr/Mrm/YtjCZVWgaOYIhwrXwKLqPr/11inWsAkfIy
tvHWTxZYEcXLgAXFuUuaS3uF9gEiNQwzGTU1v0FqkqTBr4B8nW3HCN47XUu0t8Y0
e+lf4s4OxQawWD79J9/5d3Ry0vbV3Am1FtGJiJvOwRsIfVChDpYStTcHTCMqtvWb
V6L11BWkpzGXSW4Hv43qa+GSYOD2QU68Mb59oSk2OB+BtOLpJofmbGEGgvmwyCI9
MwIDAQAB
-----END PUBLIC KEY-----
        "#;

    // cargo test -- --show-output test_sign_and_verify
    #[test]
    fn test_sign_and_verify() -> Result<()> {
        let jwt_svc = JWTService::new(
            SAMPLE_PUBLIC_KEY_2048.to_owned(),
            Some(SAMPLE_PRIVATE_KEY_2048.to_owned()),
        );
        let token = jwt_svc.sign(Claims {
            command: "toggle".to_owned(),
            id: "123".to_owned(),
            ..Claims::default()
        })?;
        println!("token {}", token);

        let jwt_svc_verif = JWTService::new(SAMPLE_PUBLIC_KEY_2048.to_owned(), None);
        let claims = jwt_svc_verif.verify(&token, true)?;
        println!("claims {:#?}", claims);
        Ok(())
    }

    // cargo test -- --show-output test_sign_corrupt_fail_to_verify
    #[test]
    fn test_sign_corrupt_fail_to_verify() -> Result<()> {
        let jwt_svc = JWTService::new(
            SAMPLE_PUBLIC_KEY_2048.to_owned(),
            Some(SAMPLE_PRIVATE_KEY_2048.to_owned()),
        );
        let token = jwt_svc.sign(Claims {
            command: "toggle".to_owned(),
            id: "123".to_owned(),
            ..Claims::default()
        })?;
        println!("token {}", token);

        // replace last character of token with z effectively corrupting it
        let token = format!("{}z", &token[..token.len() - 1]);
        println!("token {}", token);

        let jwt_svc_verif = JWTService::new(SAMPLE_PUBLIC_KEY_2048.to_owned(), None);
        let claims = jwt_svc_verif.verify(&token, true);

        match claims {
            Ok(claims) => panic!("test_sign_corrupt_fail_to_verify expected error"),
            Err(error) => assert_eq!(
                error.message.contains("Base64 error: Invalid last symbol"),
                true
            ),
        }

        Ok(())
    }

    // cargo test -- --show-output test_verify_with_real_cert
    #[test]
    #[ignore]
    fn test_verify_with_real_cert() -> Result<()> {
        let jwt_svc_verif = JWTService::new(SMART_HOME_ACTION_PUBLIC_KEY.to_owned(), None);
        let claims = jwt_svc_verif.verify(&SMART_HOME_ACTION_REAL_TOKEN, false)?;
        println!("claims {:#?}", claims);
        Ok(())
    }

    // cargo test -- --show-output test_default
    #[test]
    fn test_default() {
        #[derive(Debug, PartialEq)]
        struct Foo {
            foo: i32,
            bar: i32,
        }

        impl Default for Foo {
            fn default() -> Self {
                Foo { foo: 123, bar: 0 }
            }
        }

        // see https://doc.rust-lang.org/stable/book/ch05-01-defining-structs.html#creating-instances-from-other-instances-with-struct-update-syntax
        let my_foo = Foo {
            bar: 456,
            ..Foo::default()
        };

        // alternative syntax ..Foo::default() -> ..Default::default()
        let my_foo2 = Foo {
            bar: 789,
            ..Default::default()
        };

        assert_eq!(my_foo, Foo { foo: 123, bar: 456 });
        assert_eq!(my_foo2, Foo { foo: 123, bar: 789 });
    }
}
