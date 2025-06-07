use crate::derive_resp_error::structure::http_code::HttpCode;
use darling::FromVariant;

#[derive(Debug, FromVariant)]
#[darling(attributes(resp_result))]
pub struct VariantInfo {
    pub(crate) ident: syn::Ident,
    #[darling(rename = "err_msg")]
    pub(crate) resp_msg: Option<String>,
    #[darling(rename = "err_code")]
    pub(crate) http_code: Option<HttpCode>,
}
