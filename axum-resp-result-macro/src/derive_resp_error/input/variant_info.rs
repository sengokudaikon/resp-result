use darling::FromVariant;

use crate::derive_resp_error::structure::http_code::HttpCode;

#[derive(Debug, FromVariant)]
#[darling(attributes(resp_result))]
pub struct VariantInfo {
    pub(crate) ident: syn::Ident,
    #[darling(rename = "err_msg")]
    pub(crate) resp_msg: Option<String>,
    #[darling(rename = "err_code")]
    pub(crate) http_code: Option<HttpCode>,
    #[cfg(feature = "extra-error")]
    #[darling(rename = "extra_msg")]
    pub(crate) extra_msg: Option<syn::Expr>,
}
