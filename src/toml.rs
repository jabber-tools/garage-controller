use crate::errors::Result;
use serde::Deserialize;
use toml;

/// master configuration file of application
#[derive(Debug, Deserialize)]
pub struct ApplicationConfiguration {
    pub mqtt: MQTT,
    pub aes: AES,
    pub smart_home: SmartHome,
    pub microcontroller: MicroController,
}

/// defines attributes of mqtt section
#[derive(Debug, Deserialize, Default)]
pub struct MQTT {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

/// defines attributes of aes section
#[derive(Debug, Deserialize)]
pub struct AES {
    pub key: String,
}

/// defines attributes of smart_home section
#[derive(Debug, Deserialize)]
pub struct SmartHome {
    pub pub_key: String,
}

/// defines attributes of smart_home section
#[derive(Debug, Deserialize)]
pub struct MicroController {
    pub pub_key: String,
    pub priv_key: String,
}

impl ApplicationConfiguration {
    pub fn new(toml_path: &str) -> Result<ApplicationConfiguration> {
        let toml_str = std::fs::read_to_string(toml_path)?;
        let app_config = toml::from_str::<ApplicationConfiguration>(&toml_str)?;
        Ok(app_config)
    }
}
