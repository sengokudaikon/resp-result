use axum::extract::{FromRequest, FromRequestParts, Request};

use super::{FromRequestFamily, MapReject, ToInner};
use crate::{Nil, RespError, RespResult};

impl<S, T, E> FromRequest<S> for MapReject<T, E>
where
    S: Sync + Send,
    E: Send + From<<T::Payload as FromRequest<S>>::Rejection> + RespError,
    T: FromRequestFamily<E>,
    T::Payload: FromRequest<S>,
{
    type Rejection = RespResult<Nil, E>;

    async fn from_request(
        req: Request, state: &S,
    ) -> Result<Self, Self::Rejection> {
        match <T::Payload as FromRequest<S>>::from_request(req, state).await {
            Ok(data) => Ok(Self(data.to_inner())),
            Err(err) => Err(RespResult::Err(E::from(err))),
        }
    }
}

impl<S, T, E> FromRequestParts<S> for MapReject<T, E>
where
    S: Sync + Send,
    E: Send
        + From<<T::Payload as FromRequestParts<S>>::Rejection>
        + RespError,
    T: FromRequestFamily<E>,
    T::Payload: FromRequestParts<S>,
{
    type Rejection = RespResult<Nil, E>;

    async fn from_request_parts(
        parts: &mut http::request::Parts, state: &S,
    ) -> Result<Self, Self::Rejection> {
        match <T::Payload as FromRequestParts<S>>::from_request_parts(
            parts, state,
        )
        .await
        {
            Ok(data) => Ok(Self(data.to_inner())),
            Err(err) => Err(RespResult::Err(E::from(err))),
        }
    }
}
mod from_request_families {
    use axum::extract::{Extension, Form, Json, Path, Query, State};

    use crate::convert::from_request::ToInner;

    impl<T> ToInner for Extension<T> {
        type Inner = T;

        fn to_inner(self) -> Self::Inner { self.0 }
    }

    impl<T> ToInner for Form<T> {
        type Inner = T;

        fn to_inner(self) -> Self::Inner { self.0 }
    }

    impl<T> ToInner for Json<T> {
        type Inner = T;

        fn to_inner(self) -> Self::Inner { self.0 }
    }

    impl<T> ToInner for Path<T> {
        type Inner = T;

        fn to_inner(self) -> Self::Inner { self.0 }
    }

    impl<T> ToInner for Query<T> {
        type Inner = T;

        fn to_inner(self) -> Self::Inner { self.0 }
    }

    impl<T> ToInner for State<T> {
        type Inner = T;

        fn to_inner(self) -> Self::Inner { self.0 }
    }
}
