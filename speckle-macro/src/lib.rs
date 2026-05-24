use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemConst, ItemEnum, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMod, ItemStatic,
    ItemStruct, ItemTrait, ItemUnion,
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
    ForeignMod(ItemForeignMod),
    Mod(ItemMod),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Fn) {
            Ok(Item::Fn(input.parse::<ItemFn>()?))
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
                    compile_error!(concat!("speckle_impl::", module_path!(), ":", file!(), ":", line!()));
                }
            })
        }
        Item::Trait(_) => todo!(),
        Item::Impl(_) => todo!(),
        Item::Macro(_) => todo!(),
        Item::ForeignMod(_) => todo!(),
        Item::Mod(_) => todo!(),
    }
}
