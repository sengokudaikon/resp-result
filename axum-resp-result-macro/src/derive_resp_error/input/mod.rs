mod variant_info;

use crate::derive_resp_error::codegen::{RespErrorCodeGen, VariantCodeGen};
use crate::derive_resp_error::input::variant_info::VariantInfo;
use darling::util::Ignored;
use darling::{ast, FromDeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(resp_result), supports(enum_any))]
pub struct RespErrorDeriveInput {
    pub(crate) ident: syn::Ident,
    pub(crate) data: ast::Data<VariantInfo, Ignored>,
}

impl TryInto<RespErrorCodeGen> for RespErrorDeriveInput {
    type Error = syn::Error;

    fn try_into(self) -> Result<RespErrorCodeGen, Self::Error> {
        let variants = self
            .data
            .take_enum()
            .ok_or_else(|| syn::Error::new(self.ident.span(), "Only Support Enum"))?;
        let size = variants.capacity();
        let mut vars = Vec::with_capacity(size);

        for VariantInfo {
            ident,
            http_code,
            resp_msg,
        } in variants
        {
            let http_code = http_code.map(TryInto::try_into).transpose()?;
            vars.push(VariantCodeGen {
                ident,
                resp_msg,
                http_code,
            })
        }

        Ok(RespErrorCodeGen {
            ident: self.ident,
            variants: vars,
        })
    }
}
