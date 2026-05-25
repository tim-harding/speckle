use proc_macro2::Span;
use syn::{
    ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait,
    ItemUnion,
    parse::{Parse, ParseStream, Result as ParseResult},
    spanned::Spanned,
};

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
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Static) {
            Ok(Item::Static(input.parse::<ItemStatic>()?))
        } else if lookahead.peek(syn::token::Const) {
            Ok(Item::Const(input.parse::<ItemConst>()?))
        } else if lookahead.peek(syn::token::Struct) {
            Ok(Item::Struct(input.parse::<ItemStruct>()?))
        } else if lookahead.peek(syn::token::Enum) {
            Ok(Item::Enum(input.parse::<ItemEnum>()?))
        } else if lookahead.peek(syn::token::Union) {
            Ok(Item::Union(input.parse::<ItemUnion>()?))
        } else if lookahead.peek(syn::token::Fn) {
            Ok(Item::Fn(input.parse::<ItemFn>()?))
        } else if lookahead.peek(syn::token::Trait) {
            Ok(Item::Trait(input.parse::<ItemTrait>()?))
        } else if lookahead.peek(syn::token::Impl) {
            Ok(Item::Impl(input.parse::<ItemImpl>()?))
        } else if lookahead.peek(syn::token::Macro) {
            Ok(Item::Macro(input.parse::<ItemMacro>()?))
        } else if lookahead.peek(syn::token::Mod) {
            Ok(Item::Mod(input.parse::<ItemMod>()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Item {
    pub fn span_full(&self) -> Span {
        match self {
            Item::Static(item) => item.span(),
            Item::Const(item) => item.span(),
            Item::Struct(item) => item.span(),
            Item::Enum(item) => item.span(),
            Item::Union(item) => item.span(),
            Item::Fn(item) => item.span(),
            Item::Trait(item) => item.span(),
            Item::Impl(item) => item.span(),
            Item::Macro(item) => item.span(),
            Item::Mod(item) => item.span(),
        }
    }

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
}
