use crate::errors::Result;
use mqtt_async_client::client::{
    Client, Publish, QoS, Subscribe, SubscribeTopic, Unsubscribe, UnsubscribeTopic,
};

use tokio::{self, time::Duration};

pub fn plain_client(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> mqtt_async_client::Result<Client> {
    Client::builder()
        .set_host(host.to_owned())
        .set_port(port)
        .set_username(Some(username.to_owned()))
        .set_password(Some(password.as_bytes().to_vec()))
        .set_connect_retry_delay(Duration::from_secs(1))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aes::decrypt;
    use crate::gpio;
    use crate::init_logging; // set RUST_LOG=garage_controller::mqtt=debug
    use crate::jwt::{Claims, JWTService};
    use lazy_static::lazy_static;
    use log::debug;
    use serde::Deserialize;
    use std::default::Default;
    use std::fmt::Debug;
    use std::fs;
    use std::time::Duration;
    use tokio::time::delay_for;
    use toml;

    lazy_static! {
        pub static ref SMART_HOME_ACTION_PUBLIC_KEY: String =
            fs::read_to_string("./examples/testdata/smart-home-pub.pem").unwrap_or("".to_owned());
        pub static ref AES_KEY: String =
            fs::read_to_string("./examples/testdata/aes.txt").unwrap_or("".to_owned());
        pub static ref MQTT_CONN: MQTT = {
            let mqtt = MQTT::new().unwrap_or_default();
            mqtt
        };
        pub static ref MICROCONTROLLER_PUBLIC_KEY: String =
            fs::read_to_string("./examples/testdata/microcontroller-pubkey.pem")
                .unwrap_or("".to_owned());
        pub static ref MICROCONTROLLER_PRIVATE_KEY: String =
            fs::read_to_string("./examples/testdata/microcontroller-privkey.pem")
                .unwrap_or("".to_owned());
    }

    #[derive(Debug, Deserialize, Default)]
    struct TomlFile {
        pub mqtt: MQTT,
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct MQTT {
        host: String,
        port: u16,
        username: String,
        password: String,
    }

    impl MQTT {
        fn new() -> crate::errors::Result<Self> {
            let toml_str = std::fs::read_to_string("./examples/testdata/mqtt.toml")?;
            let toml = toml::from_str::<TomlFile>(&toml_str)?;
            Ok(toml.mqtt)
        }
    }

    // cargo test -- --show-output test_plain_client
    #[test]
    #[ignore]
    fn test_plain_client() -> Result<()> {
        let mut rt = tokio::runtime::Runtime::new()?;

        let result = rt.block_on(async {
            let mut c = plain_client(
                &MQTT_CONN.host,
                MQTT_CONN.port,
                &MQTT_CONN.username,
                &MQTT_CONN.password,
            )?;
            c.connect().await?;
            println!("connected");
            c.disconnect().await?;
            println!("disconnected");
            Ok(())
        });
        result
    }

    // cargo test -- --show-output test_pub_and_sub
    #[test]
    #[ignore]
    fn test_pub_and_sub() -> Result<()> {
        let mut rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut c = plain_client(
                &MQTT_CONN.host,
                MQTT_CONN.port,
                &MQTT_CONN.username,
                &MQTT_CONN.password,
            )?;
            c.connect().await?;

            // Subscribe
            let subopts = Subscribe::new(vec![SubscribeTopic {
                qos: QoS::AtMostOnce,
                topic_path: "test/pub_and_sub".to_owned(),
            }]);
            let subres = c.subscribe(subopts).await?;
            subres.any_failures()?;

            // Publish
            let mut p = Publish::new("test/pub_and_sub".to_owned(), "x".as_bytes().to_vec());
            p.set_qos(QoS::AtMostOnce);
            c.publish(&p).await?;

            // Read
            let r = c.read_subscriptions().await?;
            assert_eq!(r.topic(), "test/pub_and_sub");
            assert_eq!(r.payload(), b"x");
            c.disconnect().await?;
            println!("pub_and_sub_plain OK!");
            Ok(())
        })
    }

    // cargo test -- --show-output test_sub_real_smart_home_msg
    #[test]
    #[ignore]
    fn test_sub_real_smart_home_msg() -> Result<()> {
        let mut rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut c = plain_client(
                &MQTT_CONN.host,
                MQTT_CONN.port,
                &MQTT_CONN.username,
                &MQTT_CONN.password,
            )?;
            c.connect().await?;

            // Subscribe
            let subopts = Subscribe::new(vec![SubscribeTopic {
                qos: QoS::AtMostOnce,
                topic_path: "garage/toggle".to_owned(),
            }]);
            let subres = c.subscribe(subopts).await?;
            subres.any_failures()?;

            // Read
            let r = c.read_subscriptions().await?;
            assert_eq!(r.topic(), "garage/toggle");

            let payload = String::from_utf8(r.payload().to_vec())?;
            println!("original payload from mqtt {}", payload);

            let decrypted_payload = decrypt(&payload, &AES_KEY)?;
            println!("decrypted payload from mqtt {}", decrypted_payload);

            let jwt_svc_verif = JWTService::new(SMART_HOME_ACTION_PUBLIC_KEY.to_owned(), None);
            let claims = jwt_svc_verif.verify(&decrypted_payload, true)?;
            println!("token verified. claims {:#?}", claims);

            let confirmation_payload = Claims {
                command: "confirmation".to_owned(),
                id: claims.id,
                ..Claims::default()
            };
            let jwt_svc_signing = JWTService::new(
                MICROCONTROLLER_PUBLIC_KEY.to_owned(),
                Some(MICROCONTROLLER_PRIVATE_KEY.to_owned()),
            );
            let confirmation_token = jwt_svc_signing.sign(confirmation_payload)?;
            println!("acknowledgment prepared {}", confirmation_token);

            // Publish
            let mut p = Publish::new(
                "garage/toggleConfirm".to_owned(),
                confirmation_token.as_bytes().to_vec(),
            );
            p.set_qos(QoS::AtMostOnce);
            c.publish(&p).await?;
            println!("acknowledgment sent!");

            init_logging(); // enable logging so that we see timestamps when calling delay_for
            let mut gpio = gpio::Gpio::new()?;
            debug!("setting pin high");
            gpio.set_pin_high();
            delay_for(Duration::from_millis(1000)).await;
            gpio.set_pin_low();
            debug!("setting pin low");

            c.disconnect().await?;
            Ok(())
        })
    }
}
