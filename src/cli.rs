use clap::{App, Arg, ArgMatches};
use std::path::Path;

#[derive(Debug)]
pub struct CommandLine<'a> {
    pub app_config_path: &'a Path,
}

impl<'a> CommandLine<'a> {
    fn new(app_config_path: &'a Path) -> Self {
        CommandLine { app_config_path }
    }
}

pub fn get_cmd_line_parser<'a, 'b>() -> App<'a, 'b> {
    App::new("Garage Microcontroller")
        .version("v0.1.1")
        .author("Adam Bezecny")
        .about("Raspberry Pi Software for controlling garage door via MQTT and GPIO")
        .arg(
            Arg::with_name("app_config_path")
                .short("f")
                .long("config-file")
                .value_name("FILE")
                .help("Path to TOML file with application configuration. See examples/app_config_example.toml.")
                .takes_value(true)
                .required(true),
        )
}

pub fn get_cmdl_options<'a>(matches: &'a ArgMatches) -> CommandLine<'a> {
    let app_config_path = Path::new(matches.value_of("app_config_path").unwrap());

    CommandLine::new(app_config_path)
}
