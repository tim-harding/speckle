use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

pub fn find_rust_sources(path: &Path) -> Vec<PathBuf> {
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
