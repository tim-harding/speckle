use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemFn,
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_macro_input,
};

enum Item {
    Fn(ItemFn),
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
    }
}
