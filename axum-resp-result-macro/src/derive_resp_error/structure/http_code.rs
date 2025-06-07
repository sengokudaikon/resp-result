use darling::FromMeta;
use heck::ToShoutySnakeCase;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, parse_str, Expr, Lit};

#[derive(Debug)]
pub enum HttpCode {
    Str(String, Span),
    Num(u16, Span),
}

impl FromMeta for HttpCode {
    fn from_value(lit: &Lit) -> darling::Result<Self> {
        let span = lit.span();
        match lit {
            Lit::Int(lit_int) => {
                let num = lit_int.base10_parse::<u16>()?;
                Ok(Self::Num(num, span))
            }
            Lit::Str(str) => {
                let str = str.value();
                Ok(Self::Str(str, span))
            }
            _ => Err(syn::Error::new(span, "Unexpected literal type").into()),
        }
    }
}

impl Parse for HttpCode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<Lit>()?;
        let span = lit.span();
        match lit {
            Lit::Int(lit_int) => {
                let num = lit_int.base10_parse::<u16>()?;
                Ok(Self::Num(num, span))
            }
            Lit::Str(str) => {
                let str = str.value();
                Ok(Self::Str(str, span))
            }
            _ => Err(syn::Error::new(span, "Unexpected literal type")),
        }
    }
}

impl TryInto<Expr> for HttpCode {
    type Error = syn::Error;

    fn try_into(self) -> Result<Expr, Self::Error> {
        match self {
            HttpCode::Str(str, span) => parse_str(&format!(
                "::axum_resp_result::StatusCode::{}",
                str.to_shouty_snake_case()
            ))
            .map_err(|err| {
                let mut er = syn::Error::new(span, "Parse StatusCode Error");
                er.combine(err);
                er
            }),
            HttpCode::Num(code, span) => {
                let _status =
                    http::StatusCode::from_u16(code).map_err(|err| syn::Error::new(span, err))?;
                parse2(quote!(::axum_resp_result::StatusCode::from_u16(#code).unwrap()))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use syn::Expr;

    #[test]
    fn test_parse_code() {
        const CODE: &str = r#"401"#;

        let expr: Expr = syn::parse_str::<super::HttpCode>(CODE)
            .unwrap()
            .try_into()
            .unwrap();

        println!("{expr:?}")
    }

    #[test]
    #[should_panic]
    fn test_parse_no_exist_code() {
        const CODE: &str = r#"9101"#;

        let expr: Expr = syn::parse_str::<super::HttpCode>(CODE)
            .unwrap()
            .try_into()
            .unwrap();

        println!("{expr:?}")
    }

    #[test]
    fn test_parse_name() {
        const CODE: &str = r#""NotFound""#;

        let expr: Expr = syn::parse_str::<super::HttpCode>(CODE)
            .unwrap()
            .try_into()
            .unwrap();

        println!("{expr:?}")
    }

    #[test]
    fn test_parse_no_exist_name() {
        const CODE: &str = r#""NotFoundABABAA""#;

        let expr: Expr = syn::parse_str::<super::HttpCode>(CODE)
            .unwrap()
            .try_into()
            .unwrap();

        println!("{expr:?}")
    }
}
