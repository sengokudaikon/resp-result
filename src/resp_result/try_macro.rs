/// similar to the [try](core::r#try) macro, but for
/// [RespResult](crate::RespResult)
/// - if the $expr is [RespResult::Success](crate::RespResult::Success), will
///   make it be the value of this expr
/// - if the $expr is [RespResult::Error](crate::RespResult::Success), will
///   direct return with the error
#[macro_export]
macro_rules! rtry {
    { $exp:expr } => {
        match $crate::IntoRespResult::into_rresult($exp.map_err(Into::into)){
            $crate::RespResult::Success(value) => value,
            err @ $crate::RespResult::Err(_) =>{
                return err
            },
        }
    };
}
