use proc_macro2::Span;
use syn::{
    Attribute, Expr, ExprLit, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemUnion, Lit, MetaNameValue,
    parse::{Parse, ParseStream, Result as ParseResult},
    spanned::Spanned,
};

#[cfg(test)]
mod item_docs_tests;

#[cfg(test)]
mod item_span_content_tests;

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
            syn::Item::Mod(item) => Item::Mod(item),
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
            Item::Macro(item) => item.mac.span(),
            Item::Mod(item) => item.mod_token.span(),
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

    pub fn speckle_attribute(&self) -> Result<&Attribute, SyntaxError> {
        self.attributes()
            .iter()
            .find(|attr| attr.path().is_ident("speckle"))
            .ok_or(SyntaxError::MissingSpeckleAttribute)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SyntaxError {
    #[error("Missing #[speckle] attribute")]
    MissingSpeckleAttribute,
}
