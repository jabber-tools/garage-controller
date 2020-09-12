use crate::errors::Result;
use log::debug;

pub struct Gpio;

impl Gpio {
    pub fn new() -> Result<Self> {
        debug!("initiating on non-arm architecture, creating dummy Gpio handler");
        Ok(Gpio)
    }

    pub fn set_pin_high(&mut self) {
        debug!("Setting dummy pin HIGH");
    }

    pub fn set_pin_low(&mut self) {
        debug!("Setting dummy pin LOW");
    }
}
