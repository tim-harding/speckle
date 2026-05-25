use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod file_patcher;

use file_patcher::FilePatcher;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser)]
#[command(name = "cargo-speckle", about = "Manage Speckle attributes in Rust source files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Assign UUIDs to bare `#[speckle]` attributes
    InitIds(InitIdsArgs),
}

#[derive(Parser)]
struct InitIdsArgs {
    /// Directory to search for Rust source files
    #[arg(default_value = "src")]
    path: PathBuf,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::InitIds(args) => match init_ids(&args.path) {
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
    }
}

struct InitIdsSummary {
    attributes: usize,
    files: usize,
}

fn init_ids(path: &Path) -> Result<InitIdsSummary, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("path not found: {}", path.display()).into());
    }

    let mut attributes = 0;
    let mut files = 0;

    for source_path in find_rust_sources(path) {
        let mut patcher = FilePatcher::open(&source_path)?;
        let bare_attributes = patcher.find_bare_speckle_attributes();
        if bare_attributes.is_empty() {
            continue;
        }

        let uuids = (0..bare_attributes.len())
            .map(|_| Uuid::new_v4().to_string())
            .collect::<Vec<_>>();
        let patched = patcher.patch_bare_attributes(&uuids)?;
        patcher.save()?;
        attributes += patched;
        files += 1;
    }

    Ok(InitIdsSummary { attributes, files })
}

fn find_rust_sources(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().is_some_and(|extension| extension == "rs")
        })
        .map(DirEntry::into_path)
        .collect()
}
