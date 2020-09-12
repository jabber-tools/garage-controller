pub mod aes;
pub mod cli;
pub mod errors;

#[cfg(all(target_family = "unix", target_arch = "arm"))]
pub mod gpio;

#[cfg(not(all(target_family = "unix", target_arch = "arm")))]
pub mod gpio_mock;

#[cfg(all(target_family = "unix", target_arch = "arm"))]
pub use gpio;

#[cfg(not(all(target_family = "unix", target_arch = "arm")))]
pub use gpio_mock as gpio;

pub mod jwt;
pub mod mqtt;
pub mod toml;

/// Utility function to enable log::debug logging in unit tests
pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
