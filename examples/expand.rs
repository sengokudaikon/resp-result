use axum_resp_result::RespError;
use error::PlainError;
fn main() {
    let _ret = test((1, 2), String::new());
    match _ret {
        RespResult::Success(_) => {}
        RespResult::Err(err) => {
            println!(
                "{:?},{:?}, {:?}",
                err.resp_message(),
                err.http_code(),
                err.log_message()
            )
        }
    }
}
mod error {
    use std::num::ParseIntError;

    use axum::extract::rejection::PathRejection;
    use axum_resp_result::RespError;
    #[derive(Debug, thiserror::Error, RespError)]
    pub(super) enum PlainError {
        #[error("Parse Request Path Error: {0}")]
        #[resp_result(err_msg = "Parse Request Path Error", err_code = 400)]
        Path(#[from] PathRejection),
        #[error("Parse Int Error: {0}")]
        #[resp_result(
            err_msg = "Invalid Input Integer",
            err_code = "Bad Request"
        )]
        ParseInt(#[from] ParseIntError),
    }
}
use axum_resp_result::{rresult, RespResult};
#[rresult]
fn test((a, b): (i32, i64), foo: String) -> Result<(), PlainError> {
    println!("{a},{b},{foo}");
    let a = foo.parse::<i32>()?;
    println!("{a:?}");
    Ok(())
}
