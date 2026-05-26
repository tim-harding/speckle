use clap::Parser;
use std::process::ExitCode;

mod cli;
mod file_patcher;
mod git;
mod init_ids;
mod sources;
mod sync;

use cli::{Cli, Command};

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::InitIds(args) => init_ids::execute(args),
        Command::Sync(args) => sync::execute(args),
    }
}
