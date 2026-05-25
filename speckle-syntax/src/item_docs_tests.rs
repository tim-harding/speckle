use syn::parse_str;

use crate::Item;

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
    let item = parse_item("/// A unit struct.\nstruct Foo;");
    insta::assert_yaml_snapshot!(item.docs());
}

#[test]
fn test_docs_multi_line() {
    let item = parse_item("/// First paragraph.\n///\n/// Second paragraph.\nstruct Foo { x: i32 }");
    insta::assert_yaml_snapshot!(item.docs());
}

#[test]
fn test_docs_with_other_attributes() {
    let item = parse_item(
        "/// Documented struct.\n#[derive(Debug)]\n#[speckle]\nstruct Foo { x: i32 }",
    );
    insta::assert_yaml_snapshot!(item.docs());
}

#[test]
fn test_docs_explicit_doc_attribute() {
    let item = parse_item("#[doc = \"Explicit doc string.\"]\nstruct Foo;");
    insta::assert_yaml_snapshot!(item.docs());
}

#[test]
fn test_docs_on_fn() {
    let item = parse_item("/// Returns nothing.\nfn foo() {}");
    insta::assert_yaml_snapshot!(item.docs());
}
