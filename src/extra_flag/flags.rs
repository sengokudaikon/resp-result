use std::{
    fmt::Debug,
    ops::{Add, AddAssign},
};

use http::{header::HeaderName, HeaderValue, StatusCode};

use crate::expect_ext::ExpectExt;

#[derive(Debug, Hash, PartialEq, Eq)]
/// the basic flag that can be using
pub enum ExtraFlag {
    /// set the respond body to empty
    EmptyBody,
    /// overwrite the default status code
    SetStatus(StatusCode),
    /// set the header value
    SetHeader(HeaderName, HeaderValue, HeaderType),
    /// remove a header
    RemoveHeader(HeaderName),
}
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
/// the action of set header
pub enum HeaderType {
    /// insert new header, if the header exist will push into the
    /// [`HeaderValue`] list
    Insert,
    /// append new header, if the header exist will overwrite old
    /// [`HeaderValue`]
    Append,
}

impl ExtraFlag {
    /// create [`ExtraFlag::EmptyBody`] flag
    #[inline]
    pub const fn empty_body() -> Self { Self::EmptyBody }

    #[inline]
    /// create [`ExtraFlag::SetStatus`] flag
    pub const fn status(status: StatusCode) -> Self {
        Self::SetStatus(status)
    }

    #[inline]
    /// create [`ExtraFlag::SetHeader`] flag with type [`HeaderType::Append`]
    pub fn append_header<K, V>(key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Debug,
        V: TryInto<HeaderValue>,
        V::Error: Debug,
    {
        Self::SetHeader(
            key.try_into().with_expect("Bad Header Name"),
            value.try_into().with_expect("Bad Header Value"),
            HeaderType::Append,
        )
    }

    #[inline]
    /// create [`ExtraFlag::SetHeader`] flag with type [`HeaderType::Append`]
    pub fn insert_header<K, V>(key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Debug,
        V: TryInto<HeaderValue>,
        V::Error: Debug,
    {
        Self::SetHeader(
            key.try_into().with_expect("Bad Header Name"),
            value.try_into().with_expect("Bad Header Value"),
            HeaderType::Insert,
        )
    }

    #[inline]
    /// create [`ExtraFlag::RemoveHeader`] flag
    pub fn remove_header<K>(key: K) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Debug,
    {
        Self::RemoveHeader(key.try_into().with_expect("Bad Header Name"))
    }
}

/// a set of extra flags
/// can using `+` or `+=` combine multiple [`ExtraFlag`]
///
/// # Example
///
/// ```rust
///
/// use http::StatusCode;
/// use axum_resp_result::{ExtraFlag, ExtraFlags};
///
/// let mut flags: ExtraFlags = ExtraFlag::empty_body() + ExtraFlag::status(StatusCode::BAD_REQUEST);
///
/// flags += ExtraFlag::append_header("bar","foo");
/// ```
pub struct ExtraFlags {
    pub(crate) flags: Vec<ExtraFlag>,
}

impl From<()> for ExtraFlags {
    fn from(_: ()) -> Self { ExtraFlags { flags: vec![] } }
}

impl From<ExtraFlag> for ExtraFlags {
    fn from(flag: ExtraFlag) -> Self {
        Self {
            flags: Vec::from_iter([flag]),
        }
    }
}

impl Add for ExtraFlag {
    type Output = ExtraFlags;

    fn add(self, rhs: Self) -> Self::Output {
        ExtraFlags {
            flags: Vec::from([self, rhs]),
        }
    }
}

impl Add<ExtraFlags> for ExtraFlag {
    type Output = ExtraFlags;

    fn add(self, mut rhs: ExtraFlags) -> Self::Output {
        rhs.flags.push(self);
        rhs
    }
}

impl Add<ExtraFlag> for ExtraFlags {
    type Output = ExtraFlags;

    fn add(mut self, rhs: ExtraFlag) -> Self::Output {
        self.flags.push(rhs);
        self
    }
}

impl AddAssign for ExtraFlags {
    fn add_assign(&mut self, rhs: Self) { self.flags.extend(rhs.flags); }
}

impl AddAssign<ExtraFlag> for ExtraFlags {
    fn add_assign(&mut self, rhs: ExtraFlag) { self.flags.push(rhs) }
}
