#![cfg_attr(feature = "nightly_try_v2", feature(try_trait_v2))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../Readme.md")]

mod config;
mod convert;
mod expect_ext;
mod extra_flag;
mod owner_leak;
mod resp_body;
mod resp_error;
mod resp_result;

pub use self::resp_result::to_response::axum::axum_respond_part;
use once_cell::sync::OnceCell;

use config::InnerConfig;
pub use config::{ConfigTrait, DefaultConfig, RespConfig, SerdeConfig, SignType, StatusSign};
pub use convert::{
    from_request::{FromRequestFamily, MapReject, ToInner},
    resp_try, IntoRespResult, IntoRespResultWithErr,
};
pub use extra_flag::{
    flag_wrap::FlagWrap,
    flags::{ExtraFlag, ExtraFlags, HeaderType},
};
pub use resp_error::RespError;
pub use resp_result::{Nil, RespResult};

pub type FlagRespResult<T, E> = RespResult<FlagWrap<T>, E>;

static RESP_RESULT_CONFIG: OnceCell<InnerConfig> = OnceCell::new();

pub fn try_set_config<C: ConfigTrait>(cfg: &C) -> Result<(), SetRespResultConfigureError> {
    let inner = InnerConfig::from_cfg(cfg);

    #[cfg(feature = "trace")]
    tracing::event!(tracing::Level::DEBUG, set_config = "On Going");
    RESP_RESULT_CONFIG
        .set(inner)
        .map_err(|_| SetRespResultConfigureError)
}

/// set the [`RespResult`] config, will change the action on generate response body
///
/// ## Panic
///
/// the config can only been set once, multiple times set will cause panic
pub fn set_config<C: ConfigTrait>(cfg: &C) {
    match try_set_config(cfg) {
        Ok(_) => {
            #[cfg(feature = "trace")]
            tracing::event!(tracing::Level::INFO, set_config = "Ready");
        }
        Err(err) => {
            #[cfg(feature = "trace")]
            tracing::event!(tracing::Level::ERROR, set_config = "Error", error = %err);
            panic!("{}", err);
        }
    }
}

pub(crate) fn get_config() -> &'static InnerConfig {
    RESP_RESULT_CONFIG.get_or_init(|| {
        #[cfg(feature = "trace")]
        tracing::event!(
            tracing::Level::WARN,
            set_config = "None",
            action = "Using Default"
        );
        Default::default()
    })
}
#[derive(Debug, thiserror::Error)]
#[error("RespResult Configure has set")]
pub struct SetRespResultConfigureError;

pub use axum_resp_result_macro::resp_result as rresult;
pub use axum_resp_result_macro::resp_result;
pub use axum_resp_result_macro::RespError;
pub use convert::Fallible;
pub use http::StatusCode;
