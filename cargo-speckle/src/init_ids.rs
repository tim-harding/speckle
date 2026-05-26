use std::path::Path;
use std::process::ExitCode;

use uuid::Uuid;

use crate::cli::InitIdsArgs;
use crate::file_patcher::FilePatcher;
use crate::sources::find_rust_sources;

pub struct InitIdsSummary {
    pub attributes: usize,
    pub files: usize,
}

pub fn execute(args: InitIdsArgs) -> ExitCode {
    match run(&args.path) {
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
    }
}

pub(crate) fn run(path: &Path) -> Result<InitIdsSummary, Box<dyn std::error::Error>> {
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
        attributes += patcher.patch_bare_attributes(&uuids)?;
        patcher.save()?;
        files += 1;
    }

    Ok(InitIdsSummary { attributes, files })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
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
    fn test_init_ids_patches_without_clean_repo() {
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

        let summary = run(&temp_dir.path().join("src")).unwrap();
        assert_eq!(summary.attributes, 2);
        assert_eq!(summary.files, 1);

        let written = fs::read_to_string(temp_dir.path().join("src/lib.rs")).unwrap();
        assert!(written.contains("#[speckle(\""));
    }
}
