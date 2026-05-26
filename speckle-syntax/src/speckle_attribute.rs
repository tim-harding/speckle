use proc_macro2::Span;
use syn::{Attribute, LitStr, Meta, MetaList, spanned::Spanned};

#[derive(Debug)]
pub struct SpeckleAttribute {
    pub span: Span,
    pub arguments: Vec<SpeckleAttributeArgument>,
}

impl SpeckleAttribute {
    pub fn parse(attr: &Attribute) -> Result<Self, SpeckleAttributeError> {
        if !attr.path().is_ident("speckle") {
            return Err(SpeckleAttributeError::ExpectedSpeckleAttribute);
        }

        let arguments = match &attr.meta {
            Meta::Path(_) => Vec::new(),
            Meta::List(list) => parse_speckle_list(list)?,
            Meta::NameValue(_) => {
                return Err(SpeckleAttributeError::ExpectedSpeckleAttributeList);
            }
        };

        Ok(Self {
            span: attr.span(),
            arguments,
        })
    }

    pub fn is_bare(&self) -> bool {
        self.arguments.is_empty()
    }

    pub fn identifier(&self) -> Option<&str> {
        match self.arguments.as_slice() {
            [SpeckleAttributeArgument::Identifier(id)] => Some(id),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SpeckleAttributeArgument {
    Identifier(String),
}

fn parse_speckle_list(
    list: &MetaList,
) -> Result<Vec<SpeckleAttributeArgument>, SpeckleAttributeError> {
    if let Ok(lit) = list.parse_args::<LitStr>() {
        return Ok(vec![SpeckleAttributeArgument::Identifier(lit.value())]);
    }

    let mut arguments = Vec::new();
    list.parse_nested_meta(|meta| {
        if !meta.path.is_ident("identifier") {
            return Err(meta.error("expected `identifier`"));
        }
        let value = meta
            .value()?
            .parse::<LitStr>()
            .map_err(|err| meta.error(err))?;
        if !arguments.is_empty() {
            return Err(meta.error("duplicate argument"));
        }
        arguments.push(SpeckleAttributeArgument::Identifier(value.value()));
        Ok(())
    })
    .map_err(|_| SpeckleAttributeError::DuplicateArgument)?;

    Ok(arguments)
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SpeckleAttributeError {
    #[error("expected `speckle` attribute")]
    ExpectedSpeckleAttribute,
    #[error("expected `#[speckle]` or `#[speckle(...)]`")]
    ExpectedSpeckleAttributeList,
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
