use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use std::env::current_exe;

pub mod aes;
pub mod cli;
pub mod errors;

#[cfg(all(target_family = "unix", target_arch = "arm"))]
pub mod gpio_arm;

#[cfg(not(all(target_family = "unix", target_arch = "arm")))]
pub mod gpio_mock;

#[cfg(all(target_family = "unix", target_arch = "arm"))]
pub use gpio_arm as gpio;

#[cfg(not(all(target_family = "unix", target_arch = "arm")))]
pub use gpio_mock as gpio;

pub mod jwt;
pub mod mqtt;
pub mod toml;

fn init_with_default_logging_config() {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();
    let _ = log4rs::init_config(config).unwrap();
}

pub fn init_logging() {
    // from current binary path remove executable name (e.g. ddf_translate.exe) and add temp folder name
    let logger_file = current_exe();
    if let Err(err) = logger_file {
        println!("unable to init logging, stdout logger will be used.");
        println!("Error detail {}", err);
        init_with_default_logging_config();
    } else {
        let logger_file = logger_file
            .unwrap()
            .into_boxed_path()
            .as_ref()
            .parent()
            .unwrap()
            .join("config")
            .join("log4rs.yml");

        println!("logger_file: {:#?}", logger_file);
        let log_init_result = log4rs::init_file(logger_file, Default::default());
        if let Err(err) = log_init_result {
            println!("unable to init logging, stdout logger will be used.");
            println!("Error detail {}", err);
            init_with_default_logging_config();
        }
    }
}
