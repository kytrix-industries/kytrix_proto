//! Repository tasks for `kytrix_proto`.
//!
//! Usage: `cargo xtask <command>` (the alias lives in `.cargo/config.toml`).

mod generate;

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return ExitCode::SUCCESS;
    };
    let rest: Vec<String> = args.collect();

    match command.as_str() {
        "gen" => match generate::run(&rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("error: {err}");
                ExitCode::FAILURE
            }
        },
        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("error: unknown command `{other}`");
            eprintln!();
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!(
        "\
Usage: cargo xtask <COMMAND>

Commands:
  gen             Regenerate the Rust protobuf code committed under `src/`
  gen --check     Verify that `src/` matches the protos without writing anything
  help            Print this message"
    );
}
