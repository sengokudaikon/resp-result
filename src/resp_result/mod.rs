#[cfg(feature = "trace")] use std::any::type_name;
use std::fmt::Debug;

use crate::resp_error::RespError;

pub mod serde;
pub mod to_response;
mod try_macro;
#[cfg(feature = "nightly_try_v2")] mod try_op;

pub use to_response::Nil;
#[cfg(feature = "trace")] use tracing::{event, Level};

/// resp result for more flexible control the response body
///
/// - [`Result`] will become `500` using as web framework response type when
///   `Err(_)`, the action usually not I expect
/// - using non-Result type as web framework response type cannot using `?`,
///   the code will fill with `if let` or `match`
///
/// that why I need a [`RespResult`] which can
/// - control respond code or other message when it become
///   [`RespResult::Err`], not always `500`
/// - impl the [`Try`](std::ops::Try) thus can using friendly `?` to simplify
///   code
///
/// > note: because the [`Try`](std::ops::Try) not stable yet, this crate need
/// > `Nightly` rust
pub enum RespResult<T, E> {
    /// the respond is success with response body `T`
    Success(T),
    /// the respond is failure with response error `E`
    Err(E),
}

impl<T: std::fmt::Debug, E: RespError> Debug for RespResult<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(arg0) => {
                f.debug_tuple("Success").field(arg0).finish()
            }
            Self::Err(arg0) => {
                f.debug_tuple("Err").field(&arg0.log_message()).finish()
            }
        }
    }
}

impl<T: std::fmt::Display, E: RespError> std::fmt::Display
    for RespResult<T, E>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespResult::Success(data) => write!(f, "RespResult Ok[{data}]"),
            RespResult::Err(err) => {
                write!(f, "RespResult Err[{}]", err.log_message())
            }
        }
    }
}

impl<T, E> RespResult<T, E> {
    #[inline]
    /// map currently `T` into `N`,
    ///
    /// this method is similar to the same name method of [`Result`]
    pub fn map<N, F>(self, f: F) -> RespResult<N, E>
    where
        F: FnOnce(T) -> N,
    {
        #[cfg(feature = "trace")]
        event!(
            Level::TRACE,
            event = "Mapping",
            from = type_name::<T>(),
            to = type_name::<N>()
        );
        match self {
            RespResult::Success(data) => RespResult::Success(f(data)),
            RespResult::Err(e) => RespResult::Err(e),
        }
    }

    #[inline]
    /// map currently `E` into `N`,
    ///
    /// this method is similar to the same name method of [`Result`]
    pub fn map_err<N, F>(self, f: F) -> RespResult<T, N>
    where
        F: FnOnce(E) -> N,
    {
        #[cfg(feature = "trace")]
        event!(
            Level::TRACE,
            event = "Mapping Error",
            from = type_name::<E>(),
            to = type_name::<N>()
        );
        match self {
            RespResult::Success(data) => RespResult::Success(data),
            RespResult::Err(e) => RespResult::Err(f(e)),
        }
    }

    #[inline]
    /// this method is similar to the same name method of [`Result`]
    pub fn and_then<N, F>(self, f: F) -> RespResult<N, E>
    where
        F: FnOnce(T) -> RespResult<N, E>,
    {
        match self {
            RespResult::Success(data) => f(data),
            RespResult::Err(e) => RespResult::Err(e),
        }
    }

    #[inline]
    /// this method is similar to the same name method of [`Result`]
    pub fn or_else<N, F>(self, f: F) -> RespResult<T, N>
    where
        F: FnOnce(E) -> RespResult<T, N>,
    {
        match self {
            RespResult::Success(data) => RespResult::Success(data),
            RespResult::Err(e) => f(e),
        }
    }
}

impl<T, E> From<Result<T, E>> for RespResult<T, E>
where
    E: RespError,
{
    #[inline]
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(data) => Self::ok(data),
            Err(err) => Self::err(err),
        }
    }
}

impl<T, E> RespResult<T, E> {
    #[inline]
    /// create an success [`RespResult`]
    pub fn ok(data: T) -> Self { Self::Success(data) }

    #[inline]
    /// create an error [`RespResult`]
    pub fn err(err: E) -> Self
    where
        E: RespError,
    {
        Self::Err(err)
    }
}
