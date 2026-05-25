use proc_macro2::Span;
use syn::{
    Attribute, Expr, ExprLit, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemUnion, Lit, LitStr, Meta, MetaList, MetaNameValue,
    parse::{Parse, ParseStream, Result as ParseResult},
    spanned::Spanned,
};

#[cfg(test)]
mod item_docs_tests;

#[cfg(test)]
mod item_span_content_tests;

#[cfg(test)]
mod item_speckle_attribute_tests;

pub struct SourceRange {
    pub file: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

impl From<Span> for SourceRange {
    fn from(span: Span) -> Self {
        let bytes = span.byte_range();
        Self {
            file: span.file(),
            byte_start: bytes.start,
            byte_end: bytes.end,
        }
    }
}

pub enum Item {
    Static(ItemStatic),
    Const(ItemConst),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Union(ItemUnion),
    Fn(ItemFn),
    Trait(ItemTrait),
    Impl(ItemImpl),
    Macro(ItemMacro),
    Mod(ItemMod),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        input.parse::<syn::Item>()?.try_into()
    }
}

impl TryFrom<syn::Item> for Item {
    type Error = syn::Error;

    fn try_from(item: syn::Item) -> Result<Self, Self::Error> {
        Ok(match item {
            syn::Item::Static(item) => Item::Static(item),
            syn::Item::Const(item) => Item::Const(item),
            syn::Item::Struct(item) => Item::Struct(item),
            syn::Item::Enum(item) => Item::Enum(item),
            syn::Item::Union(item) => Item::Union(item),
            syn::Item::Fn(item) => Item::Fn(item),
            syn::Item::Trait(item) => Item::Trait(item),
            syn::Item::Impl(item) => Item::Impl(item),
            syn::Item::Macro(item) => Item::Macro(item),
            syn::Item::Mod(item) => {
                if item.content.is_none() {
                    return Err(syn::Error::new(
                        item.span(),
                        "file modules (`mod name;`) are not supported; use an inline module (`mod name { ... }`)",
                    ));
                }
                Item::Mod(item)
            }
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "expected one of: `static`, `const`, `struct`, `enum`, `union`, `fn`, `trait`, `impl`, `macro`, `mod`",
                ));
            }
        })
    }
}

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

    fn attributes(&self) -> &[Attribute] {
        match self {
            Item::Static(item) => item.attrs.as_slice(),
            Item::Const(item) => item.attrs.as_slice(),
            Item::Struct(item) => item.attrs.as_slice(),
            Item::Enum(item) => item.attrs.as_slice(),
            Item::Union(item) => item.attrs.as_slice(),
            Item::Fn(item) => item.attrs.as_slice(),
            Item::Trait(item) => item.attrs.as_slice(),
            Item::Impl(item) => item.attrs.as_slice(),
            Item::Macro(item) => item.attrs.as_slice(),
            Item::Mod(item) => item.attrs.as_slice(),
        }
    }

    pub fn speckle_attribute(&self) -> Result<SpeckleAttribute, SyntaxError> {
        let attr = self
            .attributes()
            .iter()
            .find(|attr| attr.path().is_ident("speckle"))
            .ok_or(SyntaxError::MissingSpeckleAttribute)?;
        SpeckleAttribute::parse(attr)
    }
}

#[derive(Debug)]
pub struct SpeckleAttribute {
    pub span: Span,
    pub arguments: Vec<SpeckleAttributeArgument>,
}

impl SpeckleAttribute {
    pub fn parse(attr: &Attribute) -> Result<Self, SyntaxError> {
        if !attr.path().is_ident("speckle") {
            return Err(SyntaxError::InvalidSpeckleAttribute(
                "expected `speckle` attribute".into(),
            ));
        }

        let arguments = match &attr.meta {
            Meta::Path(_) => Vec::new(),
            Meta::List(list) => parse_speckle_list(list)?,
            Meta::NameValue(_) => {
                return Err(SyntaxError::InvalidSpeckleAttribute(
                    "expected `#[speckle]` or `#[speckle(...)]`".into(),
                ));
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

fn parse_speckle_list(list: &MetaList) -> Result<Vec<SpeckleAttributeArgument>, SyntaxError> {
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
    .map_err(|err| SyntaxError::InvalidSpeckleAttribute(err.to_string()))?;

    Ok(arguments)
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SyntaxError {
    #[error("Missing #[speckle] attribute")]
    MissingSpeckleAttribute,
    #[error("Invalid #[speckle] attribute: {0}")]
    InvalidSpeckleAttribute(String),
}
