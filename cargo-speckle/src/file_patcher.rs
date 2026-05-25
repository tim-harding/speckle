use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use speckle_syntax::SourceRange;
use syn::{Attribute, Meta, spanned::Spanned, visit::Visit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BareSpeckleAttribute {
    pub range: SourceRange,
}

pub struct FilePatcher {
    path: PathBuf,
    source: String,
    file: syn::File,
}

#[derive(thiserror::Error, Debug)]
pub enum FilePatcherError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    Parse { path: PathBuf, source: syn::Error },
    #[error("failed to save {path}: {source}")]
    Save {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parent directory not found for {0}")]
    ParentNotFound(PathBuf),
    #[error("uuid count ({uuid_count}) does not match bare attribute count ({attribute_count})")]
    UuidCountMismatch {
        uuid_count: usize,
        attribute_count: usize,
    },
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

    pub fn find_bare_speckle_attributes(&self) -> Vec<BareSpeckleAttribute> {
        let mut visitor = BareSpeckleVisitor { found: Vec::new() };
        visitor.visit_file(&self.file);
        visitor.found
    }

    pub fn patch_bare_attributes(&mut self, uuids: &[String]) -> Result<usize, FilePatcherError> {
        let bare_attributes = self.find_bare_speckle_attributes();
        if bare_attributes.len() != uuids.len() {
            return Err(FilePatcherError::UuidCountMismatch {
                uuid_count: uuids.len(),
                attribute_count: bare_attributes.len(),
            });
        }

        let mut replacements: Vec<(SourceRange, String)> = bare_attributes
            .into_iter()
            .zip(uuids.iter())
            .map(|(bare_attribute, uuid)| {
                (bare_attribute.range, format!(r#"#[speckle("{uuid}")]"#))
            })
            .collect();

        replacements.sort_by(|left, right| right.0.byte_start.cmp(&left.0.byte_start));

        let count = replacements.len();
        for (range, replacement) in replacements {
            self.source
                .replace_range(range.byte_start..range.byte_end, &replacement);
        }

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

struct BareSpeckleVisitor {
    found: Vec<BareSpeckleAttribute>,
}

impl Visit<'_> for BareSpeckleVisitor {
    fn visit_attribute(&mut self, attr: &Attribute) {
        if attr.path().is_ident("speckle") && matches!(attr.meta, Meta::Path(_)) {
            self.found.push(BareSpeckleAttribute {
                range: SourceRange::from(attr.span()),
            });
        }
        syn::visit::visit_attribute(self, attr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_ID: &str = "cb4cb14c-8e40-495a-b17f-6227b622f4a8";
    const OTHER_ID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

    impl FilePatcher {
        pub fn from_source(path: &Path, source: String) -> Result<Self, FilePatcherError> {
            let path = path.to_path_buf();
            let file = syn::parse_file(&source).map_err(|source| FilePatcherError::Parse {
                path: path.clone(),
                source,
            })?;
            Ok(Self { path, source, file })
        }

        pub fn source(&self) -> &str {
            &self.source
        }
    }

    fn patcher_for(source: &str) -> FilePatcher {
        FilePatcher::from_source(Path::new("test.rs"), source.to_string()).unwrap()
    }

    fn patch(source: &str, uuids: &[&str]) -> String {
        let mut patcher = patcher_for(source);
        patcher
            .patch_bare_attributes(
                &uuids
                    .iter()
                    .map(|uuid| (*uuid).to_string())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        patcher.source().to_string()
    }

    #[test]
    fn test_find_bare_speckle_attribute_on_struct() {
        let patcher = patcher_for(
            r#"#[speckle]
struct Foo;"#,
        );
        let bare_attributes = patcher.find_bare_speckle_attributes();
        assert_eq!(bare_attributes.len(), 1);
        assert_eq!(bare_attributes[0].range.byte_start, 0);
        assert_eq!(bare_attributes[0].range.byte_end, 10);
    }

    #[test]
    fn test_patch_single_bare_attribute() {
        insta::assert_snapshot!(patch(
            r#"#[speckle]
struct Foo;"#,
            &[EXAMPLE_ID]
        ));
    }

    #[test]
    fn test_patch_multiple_bare_attributes() {
        insta::assert_snapshot!(patch(
            r#"#[speckle]
struct Foo;

#[speckle]
fn bar() {}"#,
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_patch_bare_attribute_with_other_attributes() {
        insta::assert_snapshot!(patch(
            r#"#[derive(Debug)]
#[speckle]
struct Foo { x: i32 }"#,
            &[EXAMPLE_ID]
        ));
    }

    #[test]
    fn test_patch_nested_mod_fn() {
        insta::assert_snapshot!(patch(
            r#"#[speckle]
mod foo {
    #[speckle]
    fn bar() {}
}"#,
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_patch_trait_method() {
        insta::assert_snapshot!(patch(
            r#"#[speckle]
trait Foo {
    #[speckle]
    fn bar(&self);
}"#,
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_patch_impl_method() {
        insta::assert_snapshot!(patch(
            r#"struct Foo;

#[speckle]
impl Foo {
    #[speckle]
    fn bar(&self) {}
}"#,
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_skip_existing_positional_id() {
        let source = format!(
            r#"#[speckle("{EXAMPLE_ID}")]
struct Foo;"#
        );
        let patcher = patcher_for(&source);
        assert!(patcher.find_bare_speckle_attributes().is_empty());
    }

    #[test]
    fn test_skip_existing_named_identifier() {
        let source = format!(
            r#"#[speckle(identifier = "{EXAMPLE_ID}")]
struct Foo;"#
        );
        let patcher = patcher_for(&source);
        assert!(patcher.find_bare_speckle_attributes().is_empty());
    }

    #[test]
    fn test_ignore_speckle_derive() {
        let patcher = patcher_for(
            r#"#[speckle_derive(Foo)]
struct Bar;"#,
        );
        assert!(patcher.find_bare_speckle_attributes().is_empty());
    }

    #[test]
    fn test_uuid_count_mismatch() {
        let mut patcher = patcher_for(
            r#"#[speckle]
struct Foo;

#[speckle]
fn bar() {}"#,
        );
        let error = patcher
            .patch_bare_attributes(&[EXAMPLE_ID.to_string()])
            .unwrap_err();
        assert!(matches!(
            error,
            FilePatcherError::UuidCountMismatch {
                uuid_count: 1,
                attribute_count: 2,
            }
        ));
    }

    #[test]
    fn test_save_writes_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("lib.rs");
        fs::write(
            &path,
            r#"#[speckle]
struct Foo;"#,
        )
        .unwrap();

        let mut patcher = FilePatcher::open(&path).unwrap();
        patcher
            .patch_bare_attributes(&[EXAMPLE_ID.to_string()])
            .unwrap();
        patcher.save().unwrap();

        let written = fs::read_to_string(path).unwrap();
        assert_eq!(
            written,
            format!(
                r#"#[speckle("{EXAMPLE_ID}")]
struct Foo;"#
            )
        );
    }
}
