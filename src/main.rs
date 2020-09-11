use ctrlc;
use garage_controller::{
    aes,
    cli::{get_cmd_line_parser, get_cmdl_options},
    errors::{Error, Result},
    gpio, jwt, mqtt,
    toml::ApplicationConfiguration,
};
use log::{debug, trace};
use mqtt_async_client::client::{Client, Publish, QoS, Subscribe, SubscribeTopic};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, process, sync::Arc};
use tokio::time::{delay_for, timeout, Duration};

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

    let cmd_line_matches = get_cmd_line_parser().get_matches();
    let cmd_line_opts = get_cmdl_options(&cmd_line_matches);

    debug!("Starting microcontroller");

    #[allow(non_snake_case)]
    let APP_CONFIG: ApplicationConfiguration =
        ApplicationConfiguration::new(cmd_line_opts.app_config_path.to_str().unwrap())?;

    #[allow(non_snake_case)]
    let SMART_HOME_ACTION_PUBLIC_KEY: String = {
        let result = fs::read_to_string(APP_CONFIG.smart_home.pub_key.to_owned());
        eval_error!(result, "unable to load smart home public key");
        result.unwrap()
    };

    #[allow(non_snake_case)]
    let MICROCONTROLLER_PUBLIC_KEY: String = {
        let result = fs::read_to_string(APP_CONFIG.microcontroller.pub_key.to_owned());
        eval_error!(result, "unable to load microcontroller public key");
        result.unwrap()
    };

    #[allow(non_snake_case)]
    let MICROCONTROLLER_PRIV_KEY: String = {
        let result = fs::read_to_string(APP_CONFIG.microcontroller.priv_key.to_owned());
        eval_error!(result, "unable to load microcontroller private key");
        result.unwrap()
    };

    #[allow(non_snake_case)]
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

        // set the PIN initially LOW
        debug!("setting relay PIN initially LOW");
        gpio.set_pin_low();
        debug!("setting relay PIN initially LOW - done");

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            debug!("SIGINT/CTRL_C_EVENT/CTRL_BREAK_EVENT detected, terminating main loop");
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        debug!("Starting main processing loop!");
        while running.load(Ordering::SeqCst) {
            trace!("waiting for new messages on topic garage/toggle");

            // Read subscription with timeout to enable ctrl+c to be handled continuously
            let r = timeout(Duration::from_secs(1), mqtt::read_subscriptions(&mut c)).await;
            if r.is_err() {
                trace!("read_subscriptions timeout, continuing to allow potential ctrlc.");
                continue;
            }
            let r = r.unwrap();
            eval_error!(r, "unable to read subscriptions from MQTT server");
            let r = r.unwrap();

            // Read subscription in "blocking/awaiting way" with no timeout.
            // asynchronous block will be waiting here forever unless it receives
            // mqtt message to process (preventing ctrl+c condition in while loop being evaluated)
            // this works fine until we want to support ctrl+c handler
            // let r = mqtt::read_subscriptions(&mut c).await?;
            assert_eq!(r.topic(), "garage/toggle");

            let payload = String::from_utf8(r.payload().to_vec())?;
            debug!("original payload from mqtt {}", payload);

            let decrypted_payload = aes::decrypt(&payload, &AES_KEY)?;
            debug!("decrypted payload from mqtt {}", decrypted_payload);

            let jwt_svc_verif = jwt::JWTService::new(SMART_HOME_ACTION_PUBLIC_KEY.to_owned(), None);
            let claims = jwt_svc_verif.verify(&decrypted_payload, true)?;
            debug!("token verified. claims {:#?}", claims);

            let confirmation_payload = jwt::Claims {
                command: "confirmation".to_owned(),
                id: claims.id,
                ..jwt::Claims::default()
            };
            let jwt_svc_signing = jwt::JWTService::new(
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
        Ok::<(), Error>(())
    })?;
    debug!("main processing loop finished, quiting now. bye!");
    Ok(())
}
