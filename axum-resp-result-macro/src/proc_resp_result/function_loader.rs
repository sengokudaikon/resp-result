use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, spanned::Spanned, Attribute, Block, ItemFn, Pat, PatIdent, ReturnType, Signature,
    Visibility,
};

pub struct Function {
    attrs: Vec<Attribute>,
    vis: Visibility,
    inner_sig: Signature,
    outer_sig: Signature,
    block: Box<Block>,
    args: Vec<syn::Ident>,
    inner_ident: syn::Ident,
    is_async: bool,
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Function {
            attrs,
            vis,
            inner_sig,
            block,
            outer_sig,
            args,
            inner_ident,
            is_async,
        } = self;

        let ay = if *is_async {
            Some(quote!(.await))
        } else {
            None
        };
        let inner = quote!(#inner_sig #block);
        let outer = quote! {
            #(#attrs)*
            #vis #outer_sig
            {
                #inner

                let __tmp = #inner_ident(#(#args),*)#ay;
                let __tmp = ::axum_resp_result::Fallible::to_result(__tmp);
                ::axum_resp_result::IntoRespResult::into_rresult(__tmp)
            }
        };

        tokens.extend(outer)
    }
}

impl Parse for Function {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ItemFn {
            attrs,
            vis,
            sig,
            block,
        } = input.parse::<ItemFn>()?;

        let mut inner_sig = sig;
        if inner_sig.receiver().is_some() {
            return Err(syn::Error::new(
                inner_sig.receiver().span(),
                "macro `resp-result` not support for method",
            ));
        }

        let mut outer_sig = inner_sig.clone();

        // set return type
        let ret = outer_sig.output;
        let ret_type = match ret {
            syn::ReturnType::Default => quote!(()),
            syn::ReturnType::Type(_, ty) => quote!(#ty),
        };

        let new_ret = quote! {
            ::axum_resp_result::RespResult<
                <#ret_type as ::axum_resp_result::Fallible>::Success,
                <#ret_type as ::axum_resp_result::Fallible>::Failure,
            >
        };

        let new_ret = syn::parse::<ReturnType>(quote!(-> #new_ret).into())?;
        let is_async = inner_sig.asyncness.is_some();
        outer_sig.output = new_ret;

        let mut args = Vec::new();
        // set arg lists
        for (idx, ty) in outer_sig
            .inputs
            .iter_mut()
            .map(|v| match v {
                syn::FnArg::Typed(ty) => ty,
                syn::FnArg::Receiver(_) => unreachable!(),
            })
            .enumerate()
        {
            let ident = format_ident!("arg_{idx}");
            args.push(ident.clone());
            let pat = Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident,
                subpat: None,
            });

            ty.pat = Box::new(pat)
        }
        // set inner ident
        let inner_ident = format_ident!("__inner_func");
        inner_sig.ident = inner_ident.clone();
        Ok(Self {
            attrs,
            vis,
            inner_sig,
            block,
            outer_sig,
            args,
            inner_ident,
            is_async,
        })
    }
}
