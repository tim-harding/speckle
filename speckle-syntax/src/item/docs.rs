use crate::Item;
use syn::{Expr, ExprLit, Lit, MetaNameValue};

impl Item {
    pub fn docs(&self) -> String {
        let docs: Vec<_> = self
            .attributes()
            .iter()
            .filter_map(|attr| match &attr.meta {
                syn::Meta::NameValue(MetaNameValue {
                    path,
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }),
                    ..
                }) if path.is_ident("doc") => Some(s.value()),
                _ => None,
            })
            .collect();
        docs.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    fn parse_item(source: &str) -> Item {
        parse_str(source).expect("failed to parse item")
    }

    #[test]
    fn test_docs_none() {
        let item = parse_item("struct Foo;");
        insta::assert_yaml_snapshot!(item.docs());
    }

    #[test]
    fn test_docs_single_line() {
        let item = parse_item(
            r#"/// A unit struct.
struct Foo;"#,
        );
        insta::assert_yaml_snapshot!(item.docs());
    }

    #[test]
    fn test_docs_multi_line() {
        let item = parse_item(
            r#"/// First paragraph.
///
/// Second paragraph.
struct Foo { x: i32 }"#,
        );
        insta::assert_yaml_snapshot!(item.docs());
    }

    #[test]
    fn test_docs_with_other_attributes() {
        let item = parse_item(
            r#"/// Documented struct.
#[derive(Debug)]
#[speckle]
struct Foo { x: i32 }"#,
        );
        insta::assert_yaml_snapshot!(item.docs());
    }

    #[test]
    fn test_docs_explicit_doc_attribute() {
        let item = parse_item(
            r#"#[doc = "Explicit doc string."]
struct Foo;"#,
        );
        insta::assert_yaml_snapshot!(item.docs());
    }

    #[test]
    fn test_docs_on_fn() {
        let item = parse_item(
            r#"/// Returns nothing.
fn foo() {}"#,
        );
        insta::assert_yaml_snapshot!(item.docs());
    }
}
