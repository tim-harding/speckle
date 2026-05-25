use crate::{
    Item, SourceRange, SyntaxError,
    speckle_attribute::{SpeckleAttributeArgument, SpeckleAttributeError},
};
use serde::Serialize;
use syn::parse_str;

const EXAMPLE_ID: &str = "cb4cb14c-8e40-495a-b17f-6227b622f4a8";

fn parse_item(source: &str) -> Item {
    parse_str(source).expect("failed to parse item")
}

#[derive(Serialize)]
struct SpeckleAttributeSnapshot {
    byte_start: usize,
    byte_end: usize,
    arguments: Vec<String>,
}

fn snapshot_speckle_attribute(source: &str) -> SpeckleAttributeSnapshot {
    let item = parse_item(source);
    let attribute = item
        .speckle_attribute()
        .expect("expected item to have a #[speckle] attribute");
    let range = SourceRange::from(attribute.span);
    SpeckleAttributeSnapshot {
        byte_start: range.byte_start,
        byte_end: range.byte_end,
        arguments: attribute
            .arguments
            .into_iter()
            .map(|argument| match argument {
                SpeckleAttributeArgument::Identifier(id) => id,
            })
            .collect(),
    }
}

#[test]
fn test_speckle_attribute_bare() {
    insta::assert_yaml_snapshot!(snapshot_speckle_attribute("#[speckle]\nstruct Foo;"));
}

#[test]
fn test_speckle_attribute_positional_string() {
    insta::assert_yaml_snapshot!(snapshot_speckle_attribute(&format!(
        "#[speckle(\"{EXAMPLE_ID}\")]\nstruct Foo;"
    )));
}

#[test]
fn test_speckle_attribute_named_id() {
    insta::assert_yaml_snapshot!(snapshot_speckle_attribute(&format!(
        "#[speckle(id = \"{EXAMPLE_ID}\")]\nstruct Foo;"
    )));
}

#[test]
fn test_speckle_attribute_missing() {
    let item = parse_item("struct Foo;");
    assert!(matches!(
        item.speckle_attribute(),
        Err(SyntaxError::MissingSpeckleAttribute)
    ));
}

#[test]
fn test_speckle_attribute_rejects_unknown_named_argument() {
    let item = parse_item(&format!("#[speckle(uuid = \"{EXAMPLE_ID}\")]\nstruct Foo;"));
    assert!(matches!(
        item.speckle_attribute(),
        Err(SyntaxError::SpeckleAttribute(
            SpeckleAttributeError::DuplicateArgument
        ))
    ));
}
