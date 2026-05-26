use std::fmt;

use crate::Item;
use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;

impl Item {
    pub fn span_content(&self) -> Span {
        match self {
            Item::Static(item) => item.expr.span(),
            Item::Const(item) => item.expr.span(),
            Item::Struct(item) => item.fields.span(),
            Item::Enum(item) => item.variants.span(),
            Item::Union(item) => item.fields.span(),
            Item::Fn(item) => item.block.span(),
            Item::Trait(item) => item.brace_token.span.join(),
            Item::Impl(item) => item.brace_token.span.join(),
            Item::Macro(item) => item.mac.delimiter.span().join(),
            Item::Mod(item) => {
                let (brace, _) = item
                    .content
                    .as_ref()
                    .expect("file modules are rejected during parsing");
                brace.span.join()
            }
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Static(item) => write!(f, "{}", item.to_token_stream()),
            Item::Const(item) => write!(f, "{}", item.to_token_stream()),
            Item::Struct(item) => write!(f, "{}", item.to_token_stream()),
            Item::Enum(item) => write!(f, "{}", item.to_token_stream()),
            Item::Union(item) => write!(f, "{}", item.to_token_stream()),
            Item::Fn(item) => write!(f, "{}", item.to_token_stream()),
            Item::Trait(item) => write!(f, "{}", item.to_token_stream()),
            Item::Impl(item) => write!(f, "{}", item.to_token_stream()),
            Item::Macro(item) => write!(f, "{}", item.to_token_stream()),
            Item::Mod(item) => write!(f, "{}", item.to_token_stream()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use syn::parse_str;

    fn parse_item(source: &str) -> Item {
        parse_str(source).expect("failed to parse item")
    }

    #[derive(Serialize)]
    struct SpanContentSnapshot {
        byte_start: usize,
        byte_end: usize,
        content: String,
    }

    fn snapshot_span_content(source: &str) -> SpanContentSnapshot {
        let item = parse_item(source);
        let range = item.span_content().byte_range();
        SpanContentSnapshot {
            byte_start: range.start,
            byte_end: range.end,
            content: source[range.start..range.end].to_string(),
        }
    }

    #[test]
    fn test_span_content_static() {
        insta::assert_yaml_snapshot!(snapshot_span_content("static FOO: i32 = 42;"));
    }

    #[test]
    fn test_span_content_const() {
        insta::assert_yaml_snapshot!(snapshot_span_content("const FOO: i32 = 42;"));
    }

    #[test]
    fn test_span_content_struct() {
        insta::assert_yaml_snapshot!(snapshot_span_content("struct Foo { x: i32 }"));
    }

    #[test]
    fn test_span_content_enum() {
        insta::assert_yaml_snapshot!(snapshot_span_content("enum Foo { A, B }"));
    }

    #[test]
    fn test_span_content_union() {
        insta::assert_yaml_snapshot!(snapshot_span_content("union Foo { f1: u32, f2: f32 }"));
    }

    #[test]
    fn test_span_content_fn() {
        insta::assert_yaml_snapshot!(snapshot_span_content("fn foo() { 1 }"));
    }

    #[test]
    fn test_span_content_trait() {
        insta::assert_yaml_snapshot!(snapshot_span_content("trait Foo { fn bar(); }"));
    }

    #[test]
    fn test_span_content_impl() {
        insta::assert_yaml_snapshot!(snapshot_span_content("impl Foo { fn bar() {} }"));
    }

    #[test]
    fn test_span_content_macro() {
        insta::assert_yaml_snapshot!(snapshot_span_content("macro_rules! foo { () => {} }"));
    }

    #[test]
    fn test_span_content_mod() {
        insta::assert_yaml_snapshot!(snapshot_span_content("mod foo {}"));
    }

    #[test]
    fn test_rejects_file_module() {
        match parse_str::<Item>("mod my_file;") {
            Err(err) => assert!(err.to_string().contains("file modules")),
            Ok(_) => panic!("expected file module to be rejected"),
        }
    }

    #[test]
    fn test_display_content_fn() {
        insta::assert_yaml_snapshot!(parse_item("fn foo() { 1 }").to_string());
    }
}
