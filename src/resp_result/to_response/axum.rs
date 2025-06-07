use axum;
impl<T, E> axum::response::IntoResponse for crate::RespResult<T, E>
where
    T: crate::resp_body::RespBody,
    E: crate::RespError,
{
    #[inline]
    #[cfg_attr(
        feature = "trace",
        tracing::instrument(name = "axum-into-response", skip_all)
    )]
    fn into_response(self) -> axum::response::Response {
        use crate::expect_ext::ExpectExt;

        let respond = super::PrepareRespond::from_resp_result(&self);
        let mut builder =
            axum::response::Response::builder().status(respond.status);

        builder
            .headers_mut()
            .with_expect("RespResult 构造响应时发生异常")
            .extend(respond.headers);
        builder
            .body(axum::body::Body::from(respond.body))
            .with_expect("RespResult 构造响应时发生异常")
    }
}
pub mod axum_respond_part {
    use std::{convert::Infallible, future::Future};

    use axum::response::{IntoResponse, IntoResponseParts, ResponseParts};

    use crate::{resp_body, Nil, RespError, RespResult};

    pub mod prefab_part_handle {
        use crate::Nil;

        pub fn no_part<T>(data: T) -> (T, Nil) { (data, Nil) }

        pub fn no_resp<T>(data: T) -> ((), T) { ((), data) }
    }

    #[inline]
    pub async fn resp_result_with_respond_part<
        T,
        E,
        F,
        Fut,
        R,
        P,
        Resp,
        Part,
    >(
        handle: F, part_handle: P,
    ) -> RespResultExtraPart<Resp, E, Part>
    where
        // handles
        F: FnOnce() -> Fut,
        Fut: Future<Output = R>,
        R: Into<RespResult<T, E>>,
        // part into respond and respond part
        P: FnOnce(T) -> (Resp, Part),
        Resp: resp_body::RespBody,
        Part: IntoResponseParts,
        E: RespError,
    {
        let (resp_result, part) = match handle().await.into() {
            RespResult::Success(data) => {
                let (resp, part) = part_handle(data);
                (RespResult::Success(resp), Some(part))
            }
            RespResult::Err(err) => (RespResult::Err(err), None),
        };
        RespResultExtraPart {
            inner: resp_result,
            extra: part,
        }
    }

    impl IntoResponseParts for Nil {
        type Error = Infallible;

        #[inline]
        fn into_response_parts(
            self, res: ResponseParts,
        ) -> Result<ResponseParts, Self::Error> {
            Ok(res)
        }
    }
    #[derive(Debug)]
    pub struct RespResultExtraPart<T, E, Extra>
    where
        T: resp_body::RespBody,
        E: RespError,
        Extra: IntoResponseParts,
    {
        inner: RespResult<T, E>,
        extra: Option<Extra>,
    }

    impl<T, E, Extra> IntoResponse for RespResultExtraPart<T, E, Extra>
    where
        T: resp_body::RespBody,
        E: RespError,
        Extra: IntoResponseParts,
    {
        #[inline]
        fn into_response(self) -> axum::response::Response {
            (self.extra, self.inner).into_response()
        }
    }

    impl<T, E, Extra> RespResultExtraPart<T, E, Extra>
    where
        T: resp_body::RespBody,
        E: RespError,
        Extra: IntoResponseParts,
    {
        #[inline]
        pub fn map<R, F>(self, map: F) -> RespResultExtraPart<T, E, R>
        where
            F: FnOnce(Extra) -> R,
            R: IntoResponseParts,
        {
            RespResultExtraPart {
                inner: self.inner,
                extra: self.extra.map(map),
            }
        }

        #[inline]
        pub fn map_none<F>(self, map: F) -> Self
        where
            F: FnOnce() -> Extra,
        {
            Self {
                inner: self.inner,
                extra: self.extra.or_else(|| Some(map())),
            }
        }
    }

    #[cfg(test)]
    mod test {
        use serde::Serialize;

        use super::resp_result_with_respond_part;
        use crate::RespError;

        #[derive(Debug)]
        struct MockError;

        impl Serialize for MockError {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str("Mock Error")
            }
        }

        impl RespError for MockError {
            #[cfg(feature = "extra-error")]
            type ExtraMessage = String;

            fn log_message(&self) -> std::borrow::Cow<'_, str> {
                "Mock Error".into()
            }

            #[cfg(feature = "extra-error")]
            fn extra_message(&self) -> Self::ExtraMessage { String::new() }
        }

        #[tokio::test]
        async fn test_wrap() {
            let resp = resp_result_with_respond_part(
                || {
                    async {
                        Result::<_, MockError>::Ok((
                            12i32,
                            [("auth_type", "12345")],
                        ))
                    }
                },
                |(body, part)| (body, part),
            )
            .await;

            println!("resp : {:?}", resp);
        }
    }
}
