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
        if !meta.path.is_ident("id") {
            return Err(meta.error("expected `id`"));
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
    #[error("expected `id`")]
    ExpectedId,
    #[error("duplicate argument")]
    DuplicateArgument,
}
