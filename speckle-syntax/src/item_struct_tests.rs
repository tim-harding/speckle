use proc_macro2::Span;
use serde::Serialize;
use syn::parse_str;

use crate::{Item, SourceRange};

#[derive(Serialize)]
struct SpanSnapshot<'a> {
    source: &'a str,
    byte_start: usize,
    byte_end: usize,
    snippet: &'a str,
}

fn span_snapshot(source: &str, span: Span) -> SpanSnapshot<'_> {
    let range = SourceRange::from(span);
    SpanSnapshot {
        source,
        byte_start: range.byte_start,
        byte_end: range.byte_end,
        snippet: &source[range.byte_start..range.byte_end],
    }
}

fn parse_struct(source: &str) -> Item {
    let item: Item = parse_str(source).expect("failed to parse item");
    match item {
        item @ Item::Struct(_) => item,
        _ => panic!("expected struct"),
    }
}

#[test]
fn test_struct_unit_span_full() {
    let source = "struct Foo;";
    let item = parse_struct(source);
    insta::assert_yaml_snapshot!(span_snapshot(source, item.span_full()));
}

#[test]
fn test_struct_named_fields_span_full() {
    let source = "struct Foo { x: i32, y: String }";
    let item = parse_struct(source);
    insta::assert_yaml_snapshot!(span_snapshot(source, item.span_full()));
}

#[test]
fn test_struct_tuple_span_full() {
    let source = "struct Foo(i32, String);";
    let item = parse_struct(source);
    insta::assert_yaml_snapshot!(span_snapshot(source, item.span_full()));
}

#[test]
fn test_struct_with_attributes_span_full() {
    let source = "#[derive(Debug, Clone)]\n#[speckle]\nstruct Foo { x: i32 }";
    let item = parse_struct(source);
    insta::assert_yaml_snapshot!(span_snapshot(source, item.span_full()));
}

#[test]
fn test_struct_generic_span_full() {
    let source = "pub struct Foo<T: Clone> { x: T }";
    let item = parse_struct(source);
    insta::assert_yaml_snapshot!(span_snapshot(source, item.span_full()));
}
