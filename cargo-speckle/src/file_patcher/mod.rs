mod bare_speckle_visitor;
mod identified_speckle_visitor;
mod patcher;

pub use bare_speckle_visitor::BareSpeckleAttribute;
pub use identified_speckle_visitor::IdentifiedSpeckleItem;
pub use patcher::FilePatcher;

#[derive(thiserror::Error, Debug)]
pub enum FilePatcherError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: std::path::PathBuf,
        source: syn::Error,
    },
    #[error("failed to save {path}: {source}")]
    Save {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("parent directory not found for {0}")]
    ParentNotFound(std::path::PathBuf),
    #[error("uuid count ({uuid_count}) does not match bare attribute count ({attribute_count})")]
    UuidCountMismatch {
        uuid_count: usize,
        attribute_count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::{formatdoc, indoc};
    use std::fs;
    use std::path::Path;

    const EXAMPLE_ID: &str = "cb4cb14c-8e40-495a-b17f-6227b622f4a8";
    const OTHER_ID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

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
        let patcher = patcher_for(indoc! {"
            #[speckle]
            struct Foo;
        "});
        let bare_attributes = patcher.find_bare_speckle_attributes();
        assert_eq!(bare_attributes.len(), 1);
        assert_eq!(bare_attributes[0].range.byte_start, 0);
        assert_eq!(bare_attributes[0].range.byte_end, 10);
    }

    #[test]
    fn test_find_identified_speckle_items_after_patch() {
        let mut patcher = patcher_for(indoc! {"
            #[speckle]
            struct Foo;

            #[speckle]
            mod bar {
                #[speckle]
                fn baz() {}
            }
        "});
        patcher
            .patch_bare_attributes(&[
                EXAMPLE_ID.to_string(),
                OTHER_ID.to_string(),
                "cccccccc-dddd-eeee-ffff-000000000000".to_string(),
            ])
            .unwrap();

        let items = patcher.find_identified_speckle_items(&[
            EXAMPLE_ID.to_string(),
            OTHER_ID.to_string(),
            "cccccccc-dddd-eeee-ffff-000000000000".to_string(),
        ]);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].identifier, EXAMPLE_ID);
        assert!(items[0].item_range.byte_end > items[0].item_range.byte_start);
    }

    #[test]
    fn test_patch_single_bare_attribute() {
        insta::assert_snapshot!(patch(
            indoc! {"
                #[speckle]
                struct Foo;
            "},
            &[EXAMPLE_ID]
        ));
    }

    #[test]
    fn test_patch_multiple_bare_attributes() {
        insta::assert_snapshot!(patch(
            indoc! {"
                #[speckle]
                struct Foo;

                #[speckle]
                fn bar() {}
            "},
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_patch_bare_attribute_with_other_attributes() {
        insta::assert_snapshot!(patch(
            indoc! {"
                #[derive(Debug)]
                #[speckle]
                struct Foo { x: i32 }
            "},
            &[EXAMPLE_ID]
        ));
    }

    #[test]
    fn test_patch_nested_mod_fn() {
        insta::assert_snapshot!(patch(
            indoc! {"
                #[speckle]
                mod foo {
                    #[speckle]
                    fn bar() {}
                }
            "},
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_patch_trait_method() {
        insta::assert_snapshot!(patch(
            indoc! {"
                #[speckle]
                trait Foo {
                    #[speckle]
                    fn bar(&self);
                }
            "},
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_patch_impl_method() {
        insta::assert_snapshot!(patch(
            indoc! {"
                struct Foo;

                #[speckle]
                impl Foo {
                    #[speckle]
                    fn bar(&self) {}
                }
            "},
            &[EXAMPLE_ID, OTHER_ID]
        ));
    }

    #[test]
    fn test_skip_existing_positional_id() {
        let source = formatdoc! {"
            #[speckle(\"{EXAMPLE_ID}\")]
            struct Foo;
        "};
        let patcher = patcher_for(&source);
        assert!(patcher.find_bare_speckle_attributes().is_empty());
    }

    #[test]
    fn test_skip_existing_named_identifier() {
        let source = formatdoc! {"
            #[speckle(identifier = \"{EXAMPLE_ID}\")]
            struct Foo;
        "};
        let patcher = patcher_for(&source);
        assert!(patcher.find_bare_speckle_attributes().is_empty());
    }

    #[test]
    fn test_ignore_speckle_derive() {
        let patcher = patcher_for(indoc! {"
            #[speckle_derive(Foo)]
            struct Bar;
        "});
        assert!(patcher.find_bare_speckle_attributes().is_empty());
    }

    #[test]
    fn test_uuid_count_mismatch() {
        let mut patcher = patcher_for(indoc! {"
            #[speckle]
            struct Foo;

            #[speckle]
            fn bar() {}
        "});
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
            indoc! {"
                #[speckle]
                struct Foo;
            "},
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
            formatdoc! {"
                #[speckle(\"{EXAMPLE_ID}\")]
                struct Foo;
            "}
        );
    }
}
