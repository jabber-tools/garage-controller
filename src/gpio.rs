use crate::errors::Result;
use log::debug;
use rppal;

// see https://www.raspberrypi-spy.co.uk/2012/06/simple-guide-to-the-rpi-gpio-header-and-pins/
// pin 7 / GPIO04, relay power connected to pin 1 (3V3), relay ground to pin 6
// rppal uses GPIO.BCM, not GPIO.BOARD numbering
const OUTPUT_PINT: u8 = 4;

pub struct Gpio {
    #[allow(dead_code)]
    gpio_handler: rppal::gpio::Gpio,
    pin: rppal::gpio::OutputPin,
}

impl Gpio {
    pub fn new() -> Result<Self> {
        debug!("initiating on arm architecture, creating real Gpio handler");
        let handler = rppal::gpio::Gpio::new()?;
        let output_pin = handler.get(OUTPUT_PINT)?.into_output();
        Ok(Gpio {
            gpio_handler: handler,
            pin: output_pin,
        })
    }

    pub fn set_pin_high(&mut self) {
        debug!("Setting pin HIGH");
        // based on test with real raspberry 3b
        // we need to call set_low to set the pin actually high
        self.pin.set_low();
    }

    pub fn set_pin_low(&mut self) {
        debug!("Setting pin LOW");
        // based on test with real raspberry 3b
        // we need to call set_high to set the pin actually low
        self.pin.set_high();
    }
}