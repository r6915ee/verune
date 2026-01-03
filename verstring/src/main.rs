use clap::{ArgAction, ArgMatches, Command, arg, command};
use std::process::exit;

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
        .get_matches()
}

fn main() {
    let matches: ArgMatches = handle_commands();
    let error_status: (i32, &str, bool) = (
        1,
        "No subcommand was passed to verstring; for a list of subcommands, please use \"verstring help\"",
        false,
    );

    if matches.subcommand_matches("check").is_some() {
        todo!("Unimplemented command")
    }

    if error_status.2 {
        println!("verstring: {}", error_status.1);
    } else if error_status.0 != 0 {
        eprintln!("verstring: {}", error_status.1);
    }
    exit(error_status.0);
}
