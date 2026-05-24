use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait,
    ItemUnion,
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_macro_input,
};

#[allow(dead_code)]
enum Item {
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

#[proc_macro_attribute]
pub fn speckle(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    match item {
        Item::Static(_) => todo!(),
        Item::Const(_) => todo!(),
        Item::Struct(_) => todo!(),
        Item::Enum(_) => todo!(),
        Item::Union(_) => todo!(),
        Item::Fn(item) => {
            let ItemFn {
                attrs,
                vis,
                sig,
                block: _,
            } = item;
            TokenStream::from(quote! {
                #(#attrs)*
                #vis #sig {
                    todo!(concat!("speckle_impl::", module_path!(), ":", file!(), ":", line!()));
                }
            })
        }
        Item::Trait(_) => todo!(),
        Item::Impl(_) => todo!(),
        Item::Macro(_) => todo!(),
        Item::Mod(_) => todo!(),
    }
}
