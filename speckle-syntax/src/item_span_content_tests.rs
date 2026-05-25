use serde::Serialize;
use syn::parse_str;

use crate::{Item, SourceRange};

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
    let range = SourceRange::from(item.span_content());
    SpanContentSnapshot {
        byte_start: range.byte_start,
        byte_end: range.byte_end,
        content: source[range.byte_start..range.byte_end].to_string(),
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
