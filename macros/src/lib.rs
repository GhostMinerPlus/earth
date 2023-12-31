use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AsConfig)]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote::quote! {
        impl earth::AsConfig for #ident {}
    };
    output.into()
}
