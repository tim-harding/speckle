use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use proc_macro2::Span;

use super::bare_speckle_visitor;
use super::identified_speckle_visitor;
use super::{BareSpeckleAttribute, FilePatcherError, IdentifiedSpeckleItem};

pub struct FilePatcher {
    path: PathBuf,
    source: String,
    file: syn::File,
}

impl FilePatcher {
    pub fn open(path: &Path) -> Result<Self, FilePatcherError> {
        let path = path.to_path_buf();
        let source = fs::read_to_string(&path).map_err(|source| FilePatcherError::Read {
            path: path.clone(),
            source,
        })?;
        let file = syn::parse_file(&source).map_err(|source| FilePatcherError::Parse {
            path: path.clone(),
            source,
        })?;
        Ok(Self { path, source, file })
    }

    #[cfg(test)]
    pub fn from_source(path: &Path, source: String) -> Result<Self, FilePatcherError> {
        let path = path.to_path_buf();
        let file = syn::parse_file(&source).map_err(|source| FilePatcherError::Parse {
            path: path.clone(),
            source,
        })?;
        Ok(Self { path, source, file })
    }

    #[cfg(test)]
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn find_bare_speckle_attributes(&self) -> Vec<BareSpeckleAttribute> {
        bare_speckle_visitor::find_bare_speckle_attributes(&self.file)
    }

    pub fn find_all_identified_speckle_items(&self) -> Vec<IdentifiedSpeckleItem> {
        identified_speckle_visitor::find_identified_speckle_items(&self.source, &self.file)
    }

    pub fn patch_bare_attributes(&mut self, uuids: &[String]) -> Result<usize, FilePatcherError> {
        let bare_attributes = self.find_bare_speckle_attributes();
        if bare_attributes.len() != uuids.len() {
            return Err(FilePatcherError::UuidCountMismatch {
                uuid_count: uuids.len(),
                attribute_count: bare_attributes.len(),
            });
        }

        let mut replacements: Vec<(Span, String)> = bare_attributes
            .into_iter()
            .zip(uuids.iter())
            .map(|(bare_attribute, uuid)| {
                (bare_attribute.range, format!(r#"#[speckle("{uuid}")]"#))
            })
            .collect();

        replacements
            .sort_by(|left, right| right.0.byte_range().start.cmp(&left.0.byte_range().start));

        let count = replacements.len();
        for (range, replacement) in replacements {
            let bytes = range.byte_range();
            self.source
                .replace_range(bytes.start..bytes.end, &replacement);
        }

        self.file = syn::parse_file(&self.source).map_err(|source| FilePatcherError::Parse {
            path: self.path.clone(),
            source,
        })?;

        Ok(count)
    }

    pub fn save(&self) -> Result<(), FilePatcherError> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| FilePatcherError::ParentNotFound(self.path.clone()))?;

        let mut temp_file = tempfile::Builder::new()
            .suffix(".speckle.tmp")
            .tempfile_in(parent)
            .map_err(|source| FilePatcherError::Save {
                path: self.path.clone(),
                source,
            })?;

        write!(temp_file, "{}", self.source).map_err(|source| FilePatcherError::Save {
            path: self.path.clone(),
            source,
        })?;
        temp_file.flush().map_err(|source| FilePatcherError::Save {
            path: self.path.clone(),
            source,
        })?;

        fs::rename(temp_file.path(), &self.path).map_err(|source| FilePatcherError::Save {
            path: self.path.clone(),
            source,
        })?;

        Ok(())
    }
}
