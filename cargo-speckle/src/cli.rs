use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cargo-speckle",
    about = "Manage Speckle attributes in Rust source files"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Assign UUIDs to bare `#[speckle]` attributes
    InitIds(InitIdsArgs),
    /// Register identified `#[speckle]` attributes in the database
    Sync(SyncArgs),
}

#[derive(Parser)]
pub struct InitIdsArgs {
    /// Directory to search for Rust source files
    #[arg(default_value = "src")]
    pub path: PathBuf,
}

#[derive(Parser)]
pub struct SyncArgs {
    /// Directory to search for Rust source files
    #[arg(default_value = "src")]
    pub path: PathBuf,
}
