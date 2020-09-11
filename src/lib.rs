pub mod aes;
pub mod cli;
pub mod errors;

// rustfmt does not understand path properly:( cargo fmt will cause content of gpio_mock.rs
// to be replaced with gpio.rs! Similar issue described here: https://github.com/rust-lang/rustfmt/issues/2407
// luckily we will always find this during compilation on windows since rppal is not in windows dependencies
// and hence code will not compile
#[cfg_attr(all(target_family = "unix", target_arch = "arm"), path = "gpio.rs")]
#[cfg_attr(
    all(target_family = "unix", target_arch = "x86"),
    path = "gpio_mock.rs"
)]
#[cfg_attr(
    all(target_family = "unix", target_arch = "x86_64"),
    path = "gpio_mock.rs"
)]
#[cfg_attr(windows, path = "gpio_mock.rs")]
pub mod gpio;

pub mod jwt;
pub mod mqtt;
pub mod toml;

/// Utility function to enable log::debug logging in unit tests
pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
