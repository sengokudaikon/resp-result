use std::borrow::Cow;

use serde::Serialize;

use crate::owner_leak::OwnerLeaker;

#[derive(Debug, Clone)]
/// the full info of status sign
pub struct StatusSign {
    /// the field name of the status sign
    pub(super) field_name: Cow<'static, str>,
    /// the type of the status sign
    pub(super) ty: SignType,
}

impl StatusSign {
    /// create a new [`StatusSign`] with provide `name` and `ty`
    pub fn new(name: impl Into<Cow<'static, str>>, ty: SignType) -> Self {
        Self {
            field_name: name.into(),
            ty,
        }
    }
}
#[derive(Debug, Clone)]
/// the type of the sign
pub enum SignType {
    /// using bool sign the resp result
    /// - `Success` => true
    /// - `Err` => false
    Bool,
    /// using bool sign the resp result, but revert it
    /// - `Success` => false
    /// - `Err` => true
    BoolRevert,
    /// using the provide number as the sign
    /// - `Success` => `on_ok`
    /// - `Err` => `on_fail`
    Number { on_ok: u8, on_fail: u8 },
    /// using the provide string as the sign
    /// - `Success` => `on_ok`
    /// - `Err` => `on_fail`
    Str {
        on_ok: Cow<'static, str>,
        on_fail: Cow<'static, str>,
    },
}

impl SignType {
    /// create a [`SignType`] using `Bool`
    pub const fn new_bool() -> Self { SignType::Bool }

    /// create a [`SignType`] using `BoolRevert`
    pub const fn new_bool_rev() -> Self { SignType::BoolRevert }

    /// create a [`SignType`] using `Number`
    pub const fn new_number(ok: u8, err: u8) -> Self {
        Self::Number {
            on_ok: ok,
            on_fail: err,
        }
    }

    /// create a [`SignType`] using `Str`
    pub fn new_str(
        ok: impl Into<Cow<'static, str>>, err: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::Str {
            on_ok: ok.into(),
            on_fail: err.into(),
        }
    }
}

pub(crate) struct InnerStatusSign {
    pub(crate) field: &'static str,
    pub(crate) ok: StatusEnum,
    pub(crate) err: StatusEnum,
}

impl From<StatusSign> for InnerStatusSign {
    fn from(StatusSign { field_name, ty }: StatusSign) -> Self {
        let (ok, err) = match ty {
            SignType::Bool => (StatusEnum::Bool, StatusEnum::BoolRev),
            SignType::BoolRevert => (StatusEnum::BoolRev, StatusEnum::Bool),
            SignType::Number { on_ok, on_fail } => {
                (StatusEnum::Number(on_ok), StatusEnum::Number(on_fail))
            }
            SignType::Str { on_ok, on_fail } => {
                (StatusEnum::Str(on_ok), StatusEnum::Str(on_fail))
            }
        };

        Self {
            field: field_name.leak(),
            ok,
            err,
        }
    }
}

pub(crate) enum StatusEnum {
    Bool,
    BoolRev,
    Number(u8),
    Str(Cow<'static, str>),
}

impl Serialize for StatusEnum {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            StatusEnum::Bool => serializer.serialize_bool(true),
            StatusEnum::BoolRev => serializer.serialize_bool(false),
            StatusEnum::Number(num) => serializer.serialize_u8(*num),
            StatusEnum::Str(s) => serializer.serialize_str(s),
        }
    }
}
