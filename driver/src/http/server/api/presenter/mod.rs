use crate::http::server::response::{constants, response_with_code, WithSetCookie};

use axum::http::StatusCode;
use axum::response::sse::Event;
use axum::response::{IntoResponse, Response, Sse};
use axum::Json;
use cookie::{Cookie, CookieBuilder, SameSite};
use futures::StreamExt;

use ::openapi::models::{Account, ErrorMessage, StatusOk, StatusResponse};
use application::usecase::account::GetAccountOutput;
use application::usecase::auth::{
    ForgetPasswordOutput, SignInOutput, SignOutOutput, SignUpFinishOutput,
};
use application::usecase::channel::SubscribeOutput;
use application::usecase::status::StatusOutput;
use kernel::error::Codes;
use kernel::Result;
use log;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::convert::Infallible;
use std::time::Duration;

pub mod openapi;

pub(crate) fn present_status_output(data: Result<StatusOutput>) -> Result<Response, ()> {
    match data {
        Ok(s) => Ok(ok_response_with_message(StatusResponse::new(
            status_reason(StatusCode::OK),
            s.version.unwrap_or_else(|| "-".to_string()),
            s.build_timestamp,
        ))),
        Err(e) => Ok(convert_server_error(e)),
    }
}

pub(crate) fn present_get_account_output(data: Result<GetAccountOutput>) -> Result<Response, ()> {
    match data {
        Ok(output) => {
            let Some(account) = output.account else  {
                return Ok(not_found(Codes::CommonNotFound, Some("account not found".to_string())));
            };
            Ok(ok_response_with_message(Account::new(
                account.id,
                account.name,
                account.display_name,
            )))
        }
        Err(e) => Ok(convert_server_error(e)),
    }
}

pub(crate) fn present_signin_output(data: Result<SignInOutput>) -> Result<Response, ()> {
    match data {
        Ok(output) => {
            let SignInOutput {
                session_id,
                remember_me,
            } = output;
            let max_age = if remember_me {
                Some(time::Duration::seconds(86400 * 10))
            } else {
                None
            };
            Ok(status_ok_response().with_cookie(set_session_cookie(session_id, max_age)))
        }
        Err(e) => Ok(convert_server_error(e)),
    }
}

pub(crate) fn present_signup_finish_output(
    data: Result<SignUpFinishOutput>,
) -> Result<Response, ()> {
    match data {
        Ok(output) => {
            let SignUpFinishOutput { session_id } = output;
            Ok(status_ok_response().with_cookie(set_session_cookie(session_id, None)))
        }
        Err(e) => Ok(convert_server_error(e)),
    }
}
pub(crate) fn present_signout_output(data: Result<SignOutOutput>) -> Result<Response, ()> {
    match data {
        Ok(_) => Ok(status_ok_response().with_cookie(delete_session_cookie())),
        Err(e) => Ok(convert_server_error(e)),
    }
}
pub(crate) fn present_forget_password_output(
    data: Result<ForgetPasswordOutput>,
) -> Result<Response, ()> {
    match data {
        Ok(_) => Ok(status_ok_response().with_cookie(delete_session_cookie())),
        Err(e) => Ok(convert_server_error(e)),
    }
}
pub(crate) fn present_subscribe_output(data: Result<SubscribeOutput>) -> Result<Response, ()> {
    match data {
        Ok(output) => {
            let (sender, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(1000);
            tokio::spawn(async move {
                let mut stream = tokio_stream::wrappers::ReceiverStream::new(output.rx);
                while let Some(msg) = stream.next().await {
                    let result = String::from_utf8_lossy(msg.as_slice()).to_string();
                    match sender.send(Ok(Event::default().data(result))).await {
                        Ok(_) => (),
                        Err(e) => {
                            log::debug!("send error: {}", e);
                            break;
                        }
                    }
                }
            });
            Ok(Sse::new(tokio_stream::wrappers::ReceiverStream::new(rx))
                .keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(1))
                        .text("keep-alive-text"),
                )
                .into_response())
        }
        Err(e) => Ok(convert_server_error(e)),
    }
}

pub(crate) fn present_status_ok<A>(data: Result<A>) -> Result<Response, ()> {
    match data {
        Ok(_) => Ok(status_ok_response()),
        Err(e) => Ok(convert_server_error(e)),
    }
}

fn convert_server_error(err: anyhow::Error) -> Response {
    if let Some(usecase_err) = err.downcast_ref::<kernel::error::Error>() {
        match usecase_err {
            kernel::error::Error::BadRequest(type_code, message) => {
                log::warn!("{}", err);
                bad_request(type_code.clone(), message.clone())
            }
            kernel::error::Error::Unauthorized(type_code, message) => {
                log::warn!("{}", err);
                unauthorized(type_code.clone(), message.clone())
            }
            kernel::error::Error::Forbidden(type_code, message) => {
                log::warn!("{}", err);
                forbidden(type_code.clone(), message.clone())
            }
            kernel::error::Error::NotFound(type_code, message) => {
                log::warn!("{}", err);
                not_found(type_code.clone(), message.clone())
            }
            kernel::error::Error::Unexpected(type_code, message) => {
                log::error!("{:?}", err);
                internal_server_error(type_code.clone(), message.clone())
            }
        }
    } else {
        log::error!("{:?}", err);
        internal_server_error(
            Codes::CommonUnexpected,
            Some(status_reason(StatusCode::INTERNAL_SERVER_ERROR)),
        )
    }
}

fn bad_request(type_code: Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::BAD_REQUEST, type_code, message)
}
fn unauthorized(type_code: Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::UNAUTHORIZED, type_code, message)
}
fn forbidden(type_code: Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::FORBIDDEN, type_code, message)
}
fn not_found(type_code: Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::NOT_FOUND, type_code, message)
}
fn internal_server_error(type_code: Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::INTERNAL_SERVER_ERROR, type_code, message)
}

fn server_error_message(code: StatusCode, type_code: Codes, message: Option<String>) -> Response {
    server_response_message(
        code,
        ErrorMessage {
            status: code.as_u16().to_i32().unwrap(),
            r#type: type_code.to_string(),
            message,
        },
    )
}

fn ok_response_with_message<A: Serialize>(message: A) -> Response {
    server_response_message(StatusCode::OK, message)
}

fn server_response_message<A: Serialize>(code: StatusCode, message: A) -> Response
where
    (StatusCode, Json<A>): IntoResponse,
{
    response_with_code(code, Json(message))
}

fn status_reason(code: StatusCode) -> String {
    code.canonical_reason().unwrap().to_string()
}

fn status_ok_response() -> Response {
    ok_response_with_message(StatusOk::new())
}

fn delete_session_cookie() -> Cookie<'static> {
    set_session_cookie("", Some(time::Duration::seconds(0)))
}

fn set_session_cookie(
    session_id: impl Into<String>,
    max_age: Option<time::Duration>,
) -> Cookie<'static> {
    let mut builder = CookieBuilder::new(constants::SESSION_COOKIE_ID, session_id.into());
    if let Some(max_age) = max_age {
        builder = builder.max_age(max_age)
    }
    builder
        .secure(true)
        .path("/")
        .same_site(SameSite::Strict)
        .finish()
}
