pub mod from_request;
use std::{convert::Infallible, future::Future};

use crate::{RespError, RespResult};

/// convert into [`RespResult`](crate::RespResult)
pub trait IntoRespResult<T, E: RespError> {
    fn into_rresult(self) -> RespResult<T, E>;
}

/// convert into [`RespResult`](crate::RespResult) with provide error
pub trait IntoRespResultWithErr<T, E: RespError> {
    fn into_with_err<Et: Into<E>>(self, err: Et) -> RespResult<T, E>;
}

impl<E, T> IntoRespResult<T, E> for Result<T, E>
where
    E: RespError,
{
    #[inline]
    fn into_rresult(self) -> RespResult<T, E> { RespResult::from(self) }
}

impl<E, T> IntoRespResult<T, E> for RespResult<T, E>
where
    E: RespError,
{
    #[inline]
    fn into_rresult(self) -> RespResult<T, E> { self }
}

impl<T, E> IntoRespResultWithErr<T, E> for Option<T>
where
    E: RespError,
{
    #[inline]
    fn into_with_err<Et: Into<E>>(self, err: Et) -> RespResult<T, E> {
        self.ok_or(err).map_err(Into::into).into_rresult()
    }
}

pub trait Fallible {
    type Success;
    type Failure;

    fn to_result(self) -> Result<Self::Success, Self::Failure>;
}

impl<T, R> Fallible for Result<T, R> {
    type Failure = R;
    type Success = T;

    fn to_result(self) -> Result<Self::Success, Self::Failure> { self }
}

impl<T, R> Fallible for RespResult<T, R> {
    type Failure = R;
    type Success = T;

    fn to_result(self) -> Result<<Self as Fallible>::Success, Self::Failure> {
        match self {
            RespResult::Success(suc) => Ok(suc),
            RespResult::Err(err) => Err(err),
        }
    }
}

impl Fallible for () {
    type Failure = Infallible;
    type Success = ();

    fn to_result(self) -> Result<Self::Success, Self::Failure> { Ok(self) }
}

#[inline]
/// receive a [Future](core::future::Future) applying it immediately, then
/// convent the result into [RespResult](crate::RespResult)
pub async fn resp_try<Fut, T, E>(future: Fut) -> RespResult<T, E>
where
    Fut: Future,
    Fut::Output: IntoRespResult<T, E>,
    E: RespError,
{
    future.await.into_rresult()
}
