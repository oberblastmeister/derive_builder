mod valid;
mod expand;
mod ast;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// The entry point for the derive macro
#[proc_macro_derive(Builder, attributes(builder, each))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
