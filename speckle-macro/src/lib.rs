use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn speckle(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block: _,
    } = parse_macro_input!(input as ItemFn);

    TokenStream::from(quote! {
        #(#attrs)*
        #vis #sig {
            todo!()
        }
    })
}
