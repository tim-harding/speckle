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

#[derive(Serialize)]
struct AllVariantSpanContents {
    r#static: SpanContentSnapshot,
    r#const: SpanContentSnapshot,
    r#struct: SpanContentSnapshot,
    r#enum: SpanContentSnapshot,
    union: SpanContentSnapshot,
    r#fn: SpanContentSnapshot,
    r#trait: SpanContentSnapshot,
    r#impl: SpanContentSnapshot,
    r#macro: SpanContentSnapshot,
    r#mod: SpanContentSnapshot,
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
fn test_span_content_each_variant() {
    insta::assert_yaml_snapshot!(AllVariantSpanContents {
        r#static: snapshot_span_content("static FOO: i32 = 42;"),
        r#const: snapshot_span_content("const FOO: i32 = 42;"),
        r#struct: snapshot_span_content("struct Foo { x: i32 }"),
        r#enum: snapshot_span_content("enum Foo { A, B }"),
        union: snapshot_span_content("union Foo { f1: u32, f2: f32 }"),
        r#fn: snapshot_span_content("fn foo() { 1 }"),
        r#trait: snapshot_span_content("trait Foo { fn bar(); }"),
        r#impl: snapshot_span_content("impl Foo { fn bar() {} }"),
        r#macro: snapshot_span_content("macro_rules! foo { () => {} }"),
        r#mod: snapshot_span_content("mod foo {}"),
    });
}
