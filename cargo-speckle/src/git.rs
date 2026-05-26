use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitContext {
    pub toplevel: PathBuf,
    pub head: String,
}

#[derive(thiserror::Error, Debug)]
pub enum GitError {
    #[error("not a git repository")]
    NotARepository,
    #[error("bare git repositories are not supported")]
    BareRepository,
    #[error("no commits yet")]
    NoCommits,
    #[error("working tree has uncommitted changes; commit or stash before running init-ids")]
    UncommittedChanges,
    #[error("{0}")]
    Gix(String),
}

pub fn require_clean_repo(cwd: &Path) -> Result<GitContext, GitError> {
    let repo = gix::discover(cwd).map_err(|_| GitError::NotARepository)?;

    let toplevel = repo
        .workdir()
        .ok_or(GitError::BareRepository)?
        .to_path_buf();

    let head = repo
        .head()
        .map_err(|error| GitError::Gix(error.to_string()))?;
    if head.is_unborn() {
        return Err(GitError::NoCommits);
    }
    let head = repo
        .head_commit()
        .map_err(|error| GitError::Gix(error.to_string()))?;
    let head = head.id().to_hex().to_string();

    let has_changes = repo
        .status(gix::progress::Discard)
        .map_err(|error| GitError::Gix(error.to_string()))?
        .untracked_files(gix::status::UntrackedFiles::None)
        .into_iter([])
        .map_err(|error| GitError::Gix(error.to_string()))?
        .next()
        .is_some();
    if has_changes {
        return Err(GitError::UncommittedChanges);
    }

    Ok(GitContext { toplevel, head })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn init_repo_with_commit(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("git init");
        fs::write(dir.join("README"), "hello").unwrap();
        Command::new("git")
            .args(["add", "README"])
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
    fn test_require_clean_repo_succeeds() {
        let temp_dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(temp_dir.path());

        let context = require_clean_repo(temp_dir.path()).unwrap();
        assert_eq!(context.toplevel, temp_dir.path());
        assert_eq!(context.head.len(), 40);
    }

    #[test]
    fn test_require_clean_repo_rejects_uncommitted_changes() {
        let temp_dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(temp_dir.path());
        fs::write(temp_dir.path().join("README"), "changed").unwrap();

        let error = require_clean_repo(temp_dir.path()).unwrap_err();
        assert!(matches!(error, GitError::UncommittedChanges));
    }

    #[test]
    fn test_require_clean_repo_rejects_unborn_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("git init");

        let error = require_clean_repo(temp_dir.path()).unwrap_err();
        assert!(matches!(error, GitError::NoCommits));
    }
}
