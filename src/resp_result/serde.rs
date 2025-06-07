use serde::{ser::SerializeStruct, Serialize, Serializer};
#[cfg(feature = "trace")]
use {
    std::any::type_name,
    tracing::{event, Level},
};

use super::RespResult;
use crate::{get_config, resp_body::RespBody, resp_error::RespError};

pub trait RespSerialize {
    fn resp_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

pub struct SerializeWrap<'s, S>(pub(crate) &'s S);

impl<'s, Rs: RespSerialize> Serialize for SerializeWrap<'s, Rs> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.resp_serialize(serializer)
    }
}

impl<T, E> RespSerialize for RespResult<T, E>
where
    T: RespBody,
    E: RespError,
{
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    fn resp_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let cfg = &get_config().serde;
        let (ok_size, err_size) = cfg.get_field_size();

        #[cfg(feature = "trace")]
        event!(
            Level::TRACE,
            serialize_field.Ok = ok_size,
            serialize_field.Err = err_size
        );

        let resp = match self {
            RespResult::Success(data) => {
                #[cfg(feature = "trace")]
                event!(
                    Level::DEBUG,
                    entry = "Success",
                    "data.type" = type_name::<T>(),
                    "data.payload.type" = type_name::<
                        <T as crate::resp_body::LoadSerde>::SerdeData,
                    >()
                );

                let mut body =
                    serializer.serialize_struct("RespResult", ok_size)?;
                if let Some(ref signed_status) = cfg.signed_status {
                    body.serialize_field(
                        signed_status.field,
                        &signed_status.ok,
                    )?;
                }
                if cfg.full_field {
                    #[cfg(feature = "extra-error")]
                    if let Some(ecl) = cfg.extra_code {
                        body.serialize_field(
                            ecl,
                            &E::extra_message_default(),
                        )?;
                    }
                    body.serialize_field(
                        cfg.err_msg_name,
                        &E::resp_message_default(),
                    )?;
                }

                body.serialize_field(cfg.body_name, data.load_serde())?;

                body.end()?
            }
            RespResult::Err(err) => {
                #[cfg(feature = "trace")]
                event!(
                    Level::DEBUG,
                    entry = "Error",
                    "error.type" = type_name::<E>(),
                    error = %err.log_message()
                );
                let mut body =
                    serializer.serialize_struct("RespResult", err_size)?;

                if let Some(ref status_sign) = cfg.signed_status {
                    body.serialize_field(
                        status_sign.field,
                        &status_sign.err,
                    )?;
                }
                #[cfg(feature = "extra-error")]
                if let Some(ecl) = cfg.extra_code {
                    body.serialize_field(ecl, &err.extra_message())?;
                }
                body.serialize_field(cfg.err_msg_name, &err.resp_message())?;

                if cfg.full_field {
                    body.serialize_field(cfg.body_name, &())?;
                }
                body.end()?
            }
        };
        Ok(resp)
    }
}
