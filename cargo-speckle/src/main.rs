use std::process::ExitCode;

use clap::Parser;

mod cli;
mod file_patcher;
mod git;
mod init_ids;
mod sources;
mod sync;

use cli::{Cli, Command};

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::InitIds(args) => match init_ids::run(&args.path) {
            Ok(summary) => {
                if summary.attributes == 0 {
                    println!("no bare #[speckle] attributes found");
                } else {
                    println!(
                        "patched {} attribute{} in {} file{}",
                        summary.attributes,
                        if summary.attributes == 1 { "" } else { "s" },
                        summary.files,
                        if summary.files == 1 { "" } else { "s" },
                    );
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        Command::Sync(args) => match sync::run(&args.path) {
            Ok(summary) => {
                if summary.speckles == 0 {
                    println!("no identified #[speckle] attributes found");
                } else {
                    println!(
                        "registered {} speckle{} from {} file{}",
                        summary.speckles,
                        if summary.speckles == 1 { "" } else { "s" },
                        summary.files,
                        if summary.files == 1 { "" } else { "s" },
                    );
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
    }
}
