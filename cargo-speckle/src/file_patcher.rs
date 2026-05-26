use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use speckle_syntax::{Item, SourceRange};
use syn::{Attribute, ImplItem, Meta, TraitItem, spanned::Spanned, visit::Visit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BareSpeckleAttribute {
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifiedSpeckleItem {
    pub identifier: String,
    pub item_range: SourceRange,
    pub source_text: String,
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
        for item in &self.file.items {
            visitor.visit_item(item);
        }
        visitor.found
    }

    pub fn find_all_identified_speckle_items(&self) -> Vec<IdentifiedSpeckleItem> {
        let mut visitor = IdentifiedSpeckleVisitor {
            source: &self.source,
            found: Vec::new(),
        };
        for item in &self.file.items {
            visitor.visit_item(item);
        }
        visitor.found
    }

    pub fn find_identified_speckle_items(
        &self,
        identifiers: &[String],
    ) -> Vec<IdentifiedSpeckleItem> {
        let identifiers: std::collections::HashSet<_> = identifiers.iter().collect();
        self.find_all_identified_speckle_items()
            .into_iter()
            .filter(|item| identifiers.contains(&item.identifier))
            .collect()
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

fn is_bare_speckle(attr: &Attribute) -> bool {
    attr.path().is_ident("speckle") && matches!(attr.meta, Meta::Path(_))
}

fn speckle_identifier(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("speckle") {
        return None;
    }

    let list = match &attr.meta {
        Meta::List(list) => list,
        _ => return None,
    };

    if let Ok(lit) = list.parse_args::<syn::LitStr>() {
        return Some(lit.value());
    }

    let mut identifier = None;
    let _ = list.parse_nested_meta(|meta| {
        if !meta.path.is_ident("identifier") {
            return Err(meta.error("expected `identifier`"));
        }
        let value = meta
            .value()?
            .parse::<syn::LitStr>()
            .map_err(|err| meta.error(err))?;
        identifier = Some(value.value());
        Ok(())
    });
    identifier
}

struct BareSpeckleVisitor {
    found: Vec<BareSpeckleAttribute>,
}

impl BareSpeckleVisitor {
    fn check_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if is_bare_speckle(attr) {
                self.found.push(BareSpeckleAttribute {
                    range: SourceRange::from(attr.span()),
                });
            }
        }
    }
}

impl Visit<'_> for BareSpeckleVisitor {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        self.check_attrs(&node.attrs);
        for item in &node.items {
            if let TraitItem::Fn(method) = item {
                self.check_attrs(&method.attrs);
            }
        }
    }

    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        self.check_attrs(&node.attrs);
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                self.check_attrs(&method.attrs);
            }
        }
    }

    fn visit_item_mod(&mut self, node: &syn::ItemMod) {
        self.check_attrs(&node.attrs);
        if let Some((_, items)) = &node.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }

    fn visit_item_const(&mut self, node: &syn::ItemConst) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_static(&mut self, node: &syn::ItemStatic) {
        self.check_attrs(&node.attrs);
    }

    fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
        self.check_attrs(&node.attrs);
    }
}

struct IdentifiedSpeckleVisitor<'source> {
    source: &'source str,
    found: Vec<IdentifiedSpeckleItem>,
}

impl IdentifiedSpeckleVisitor<'_> {
    fn check_item(&mut self, attrs: &[Attribute], item_span: impl Spanned) {
        for attr in attrs {
            if let Some(identifier) = speckle_identifier(attr) {
                let item_range = SourceRange::from(item_span.span());
                let item_source = &self.source[item_range.byte_start..item_range.byte_end];
                let item = syn::parse_str::<Item>(item_source)
                    .expect("item source should parse after successful file parse");
                let source_text = item.to_string();
                self.found.push(IdentifiedSpeckleItem {
                    identifier,
                    item_range,
                    source_text,
                });
                return;
            }
        }
    }
}

impl Visit<'_> for IdentifiedSpeckleVisitor<'_> {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        self.check_item(&node.attrs, node.span());
        for item in &node.items {
            if let TraitItem::Fn(method) = item {
                self.check_item(&method.attrs, method.span());
            }
        }
    }

    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        self.check_item(&node.attrs, node.span());
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                self.check_item(&method.attrs, method.span());
            }
        }
    }

    fn visit_item_mod(&mut self, node: &syn::ItemMod) {
        self.check_item(&node.attrs, node.span());
        if let Some((_, items)) = &node.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }

    fn visit_item_const(&mut self, node: &syn::ItemConst) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_static(&mut self, node: &syn::ItemStatic) {
        self.check_item(&node.attrs, node.span());
    }

    fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
        self.check_item(&node.attrs, node.span());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::{formatdoc, indoc};

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
