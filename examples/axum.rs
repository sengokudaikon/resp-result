use axum::{body::Body, routing::get, Router};
use config::AxumConfig;
use echo::echo_number;
use error::PlainError;
use http::Request;

use axum_resp_result::{set_config, RespResult};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer};
use tracing::{metadata::LevelFilter, Level};
use tracing_subscriber::{
    fmt::format, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use want_304::want_304;

use std::net::SocketAddr;

use crate::rtry_router::{parse_to_i32, parse_to_i64};

#[tokio::main]
async fn main() {
    let fmt = tracing_subscriber::fmt::layer()
        .event_format(format())
        .with_target(true)
        .with_ansi(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse("")
        .unwrap();

    tracing_subscriber::registry().with(filter).with(fmt).init();

    set_config(&AxumConfig);

    let addr = SocketAddr::try_from(([127, 0, 0, 1], 5000u16)).unwrap();

    let router = Router::new()
        .route("/echo/:num", get(echo_number))
        .route("/want_304", get(want_304))
        .nest(
            "/parse",
            Router::new()
                .route("/i32/:v", get(parse_to_i32))
                .route("/i64/:v/:v2", get(parse_to_i64)),
        )
        .route("/panic", get(|| async { panic!("Panic it") }))
        .fallback(fallback)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO)),
        );

    let listen = TcpListener::bind(&addr).await.expect("Start server Error");

    axum::serve(listen, router.into_make_service())
        .await
        .expect("Server Error");
}

mod error {
    use std::{borrow::Cow, num::ParseIntError};

    use axum::extract::rejection::PathRejection;
    use axum_resp_result::RespError;
    use http::StatusCode;

    pub(super) struct PlainError {
        pub(super) msg: String,
        pub(super) code: u32,
    }

    impl From<PathRejection> for PlainError {
        fn from(err: PathRejection) -> Self {
            Self::new(err.to_string(), 991)
        }
    }

    impl From<ParseIntError> for PlainError {
        fn from(value: ParseIntError) -> Self {
            Self::new(value.to_string(), 1002)
        }
    }

    impl PlainError {
        pub(super) fn new(msg: String, code: u32) -> Self {
            Self { msg, code }
        }
    }

    impl RespError for PlainError {
        fn log_message(&self) -> Cow<'_, str> {
            format!("Plain Error Happened: {}", self.msg).into()
        }

        fn http_code(&self) -> http::StatusCode {
            StatusCode::BAD_REQUEST
        }

        type ExtraMessage = u32;

        fn extra_message(&self) -> Self::ExtraMessage {
            self.code
        }

        fn resp_message_default() -> Option<Cow<'static, str>> {
            Some("Success".into())
        }

        fn extra_message_default() -> Option<Self::ExtraMessage> {
            Some(0)
        }
    }
}

type PlainRResult<T> = RespResult<T, PlainError>;

mod echo {
    use crate::error::PlainError;
    use axum::extract::Path;
    use axum_resp_result::rresult;
    use axum_resp_result::MapReject;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub(super) struct Input {
        num: i32,
    }
    #[rresult]
    pub(super) async fn echo_number(
        MapReject(Input { num }): MapReject<Path<Input>, PlainError>,
    ) -> Result<String, PlainError> {
        Ok(format!("echo number {num}"))
    }
}

mod want_304 {
    use axum::extract::Query;
    use axum_resp_result::{ExtraFlag, FlagWrap, RespResult};
    use http::{header::CONTENT_TYPE, StatusCode};
    use serde::Deserialize;

    use crate::PlainRResult;

    #[derive(Debug, Deserialize)]
    pub(super) struct Input {
        want: bool,
    }

    pub(super) async fn want_304(
        Query(Input { want }): Query<Input>,
    ) -> PlainRResult<FlagWrap<&'static str>> {
        if !want {
            RespResult::flag_ok("Not a 304", ())
        } else {
            RespResult::ok("304").with_flags(
                ExtraFlag::empty_body()
                    + ExtraFlag::status(StatusCode::NOT_MODIFIED)
                    + ExtraFlag::remove_header(CONTENT_TYPE),
            )
        }
    }
}

mod rtry_router {
    use axum::extract::Path;
    use axum_resp_result::{resp_try, rtry, RespResult};

    use crate::{error::PlainError, PlainRResult};

    pub(super) async fn parse_to_i32(Path(v): Path<String>) -> PlainRResult<i32> {
        let v = rtry! {v.parse::<i32>()};
        RespResult::Success(v)
    }
    pub(super) async fn parse_to_i64(
        Path((v, v2)): Path<(String, String)>,
    ) -> RespResult<(i64, i64), PlainError> {
        resp_try(async { Ok((v.parse()?, v2.parse()?)) }).await
    }
}

async fn fallback(req: Request<Body>) -> PlainRResult<()> {
    Err(PlainError::new(
        format!("Router not exist {}", req.uri()),
        1000,
    ))
    .into()
}

mod config {
    use std::borrow::Cow;

    use axum_resp_result::{ConfigTrait, RespConfig, SerdeConfig, SignType, StatusSign};

    pub(super) struct AxumConfig;

    impl SerdeConfig for AxumConfig {
        fn signed_status(&self) -> Option<StatusSign> {
            Some(StatusSign::new("status", SignType::new_str("ok", "fail")))
        }

        fn extra_message(&self) -> Option<Cow<'static, str>> {
            Some("reterror".into())
        }

        fn fixed_field(&self) -> bool {
            true
        }

        fn err_msg_name(&self) -> Cow<'static, str> {
            "message".into()
        }
    }

    impl RespConfig for AxumConfig {
        fn head_extra_code(&self) -> Option<Cow<'static, str>> {
            None
        }
    }

    impl ConfigTrait for AxumConfig {}
}
