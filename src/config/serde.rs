use std::borrow::Cow;

use super::{
    status_signed::{InnerStatusSign, SignType},
    StatusSign,
};
use crate::owner_leak::OwnerLeaker;

static SIGNED_STATUS: StatusSign = StatusSign {
    field_name: Cow::Borrowed("is-ok"),
    ty: SignType::Bool,
};
#[cfg(feature = "extra-error")]
static EXTRA_ERR_MESSAGE: &str = "extra-error-message";
static ERROR_MESSAGE: &str = "error-message";
static BODY: &str = "body";

/// the config information of serialize
pub trait SerdeConfig {
    /// the name of body field, the field will be available when the
    /// [`RespResult`](crate::RespResult) is `Success(_)`
    ///
    /// ## Default
    /// the default field name is `body`
    fn body_name(&self) -> Cow<'static, str> { BODY.into() }
    /// the name of error_message field, this field will be available when
    /// [`RespResult`](crate::RespResult) is `Err(_)`
    ///
    /// ## Default
    /// the default field name is `error-message`
    fn err_msg_name(&self) -> Cow<'static, str> { ERROR_MESSAGE.into() }

    /// fixed field size,
    /// - if return `true`, will also serialize not available field with
    ///   `null`
    ///
    /// ## Default
    /// the default value is true
    fn fixed_field(&self) -> bool { true }

    /// sign the status of response
    /// - Some(_) **enable** this sign
    /// - None **disable** this sign
    ///
    /// ## Default
    /// default enable this sign using `bool`
    /// - true -> `Success`
    /// - false -> `Err`
    fn signed_status(&self) -> Option<StatusSign> {
        Some(SIGNED_STATUS.clone())
    }

    /// extra error message
    /// - Some(_) **enable** extra error message
    /// - None **disable** extra error message
    /// extra-error
    ///
    /// ## Default
    /// default enable with field name `extra-msg`
    #[cfg(feature = "extra-error")]
    fn extra_message(&self) -> Option<Cow<'static, str>> {
        Some(EXTRA_ERR_MESSAGE.into())
    }
}

pub(crate) struct InnerSerdeConfig {
    pub(crate) body_name: &'static str,
    pub(crate) err_msg_name: &'static str,
    pub(crate) full_field: bool,
    pub(crate) signed_status: Option<InnerStatusSign>,
    #[cfg(feature = "extra-error")]
    pub(crate) extra_code: Option<&'static str>,
    pub(crate) field_size: FieldSize,
}

impl InnerSerdeConfig {
    pub(crate) fn into_inner<C: SerdeConfig>(cfg: &C) -> Self {
        let mut s = Self {
            body_name: cfg.body_name().leak(),
            err_msg_name: cfg.err_msg_name().leak(),
            full_field: cfg.fixed_field(),
            signed_status: cfg.signed_status().map(Into::into),
            #[cfg(feature = "extra-error")]
            extra_code: cfg.extra_message().leak(),
            field_size: Default::default(),
        };

        let f = FieldSize::new(&s);
        s.field_size = f;

        s
    }

    pub(crate) fn get_field_size(&self) -> (usize, usize) {
        let FieldSize { ok_size, err_size } = self.field_size;
        (ok_size, err_size)
    }
}

#[derive(Debug, Default)]
pub(crate) struct FieldSize {
    ok_size: usize,
    err_size: usize,
}

impl FieldSize {
    pub(crate) fn new(cfg: &InnerSerdeConfig) -> Self {
        let (mut ok_size, mut err_size) = (1, 1);
        // 简易状态标记
        if cfg.signed_status.is_some() {
            ok_size += 1;
            err_size += 1;
        }
        // 额外的异常码
        #[cfg(feature = "extra-error")]
        if cfg.extra_code.is_some() {
            if cfg.full_field {
                ok_size += 1;
            }
            err_size += 1;
        }

        if cfg.full_field {
            ok_size += 1;
            err_size += 1;
        }
        Self { ok_size, err_size }
    }
}
