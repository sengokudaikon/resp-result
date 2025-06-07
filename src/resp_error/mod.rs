use std::borrow::Cow;

/// the error when [`RespResult`](crate::RespResult) is `Err(_)`
pub trait RespError {
    /// message for logger
    fn log_message(&self) -> Cow<'_, str>;
    /// message for response
    ///
    /// ## Default
    /// the [`RespError::resp_message`] default is equal to
    /// [`RespError::log_message`]
    #[inline]
    fn resp_message(&self) -> Cow<'_, str> { self.log_message() }

    /// the http code of this error
    ///
    /// ## Default
    /// the default http code is `500 Internal Server Error`
    #[inline]
    fn http_code(&self) -> http::StatusCode {
        http::StatusCode::INTERNAL_SERVER_ERROR
    }

    #[cfg(feature = "extra-error")]
    /// the associate type of extra message
    type ExtraMessage: serde::Serialize + 'static + Sized + std::fmt::Display;
    #[cfg(feature = "extra-error")]
    /// get the extra message
    fn extra_message(&self) -> Self::ExtraMessage;

    /// when `fix-field = true` using this value serialize error message
    ///
    /// ## Default
    ///
    /// default is [`None`](Option::None), it will serialize to `null`
    #[inline]
    fn resp_message_default() -> Option<Cow<'static, str>> { None }

    /// when `fix-field = true` using this value serialize extra error message
    ///
    /// ## Default
    ///
    /// default is [`None`], it will be serialized to `null`
    #[cfg(feature = "extra-error")]
    #[inline]
    fn extra_message_default() -> Option<Self::ExtraMessage> { None }
}
