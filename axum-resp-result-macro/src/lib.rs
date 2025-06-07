mod derive_resp_error;
mod proc_resp_result;
use proc_resp_result::Function;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

use crate::derive_resp_error::gen_resp_error_derive;

/// convert a return [Result] [`Handler`](axum::Handler) return [`RespResult`]
#[proc_macro_attribute]
pub fn resp_result(
    _: proc_macro::TokenStream, input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let func = syn::parse_macro_input!(input as Function);
    quote!(#func).into()
}

#[proc_macro_derive(RespError, attributes(resp_result))]
pub fn derive_resp_error(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if cfg!(feature = "extra-error") {
        return syn::Error::new(
            input.span(),
            "Derive macro `RespError` not support `extra-message` yet",
        )
        .into_compile_error()
        .into();
    }
    let token_stream = gen_resp_error_derive(&input)
        .unwrap_or_else(|err| err.into_compile_error());
    token_stream.into()
}
