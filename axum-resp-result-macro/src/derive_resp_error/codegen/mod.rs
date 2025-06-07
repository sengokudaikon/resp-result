use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use syn::Expr;

pub struct RespErrorCodeGen {
    pub(crate) ident: syn::Ident,
    pub(crate) variants: Vec<VariantCodeGen>,
}

impl ToTokens for RespErrorCodeGen {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let RespErrorCodeGen { ident, variants } = self;
        let resp_msg_rows = variants
            .iter()
            .filter_map(
                |VariantCodeGen {
                     ident, resp_msg, ..
                 }| { Some((ident, resp_msg.as_deref()?)) },
            )
            .map(|(ident, resp_msg)| quote!(Self::#ident{..} => std::borrow::Cow::Borrowed(#resp_msg)));

        let http_code_rows = variants
            .iter()
            .filter_map(
                |VariantCodeGen {
                     ident, http_code, ..
                 }| { Some((ident, http_code.as_ref()?)) },
            )
            .map(|(ident, code)| quote!(Self::#ident{..} => #code));

        let token = quote! {
            impl ::axum_resp_result::RespError for #ident{
                fn log_message(&self) -> std::borrow::Cow<'_, str> {
                    self.to_string().into()
                }
                fn http_code(&self) -> ::axum_resp_result::StatusCode {
                    match self {
                        #(#http_code_rows,)*
                        _=> ::axum_resp_result::StatusCode::INTERNAL_SERVER_ERROR
                    }
                }
                fn resp_message(&self) -> std::borrow::Cow<'_, str> {
                    match self{
                        #(#resp_msg_rows,)*
                        _ => <Self as ::axum_resp_result::RespError>::log_message(self)
                    }
                }
            }
        };
        tokens.extend(token)
    }
}

pub struct VariantCodeGen {
    pub(crate) ident: syn::Ident,
    pub(crate) resp_msg: Option<String>,
    pub(crate) http_code: Option<Expr>,
}
