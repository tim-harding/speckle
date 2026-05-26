use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use speckle_db::{DEFAULT_PATH, NewSourceRange, NewSpecification, NewSpeckle, SpeckleDb};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

mod file_patcher;
mod git;

use file_patcher::FilePatcher;
use git::require_clean_repo;

#[derive(Parser)]
#[command(
    name = "cargo-speckle",
    about = "Manage Speckle attributes in Rust source files"
)]
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

struct PendingPatch {
    path: PathBuf,
    patcher: FilePatcher,
    uuids: Vec<String>,
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
    init_ids_with_db(path, Path::new(DEFAULT_PATH))
}

fn init_ids_with_db(
    path: &Path,
    db_path: &Path,
) -> Result<InitIdsSummary, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("path not found: {}", path.display()).into());
    }

    let git = require_clean_repo(path)?;
    let mut pending = Vec::new();
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
        attributes += patched;
        files += 1;
        pending.push(PendingPatch {
            path: source_path,
            patcher,
            uuids,
        });
    }

    if pending.is_empty() {
        return Ok(InitIdsSummary {
            attributes: 0,
            files: 0,
        });
    }

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut db = SpeckleDb::open(db_path)?;
    db.migrate()?;
    let mut tx = db.transaction()?;

    for entry in &pending {
        let abs_path = fs::canonicalize(&entry.path)?;
        let rel_path = abs_path
            .strip_prefix(&git.toplevel)?
            .to_string_lossy()
            .into_owned();
        let items = entry.patcher.find_identified_speckle_items(&entry.uuids);

        for item in items {
            let speckle = tx.insert_speckle(NewSpeckle {
                identifier: item.identifier.clone(),
            })?;
            let source_range = tx.insert_source_range(NewSourceRange {
                commit_hash: git.head.clone(),
                file_path: rel_path.clone(),
                byte_start: item.item_range.byte_start as i64,
                byte_end: item.item_range.byte_end as i64,
            })?;
            tx.insert_specification(NewSpecification {
                id_speckle: speckle.id,
                id_source_range: source_range.id,
                source_text: item.source_text,
            })?;
        }
    }

    for entry in pending {
        entry.patcher.save()?;
    }
    tx.commit()?;

    Ok(InitIdsSummary { attributes, files })
}

fn find_rust_sources(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "rs")
        })
        .map(DirEntry::into_path)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use speckle_db::SpeckleDb;
    use std::process::Command;

    fn init_repo_with_commit(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("git init");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/lib.rs"), "pub fn untouched() {}\n").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "init"])
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .current_dir(dir)
            .output()
            .expect("git commit");
    }

    #[test]
    fn test_init_ids_registers_speckles_in_db() {
        let temp_dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(temp_dir.path());
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            indoc! {"
                #[speckle]
                struct Foo;

                #[speckle]
                fn bar() {}
            "},
        )
        .unwrap();
        Command::new("git")
            .args(["add", "src/lib.rs"])
            .current_dir(temp_dir.path())
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "add speckles"])
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .current_dir(temp_dir.path())
            .output()
            .expect("git commit");

        let db_path = temp_dir.path().join(".speckle/db.sqlite3");
        let summary = init_ids_with_db(&temp_dir.path().join("src"), &db_path).unwrap();
        assert_eq!(summary.attributes, 2);
        assert_eq!(summary.files, 1);

        let db = SpeckleDb::open(&db_path).unwrap();
        let written = fs::read_to_string(temp_dir.path().join("src/lib.rs")).unwrap();
        assert!(written.contains("#[speckle(\""));

        let speckles = (0..2)
            .map(|id| db.get_speckle_by_id(id + 1).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(speckles.len(), 2);
        for speckle in &speckles {
            let specs = db.list_specifications_for_speckle(speckle.id).unwrap();
            assert_eq!(specs.len(), 1);
            let source_range = db.get_source_range_by_id(specs[0].id_source_range).unwrap();
            assert_eq!(source_range.file_path, "src/lib.rs");
            assert_eq!(source_range.commit_hash.len(), 40);
        }
    }
}
