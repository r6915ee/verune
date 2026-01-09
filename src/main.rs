use clap::{Arg, ArgAction, ArgMatches, Command, arg, command, value_parser};
use libver::*;
use std::{
    collections::HashMap,
    env,
    fs::write,
    path::PathBuf,
    process::{Stdio, exit},
};

fn handle_commands() -> ArgMatches {
    command!()
        .arg(
            arg!(-c --config <FILE> "The configuration to use")
                .required(false)
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            arg!(-r --replace "Replace an entry in the configuration for this session")
                .required(false)
                .action(ArgAction::Append)
                .global(true)
                .value_names(["RUNTIME", "VERSION"])
        )
        .arg(
            arg!(-o --overlay <FILE> "Overlay a configuration on top of the current one")
                .required(false)
                .action(ArgAction::Append)
                .value_parser(value_parser!(PathBuf))
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
            Command::new("apply")
                .about("Apply a change to the configuration")
                .long_about(
                    "This subcommand, by default, will switch the version of a runtime in the \
                    configuration file. Before writing, however, it will check that **every** \
                    version set is installed; this behavior may be avoided by using the --skip-check \
                    flag.\n\n\
                    Note that this subcommand by default will operate on a copy of the configuration \
                    made before applying any overlays. To apply the changes made by overlays as well, \
                    use the --full flag.",
                )
                .arg(
                    Arg::new("skip-check")
                        .short('u')
                        .long("skip-check")
                        .help("Avoid validating the versions' installation")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    arg!(-f --full "Switch using the full configuration, which includes overlays")
                )
                .arg(arg!([RUNTIME] "The runtime to switch").required_unless_present("full"))
                .arg(arg!([VERSION] "The version to switch to").required_unless_present("full"))
                .visible_alias("switch"),
        )
        .subcommand(
            Command::new("scope")
                .about("Run a command within a scope")
                .long_about(
                    "This subcommand creates a child process with a modified environment. \
                    This environment inherits from the current program, but prepends the \
                    $PATH environment variable with both the current version directory of \
                    each runtime and its search paths, alongside setting the $VER_SCOPE \
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
        "No subcommand was passed to verune; for a list of subcommands, please use \"verune help\""
            .into(),
        false,
    );

    let config_path: PathBuf = if let Some(path) = matches.get_one::<PathBuf>("config") {
        path.to_path_buf()
    } else if let Ok(file) = env::var("VER_CONFIG") {
        file.into()
    } else {
        ".ver.ron".into()
    };
    let mut config: Option<HashMap<String, String>> = conf::parse(&config_path).ok();
    let config_copy: Option<HashMap<String, String>> = if let Some(switch) =
        matches.subcommand_matches("apply")
        && !switch.get_flag("full")
    {
        config.clone()
    } else {
        None
    };

    macro_rules! config_merge {
        ($x: expr) => {
            let config_data: &mut HashMap<String, String> = config.as_mut().unwrap();
            for i in $x {
                if let Ok(parsed) = conf::parse(i) {
                    for (runtime, version) in parsed.iter() {
                        config_data.insert(runtime.to_string(), version.to_string());
                    }
                }
            }
        };
    }

    if let Ok(paths_string) = env::var("VERUNE_OVERLAYS") {
        let paths: Vec<&str> = paths_string
            .split(if cfg!(windows) { ";" } else { ":" })
            .collect();
        if config.is_none() {
            config = Some(HashMap::with_capacity(paths.len()));
        }
        config_merge!(paths);
    }

    if let Some(data) = matches.get_occurrences::<PathBuf>("overlay") {
        let configs: Vec<PathBuf> = data.map(Iterator::collect).collect();
        if config.is_none() {
            config = Some(HashMap::with_capacity(configs.len()));
        }
        config_merge!(configs);
    }

    if let Some(data) = matches.get_occurrences::<String>("replace") {
        let replacements: Vec<Vec<&String>> = data.map(Iterator::collect).collect();
        if config.is_none() {
            config = Some(HashMap::with_capacity(replacements.len()));
        }
        let config_data: &mut HashMap<String, String> = config.as_mut().unwrap();
        for i in &replacements {
            config_data.insert(i[0].to_string(), i[1].to_string());
        }
    }

    if matches.subcommand_matches("check").is_some() {
        verify_config!(config);
        let config_data: HashMap<String, String> = config.unwrap();
        let mut should_error: bool = false;
        error_status = match conf::collect(config_data) {
            Ok(data) => {
                for (runtime, version) in data.iter() {
                    if let Err(e) = runtime.get_version_search_paths(version) {
                        should_error = true;
                        eprintln!("{}: {}: {}", env!("CARGO_BIN_NAME"), runtime.name, e)
                    }
                }
                if should_error {
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
                }
            }
            Err(e) => (1, format!("Configuration parsing error: {}", e), false),
        };
    } else if let Some(matches) = matches.subcommand_matches("apply") {
        let mut config: HashMap<String, String> = if let Some(copy) = config_copy {
            copy
        } else {
            config.unwrap_or_default()
        };

        error_status = 'errorable: {
            if let Some(rt) = matches.get_one::<String>("RUNTIME").map(|x| x.to_string())
                && let Some(ver) = matches.get_one::<String>("VERSION").map(|x| x.to_string())
            {
                config.insert(rt, ver);
            }

            if !matches.get_flag("skip-check") {
                let mut runtime_errors: String = String::new();

                macro_rules! conditional_expand {
                    {true, $i: block} => {$i};
                    {false, $i: block} => {};
                }

                macro_rules! runtime_check {
                    ($runt: expr, $ver: expr, $nesc: tt) => {
                        match Runtime::new($runt) {
                            Ok(rt) => {
                                if let Err(e) = rt.get_safe_version($ver) {
                                    conditional_expand!($nesc, { runtime_errors.push('\n') });
                                    runtime_errors.push_str(
                                        format!("{}: {}: {}", env!("CARGO_BIN_NAME"), $runt, e)
                                            .as_str(),
                                    );
                                }
                            }
                            Err(e) => {
                                break 'errorable (
                                    2,
                                    format!("Could not find runtime \"{}\": {}", $runt, e),
                                    false,
                                );
                            }
                        }
                    };
                }
                let mut iter = config.iter();
                if let Some((runtime, version)) = iter.next() {
                    runtime_check!(runtime, version, false);
                }
                for (runtime, version) in iter {
                    runtime_check!(runtime, version, true);
                }
                if !runtime_errors.is_empty() {
                    let rt_err_cl: String = runtime_errors.clone();
                    runtime_errors.retain(|c| {
                        if c == '\n'
                            && let Some(index) = rt_err_cl.find(c)
                            && index == 0
                        {
                            return false;
                        }
                        true
                    });
                    eprintln!("{}", runtime_errors);
                    break 'errorable (
                        1,
                        "Issues above must be resolved in order to apply configuration changes"
                            .into(),
                        false,
                    );
                }
            }

            if let Ok(data) = ron::to_string(&config) {
                match write(config_path, data) {
                    Ok(_) => (0, "Successfully switched configuration".into(), true),
                    Err(e) => (
                        1,
                        format!("Could not write to configuration file: {}", e),
                        false,
                    ),
                }
            } else {
                (
                    1,
                    "Could not safely serialize configuration into RON format".into(),
                    false,
                )
            }
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
            Ok(config_data) => match exec(args, config_data) {
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
        error_status = match Runtime::get_runtime(&runtime) {
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
