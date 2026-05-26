use std::fs;
use std::path::Path;
use std::process::ExitCode;

use speckle_db::{DEFAULT_PATH, NewSourceRange, NewSpecification, NewSpeckle, SpeckleDb};

use crate::cli::SyncArgs;
use crate::file_patcher::FilePatcher;
use crate::git::require_clean_repo;
use crate::sources::find_rust_sources;

pub struct SyncSummary {
    pub speckles: usize,
    pub files: usize,
}

pub fn execute(args: SyncArgs) -> ExitCode {
    match run(&args.path) {
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
    }
}

pub(crate) fn run(path: &Path) -> Result<SyncSummary, Box<dyn std::error::Error>> {
    run_with_db(path, Path::new(DEFAULT_PATH))
}

pub(crate) fn run_with_db(
    path: &Path,
    db_path: &Path,
) -> Result<SyncSummary, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("path not found: {}", path.display()).into());
    }

    let git = require_clean_repo(path)?;
    let mut speckles = 0;
    let mut files = 0;

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut db = SpeckleDb::open(db_path)?;
    db.migrate()?;
    let mut tx = db.transaction()?;

    for source_path in find_rust_sources(path) {
        let patcher = FilePatcher::open(&source_path)?;
        let items = patcher.find_all_identified_speckle_items();
        if items.is_empty() {
            continue;
        }

        let abs_path = fs::canonicalize(&source_path)?;
        let rel_path = abs_path
            .strip_prefix(&git.toplevel)?
            .to_string_lossy()
            .into_owned();

        for item in items {
            let speckle = tx.insert_speckle(NewSpeckle {
                identifier: item.identifier.clone(),
            })?;
            let source_range = tx.insert_source_range(NewSourceRange {
                commit_hash: git.head.clone(),
                file_path: rel_path.clone(),
                byte_start: item.item_range.byte_range().start as i64,
                byte_end: item.item_range.byte_range().end as i64,
            })?;
            tx.insert_specification(NewSpecification {
                id_speckle: speckle.id,
                id_source_range: source_range.id,
                source_text: item.source_text,
            })?;
            speckles += 1;
        }
        files += 1;
    }

    tx.commit()?;

    Ok(SyncSummary { speckles, files })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use speckle_db::SpeckleDb;
    use std::fs;
    use std::path::Path;
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
    fn test_sync_registers_speckles_in_db() {
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
        crate::init_ids::run(&temp_dir.path().join("src")).unwrap();
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
        let summary = run_with_db(&temp_dir.path().join("src"), &db_path).unwrap();
        assert_eq!(summary.speckles, 2);
        assert_eq!(summary.files, 1);

        let db = SpeckleDb::open(&db_path).unwrap();
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
