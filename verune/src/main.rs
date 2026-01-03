use clap::{Arg, ArgAction, ArgMatches, Command, arg, command};
use std::{collections::HashMap, fs::write, process::exit};

fn handle_commands() -> ArgMatches {
    command!()
        .arg(arg!(-g --global "Use the global configuration").action(ArgAction::SetTrue))
        .arg(
            arg!(-c --config "The configuration to use")
                .action(ArgAction::Set)
                .value_name("CONFIG"),
        )
        .subcommand(
            Command::new("check")
                .about("Checks all runtime versions for their existence")
                .long_about(
                    "This subcommand performs generic version resolution, and then \
                    identifies whether or not all of the runtime versions actually checks.",
                ),
        )
        .subcommand(
            Command::new("switch")
                .about("Switch a runtime's version")
                .long_about(
                    "This simply switches a runtime's version. By default, it \
                    will check if the version is available and safe to use; this may be \
                    avoided by using the --skip-check argument.",
                )
                .arg(
                    Arg::new("skip-check")
                        .short('u')
                        .long("skip-check")
                        .help("Avoid validating the version's installation")
                        .action(ArgAction::SetTrue),
                )
                .arg(arg!(<RUNTIME> "The runtime to switch"))
                .arg(arg!(<VERSION> "The version to switch to")),
        )
        .get_matches()
}

macro_rules! verify_config {
    ($x: expr) => {
        if $x.is_none() {
            let bin = env!("CARGO_BIN_NAME");
            eprintln!(
                "{}: No configuration data was provided. To create a configuration, \
                please ensure you have a runtime installed and then switch to it using \"{} switch\"",
                bin, bin
            );
            exit(2);
        }
    };
}

fn main() {
    let matches: ArgMatches = handle_commands();
    let mut error_status: (i32, String, bool) = (
        1,
        "No subcommand was passed to verune; for a list of subcommands, please use \"verstring help\"".into(),
        false,
    );

    let config_path: &str = if let Some(path) = matches.get_one::<&str>("config") {
        path
    } else {
        ".ver.ron"
    };
    let mut config: Option<HashMap<String, String>> = libver::parse_config(config_path).ok();

    if matches.subcommand_matches("check").is_some() {
        verify_config!(config);
        todo!("Unimplemented command")
    } else if let Some(matches) = matches.subcommand_matches("switch") {
        if config.is_none() {
            config = Some(HashMap::new());
        }
        let mut config_data: HashMap<String, String> = config.unwrap();
        let runtime: String = matches.get_one::<String>("RUNTIME").unwrap().to_string();
        let version: String = matches.get_one::<String>("VERSION").unwrap().to_string();
        config_data.insert(runtime.clone(), version.clone());
        if let Ok(data) = ron::to_string(&config_data) {
            error_status = match write(config_path, data) {
                Ok(_) => (
                    0,
                    format!(
                        "Successfully switched runtime \"{}\" to version {}",
                        runtime, version
                    ),
                    true,
                ),
                Err(e) => (
                    1,
                    format!("Could not write to configuration file: {}", e),
                    false,
                ),
            };
        }
    }

    if error_status.2 {
        println!("{}: {}", env!("CARGO_BIN_NAME"), error_status.1);
    } else if error_status.0 != 0 {
        eprintln!("{}: {}", env!("CARGO_BIN_NAME"), error_status.1);
    }
    exit(error_status.0);
}
