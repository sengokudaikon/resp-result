mod variant_info;

use darling::{ast, util::Ignored, FromDeriveInput};

use crate::derive_resp_error::{
    codegen::{RespErrorCodeGen, VariantCodeGen},
    input::variant_info::VariantInfo,
};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(resp_result), supports(enum_any))]
pub struct RespErrorDeriveInput {
    pub(crate) ident: syn::Ident,
    pub(crate) data: ast::Data<VariantInfo, Ignored>,
    #[cfg(feature = "extra-error")]
    pub(crate) extra_message_type: Option<syn::Type>,
}

impl TryInto<RespErrorCodeGen> for RespErrorDeriveInput {
    type Error = syn::Error;

    fn try_into(self) -> Result<RespErrorCodeGen, Self::Error> {
        let variants = self.data.take_enum().ok_or_else(|| {
            syn::Error::new(self.ident.span(), "Only Support Enum")
        })?;
        let size = variants.capacity();
        let mut vars = Vec::with_capacity(size);

        for VariantInfo {
            ident,
            http_code,
            resp_msg,
            #[cfg(feature = "extra-error")]
            extra_msg,
        } in variants
        {
            let http_code = http_code.map(TryInto::try_into).transpose()?;
            vars.push(VariantCodeGen {
                ident,
                resp_msg,
                http_code,
                #[cfg(feature = "extra-error")]
                extra_msg,
            })
        }

        Ok(RespErrorCodeGen {
            ident: self.ident,
            variants: vars,
            #[cfg(feature = "extra-error")]
            extra_message_type: self.extra_message_type,
        })
    }
}
