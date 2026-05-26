use proc_macro2::Span;
use syn::{Attribute, Lit::Str, LitStr, Meta, MetaList, MetaNameValue, spanned::Spanned};

#[derive(Debug)]
pub struct SpeckleAttribute {
    pub span: Span,
    pub kind: SpeckleAttributeKind,
}

impl SpeckleAttribute {
    pub fn parse(meta: &Meta) -> Result<Self, SpeckleParseError> {
        let kind = SpeckleAttributeKind::parse(meta)?;
        Ok(SpeckleAttribute {
            span: meta.span(),
            kind,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpeckleAttributeKind {
    Unidentified,
    Identified(String),
}

impl SpeckleAttributeKind {
    pub fn parse(meta: &Meta) -> Result<Self, SpeckleParseError> {
        match meta {
            // bare `#[speckle]` attribute
            Meta::Path(_) => Ok(SpeckleAttributeKind::Unidentified),
            // `#[speckle("id-foo")]` attribute
            Meta::List(list) => {
                todo!()
            }
            // `#[speckle(id = "id-foo")]` attribute
            Meta::NameValue(MetaNameValue {
                path,
                value,
                eq_token,
                ..
            }) => Ok(SpeckleAttributeKind::Identified(todo!())),
        }
    }

    /// ```rust
    /// let identified   = SpeckleAttributeKind::Identified("id-foo");
    /// let unidentified = SpeckleAttributeKind::Unidentified;
    /// assert!(   identified.is_identified())
    /// assert!(!unidentified.is_identified())
    /// ```
    pub fn is_identified(&self) -> bool {
        matches!(self, SpeckleAttributeKind::Identified(_))
    }

    /// ```rust
    /// let identified   = SpeckleAttributeKind::Identified("id-foo");
    /// let unidentified = SpeckleAttributeKind::Unidentified;
    /// assert!(!unidentified.is_identified  ())
    /// assert!( unidentified.is_unidentified())
    /// ```
    pub fn is_unidentified(&self) -> bool {
        matches!(self, SpeckleAttributeKind::Unidentified)
    }

    pub fn identifier(&self) -> Option<&str> {
        match self {
            SpeckleAttributeKind::Identified(id) => Some(id),
            SpeckleAttributeKind::Unidentified => None,
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SpeckleParseError {
    #[error("expected `speckle` attribute")]
    ExpectedSpeckleAttribute,
    #[error("expected `#[speckle]` or `#[speckle(...)]`")]
    ExpectedSpeckleAttributeList,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SpeckleAttributeError {
    #[error(transparent)]
    Parse(#[from] SpeckleParseError),
    #[error("expected `identifier`")]
    ExpectedIdentifier,
    #[error("duplicate argument")]
    DuplicateArgument,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Item, SyntaxError};
    use indoc::{formatdoc, indoc};
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
        let range = attribute.span.byte_range();
        SpeckleAttributeSnapshot {
            byte_start: range.start,
            byte_end: range.end,
            arguments: todo!(),
        }
    }

    #[test]
    fn test_speckle_attribute_bare() {
        insta::assert_yaml_snapshot!(snapshot_speckle_attribute(indoc! {"
            #[speckle]
            struct Foo;
        "}));
    }

    #[test]
    fn test_speckle_attribute_positional_string() {
        insta::assert_yaml_snapshot!(snapshot_speckle_attribute(&formatdoc! {"
            #[speckle(\"{EXAMPLE_ID}\")]
            struct Foo;
        "}));
    }

    #[test]
    fn test_speckle_attribute_named_identifier() {
        insta::assert_yaml_snapshot!(snapshot_speckle_attribute(&formatdoc! {"
            #[speckle(identifier = \"{EXAMPLE_ID}\")]
            struct Foo;
        "}));
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
        let item = parse_item(&formatdoc! {"
            #[speckle(uuid = \"{EXAMPLE_ID}\")]
            struct Foo;
        "});
        assert!(matches!(
            item.speckle_attribute(),
            Err(SyntaxError::SpeckleAttribute(
                SpeckleAttributeError::DuplicateArgument
            ))
        ));
    }
}
