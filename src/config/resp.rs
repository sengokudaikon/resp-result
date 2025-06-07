#[allow(unused_imports)] use std::borrow::Cow;

#[allow(unused_imports)] use http::header::HeaderName;

#[allow(unused_imports)] use crate::owner_leak::OwnerLeaker;
/// the config of response
pub trait RespConfig {
    /// wether write the extra error message into header with the  provided
    /// name
    /// - `Some(_)` enable
    /// - `None` disable
    ///
    /// ## Default
    /// default is enable with name `extra-error`
    #[cfg(feature = "extra-error")]
    fn head_extra_code(&self) -> Option<Cow<'static, str>> {
        Some("extra-error".into())
    }
}

pub(crate) struct InnerRespConfig {
    #[cfg(feature = "extra-error")]
    pub(crate) extra_code: Option<HeaderName>,
}

impl InnerRespConfig {
    #[allow(unused_variables)]
    pub fn into_inner<C: RespConfig>(cfg: &C) -> Self {
        Self {
            #[cfg(feature = "extra-error")]
            extra_code: cfg
                .head_extra_code()
                .leak()
                .map(HeaderName::from_static),
        }
    }
}
