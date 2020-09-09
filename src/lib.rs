pub mod aes;
pub mod errors;

#[cfg_attr(all(target_family = "unix", target_arch = "arm"), path = "gpio.rs")]
#[cfg_attr(all(target_family = "unix", target_arch = "x86"), path = "gpio_mock.rs")]
#[cfg_attr(all(target_family = "unix", target_arch = "x86_64"), path = "gpio_mock.rs")]
#[cfg_attr(windows, path = "gpio_mock.rs")]
pub mod gpio;

pub mod jwt;
pub mod mqtt;
pub mod toml;

/// Utility function to enable log::debug logging in unit tests
pub fn init_logging() {
    // enable in unit/integration tests selectively only when needed!
    // set RUST_LOG=gdf_translate::google::gcloud::storage_bucket_mgmt=debug
    let _ = env_logger::builder().is_test(true).try_init();
}
