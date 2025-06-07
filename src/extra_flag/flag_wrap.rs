use super::effect::{BodyEffect, Effects};
use crate::{
    resp_body::{LoadSerde, RespBody},
    ExtraFlags, RespResult,
};

/// an wrap for adding extra flags.
/// the [`FlagWrap`] if and only if using like following
/// ```rust ignore
/// RespResult<FlagWrap<T>, E>
/// ```
pub struct FlagWrap<T> {
    inner: T,
    flags: ExtraFlags,
}

impl<T> FlagWrap<T> {
    /// crate a new [`FlagWrap`] with `data` and `flags`
    #[inline]
    pub fn new(data: T, flags: impl Into<ExtraFlags>) -> Self {
        Self {
            inner: data,
            flags: flags.into(),
        }
    }
}

impl<T, E> RespResult<T, E> {
    #[inline]
    /// create a [`RespResult::Success`] with flags
    pub fn flag_ok(
        data: T, flags: impl Into<ExtraFlags>,
    ) -> RespResult<FlagWrap<T>, E> {
        RespResult::ok(FlagWrap::new(data, flags))
    }

    #[inline]
    /// covert a [`RespResult::<T, E>`] into [`RespResult<FlagWrap<T>, E>`]
    /// with provide flags
    pub fn with_flags(
        self, flags: impl Into<ExtraFlags>,
    ) -> RespResult<FlagWrap<T>, E> {
        match self {
            RespResult::Success(data) => {
                RespResult::Success(FlagWrap::new(data, flags))
            }
            RespResult::Err(err) => RespResult::Err(err),
        }
    }
}

impl<T, E> From<RespResult<T, E>> for RespResult<FlagWrap<T>, E> {
    fn from(inner: RespResult<T, E>) -> Self { inner.with_flags(()) }
}

impl<T: LoadSerde> LoadSerde for FlagWrap<T> {
    type SerdeData = T::SerdeData;

    fn load_serde(&self) -> &Self::SerdeData { self.inner.load_serde() }
}

impl<T> Effects for FlagWrap<T> {
    #[inline]
    fn body_effect(&self, body: &mut Vec<u8>) -> BodyEffect {
        self.flags.body_effect(body)
    }

    #[inline]
    fn status_effect(&self) -> Option<http::StatusCode> {
        self.flags.status_effect()
    }

    #[inline]
    fn headers_effect(&self, map: &mut http::HeaderMap) {
        self.flags.headers_effect(map)
    }
}

impl<T: LoadSerde> RespBody for FlagWrap<T> {}

#[cfg(test)]
mod test {
    use http::StatusCode;

    use crate::{
        resp_result::serde::SerializeWrap, ExtraFlag, RespError, RespResult,
    };

    struct MockErr;

    impl RespError for MockErr {
        #[cfg(feature = "extra-error")]
        type ExtraMessage = String;

        fn log_message(&self) -> std::borrow::Cow<'_, str> {
            "Mock Error".into()
        }

        #[cfg(feature = "extra-error")]
        fn extra_message(&self) -> Self::ExtraMessage { "Mock".into() }
    }

    #[test]
    fn test_serde() {
        let a = RespResult::<_, MockErr>::Success(12i32).with_flags(
            ExtraFlag::EmptyBody
                + ExtraFlag::status(StatusCode::NOT_MODIFIED)
                + ExtraFlag::insert_header(http::header::ETAG, "1234567890")
                + ExtraFlag::insert_header(
                    http::header::CONTENT_TYPE,
                    mime::TEXT_PLAIN_UTF_8.as_ref(),
                ),
        );

        let s = serde_json::to_string_pretty(&SerializeWrap(&a)).unwrap();

        println!("{s}")
    }
}
