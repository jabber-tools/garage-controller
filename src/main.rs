use garage_controller::{
    aes::*,
    errors::{Error, Result},
    gpio,
    jwt::*,
    mqtt,
    toml::*,
};
use log::debug;
use mqtt_async_client::client::{Client, Publish, QoS, Subscribe, SubscribeTopic};
use std::fs;
use std::process;
use std::time::Duration;
use tokio::time::delay_for;

///
/// Convenience macro to replace following boilerplate:
///
/// if let Err(err) = result {
///    debug!("unable to load smart home public key");
///    debug!("error detail {}", err);
///    process::exit(1);
/// }
///
/// with:
/// eval_error!(result, "unable to load smart home public key");
///
macro_rules! eval_error {
    ($result:expr, $msg:expr) => {
        if let Err(err) = $result {
            debug!($msg);
            debug!("error detail {}", err);
            process::exit(1);
        }
    };
}

/// initial hardcoded version of main for preliminary testing
fn main() -> Result<()> {
    env_logger::init();
    debug!("Starting microcontroller");

    let APP_CONFIG: ApplicationConfiguration =
        ApplicationConfiguration::new("/tmp/smart-home/app_config.toml").unwrap(); // TODO: read toml file path from command line
    let SMART_HOME_ACTION_PUBLIC_KEY: String = {
        let result = fs::read_to_string(APP_CONFIG.smart_home.pub_key.to_owned());
        eval_error!(result, "unable to load smart home public key");
        result.unwrap()
    };
    let MICROCONTROLLER_PUBLIC_KEY: String = {
        let result = fs::read_to_string(APP_CONFIG.microcontroller.pub_key.to_owned());
        eval_error!(result, "unable to load microcontroller public key");
        result.unwrap()
    };
    let MICROCONTROLLER_PRIV_KEY: String = {
        let result = fs::read_to_string(APP_CONFIG.microcontroller.priv_key.to_owned());
        eval_error!(result, "unable to load microcontroller private key");
        result.unwrap()
    };
    let AES_KEY: String = APP_CONFIG.aes.key.to_owned();

    debug!(
        "SMART_HOME_ACTION_PUBLIC_KEY: {}",
        APP_CONFIG.smart_home.pub_key
    );
    debug!(
        "MICROCONTROLLER_PUBLIC_KEY:   {}",
        APP_CONFIG.microcontroller.pub_key
    );
    debug!(
        "MICROCONTROLLER_PRIV_KEY:     {}",
        APP_CONFIG.microcontroller.priv_key
    );

    let rt = tokio::runtime::Runtime::new();
    eval_error!(rt, "unable to initiate tokio runtime");

    let c: mqtt_async_client::Result<Client> = mqtt::plain_client(
        &APP_CONFIG.mqtt.host,
        APP_CONFIG.mqtt.port,
        &APP_CONFIG.mqtt.username,
        &APP_CONFIG.mqtt.password,
    );
    eval_error!(c, "unable to initiate mqtt client");

    let mut c = c.unwrap();

    rt.unwrap().block_on(async {
        let conn_result = c.connect().await;
        eval_error!(conn_result, "unable to connect to MQTT server");

        let subopts = Subscribe::new(vec![SubscribeTopic {
            qos: QoS::AtMostOnce,
            topic_path: "garage/toggle".to_owned(),
        }]);
        let subres = c.subscribe(subopts).await?;
        subres.any_failures()?;

        let mut gpio = gpio::Gpio::new()?;

        debug!("Starting main processing loop!");
        loop {
            debug!("waiting for new messages on topic garage/toggle");
            // Read subscription
            let r = c.read_subscriptions().await?;
            assert_eq!(r.topic(), "garage/toggle");

            let payload = String::from_utf8(r.payload().to_vec())?;
            debug!("original payload from mqtt {}", payload);

            let decrypted_payload = decrypt(&payload, &AES_KEY)?;
            debug!("decrypted payload from mqtt {}", decrypted_payload);

            let jwt_svc_verif = JWTService::new(SMART_HOME_ACTION_PUBLIC_KEY.to_owned(), None);
            let claims = jwt_svc_verif.verify(&decrypted_payload, true)?;
            debug!("token verified. claims {:#?}", claims);

            let confirmation_payload = Claims {
                command: "confirmation".to_owned(),
                id: claims.id,
                ..Claims::default()
            };
            let jwt_svc_signing = JWTService::new(
                MICROCONTROLLER_PUBLIC_KEY.to_owned(),
                Some(MICROCONTROLLER_PRIV_KEY.to_owned()),
            );
            let confirmation_token = jwt_svc_signing.sign(confirmation_payload)?;
            debug!("acknowledgment prepared {}", confirmation_token);

            debug!("setting pin high");
            gpio.set_pin_high();
            delay_for(Duration::from_millis(400)).await;
            gpio.set_pin_low();
            debug!("setting pin low, sending acknowledgment to smart-home");

            // Publish
            let mut p = Publish::new(
                "garage/toggleConfirm".to_owned(),
                confirmation_token.as_bytes().to_vec(),
            );
            p.set_qos(QoS::AtMostOnce);
            c.publish(&p).await?;
            debug!("acknowledgment sent!");
        } // main microcontroller loop

        // just so that async block return value can be infered
        // currently no way how to specify async block ret value like for asyn fn, must use turbo fish
        // https://rust-lang.github.io/async-book/07_workarounds/03_err_in_async_blocks.html
        #[allow(unreachable_code)]
        Ok::<(), Error>(())
    })?;
    Ok(())
}
