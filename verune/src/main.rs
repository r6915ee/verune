use clap::{Arg, ArgAction, ArgMatches, Command, arg, command};
use libver::{Runtime, RuntimeMetadata, conf, exec};
use std::{
    collections::HashMap,
    env,
    fs::write,
    process::{Stdio, exit},
};

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
                    identifies whether or not all of the runtime versions actually exist.",
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
        .subcommand(
            Command::new("scope")
                .about("Run a command within a scope")
                .long_about(
                    "This subcommand creates a child process with a modified environment. \
                    This environment inherits from the current program, but prepends the \
                    $PATH environment variable with both the current version directory of \
                    each runtime and its search paths, alongside setting the $VER_OVERRIDE \
                    environment variable for tools like prompts to use. This subcommand is \
                    useful for both general execution of compilers and interpreters, as well \
                    as package managers, IDEs, and even more scenarios, making it the subcommand \
                    with the most use.\n\n\
                    By default, the subcommand will attempt to use the command specified \
                    in $SHELL, but it's possible to specify different programs.\n\n\
                    Do note that this subcommand will start the program in the modified \
                    environment immediately. This means that it is possible to immediately \
                    invoke runtimes using this method, which may be useful in some cases, \
                    such as emulating aliases or shims in other version managers.",
                )
                .disable_help_flag(true)
                .arg(
                    arg!([COMMAND]... "The command and its arguments to run")
                        .value_delimiter(' ')
                        .allow_hyphen_values(true)
                        .trailing_var_arg(true),
                ),
        )
        // Will be replaced with an interactive metadata subcommand that allows the same behavior.
        .subcommand(
            Command::new("template")
                .about("Create template metadata for a runtime")
                .long_about("This subcommand simply creates the default metadata for a runtime, nothing more.")
                .arg(arg!(<RUNTIME> "The runtime to create a template for")),
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

    let config_path: String = if let Some(path) = matches.get_one::<String>("config") {
        path.to_string()
    } else {
        ".ver.ron".into()
    };
    let mut config: Option<HashMap<String, String>> = libver::conf::parse(&config_path).ok();

    if matches.subcommand_matches("check").is_some() {
        verify_config!(config);
        let config_data: HashMap<String, String> = config.unwrap();
        let data: HashMap<Runtime, String> = libver::conf::unsafe_collect(config_data);
        let mut should_error: bool = false;
        for (runtime, version) in data.iter() {
            if let Err(e) = runtime.get_safe_version(version.to_string()) {
                should_error = true;
                eprintln!("{}: {}: {}", env!("CARGO_BIN_NAME"), runtime.name, e)
            }
        }
        error_status = if should_error {
            (
                1,
                format!(
                    "Issues above must be resolved to safely continue using {}",
                    env!("CARGO_BIN_NAME")
                ),
                false,
            )
        } else {
            (0, "All runtimes are properly installed".into(), true)
        };
    } else if let Some(matches) = matches.subcommand_matches("switch") {
        if config.is_none() {
            config = Some(HashMap::new());
        }
        let runtime: String = matches.get_one::<String>("RUNTIME").unwrap().to_string();
        let version: String = matches.get_one::<String>("VERSION").unwrap().to_string();
        let mut potential_error: Option<(i32, String, bool)> = None;
        error_status = if matches.get_flag("skip-check") || {
            match Runtime::new(runtime.clone()) {
                Ok(data) => data.get_safe_version(version.to_string()).is_ok(),
                Err(e) => {
                    potential_error = Some((
                        1,
                        format!("Could not find runtime \"{}\": {}", runtime, e),
                        false,
                    ));
                    false
                }
            }
        } {
            let mut config_data: HashMap<String, String> = config.unwrap();
            config_data.insert(runtime.clone(), version.to_string());
            if let Ok(data) = ron::to_string(&config_data) {
                match write(config_path, data) {
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
                }
            } else {
                (
                    1,
                    "Could not safely serialize configuration file".to_string(),
                    false,
                )
            }
        } else if let Some(error) = potential_error {
            error
        } else {
            (
                2,
                format!(
                    "Runtime \"{}\" version {} could not be found",
                    runtime, version
                ),
                false,
            )
        };
    } else if let Some(matches) = matches.subcommand_matches("scope") {
        verify_config!(config);
        let mut args: Vec<String> = Vec::new();
        if let Some(list) = matches.get_many::<String>("COMMAND") {
            for i in list {
                args.push(i.to_string());
            }
        }
        error_status = match conf::collect(config.unwrap()) {
            Ok(config_data) => match exec(args.into(), config_data) {
                Ok(mut cmd) => {
                    match cmd
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .output()
                    {
                        Ok(output) => {
                            if let Some(code) = output.status.code() {
                                (
                                    code,
                                    "Successfully started program, but error was returned".into(),
                                    false,
                                )
                            } else {
                                (
                                    143,
                                    "Successfully started program, but program was interrupted"
                                        .into(),
                                    false,
                                )
                            }
                        }
                        Err(e) => (1, format!("Execution error: {}", e), false),
                    }
                }
                Err(e) => (1, format!("Command error: {}", e), false),
            },
            Err(e) => (1, format!("Configuration error: {}", e), false),
        };
    } else if let Some(matches) = matches.subcommand_matches("template") {
        let runtime: String = matches.get_one::<String>("RUNTIME").unwrap().to_string();
        error_status = match Runtime::get_runtime(runtime.as_str()) {
            Ok(mut buf) => {
                buf.push("meta.ron");
                let template: RuntimeMetadata = RuntimeMetadata::default();
                let template_contents: String =
                    ron::ser::to_string_pretty(&template, ron::ser::PrettyConfig::default())
                        .unwrap();
                match write(buf, template_contents) {
                    Ok(_) => (
                        0,
                        format!("Successfully wrote template for runtime \"{}\"", runtime),
                        true,
                    ),
                    Err(e) => (
                        1,
                        format!(
                            "Error when writing template for runtime \"{}\": {}",
                            runtime, e
                        ),
                        false,
                    ),
                }
            }
            Err(e) => (
                1,
                format!("Could not fetch runtime \"{}\": {}", runtime, e),
                false,
            ),
        };
    }

    if error_status.2 {
        println!("{}: {}", env!("CARGO_BIN_NAME"), error_status.1);
    } else if error_status.0 != 0 {
        eprintln!("{}: {}", env!("CARGO_BIN_NAME"), error_status.1);
    }
    exit(error_status.0);
}
