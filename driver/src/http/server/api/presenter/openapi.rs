use crate::http::server::response::{constants, response_with_code, WithSetCookie};
use crate::Present;
use ::openapi::models::{Account, ErrorMessage, StatusOk, StatusResponse};
use application::usecase::account::GetAccountOutput;
use application::usecase::auth::{
    ForgetPasswordOutput, GetAuthStatusOutput, ResetPasswordOutput, SignInOutput, SignOutOutput,
    SignUpFinishOutput, SignUpOutput,
};
use application::usecase::channel::{PubSubOutput, PublishOutput, SubscribeOutput};
use application::usecase::status::StatusOutput;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::sse::Event;
use axum::response::{IntoResponse, Response, Sse};
use axum::Json;
use cookie::{Cookie, CookieBuilder, SameSite};
use futures::StreamExt;
use kernel::error::Codes;
use kernel::Result;
use log;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Default)]
pub struct OpenAPIServerPresenter;

#[async_trait]
impl Present<Result<StatusOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;
    async fn present(&self, data: Result<StatusOutput>, _: ()) -> Self::Output {
        Ok(present_status_output(data))
    }
}
#[async_trait]
impl Present<Result<GetAccountOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<GetAccountOutput>, _: ()) -> Self::Output {
        Ok(present_get_account_output(data))
    }
}
#[async_trait]
impl Present<Result<GetAuthStatusOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<GetAuthStatusOutput>, _: ()) -> Self::Output {
        Ok(present_status_ok(data))
    }
}
#[async_trait]
impl Present<Result<SignUpOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignUpOutput>, _: ()) -> Self::Output {
        Ok(present_status_ok(data))
    }
}
#[async_trait]
impl Present<Result<SignUpFinishOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignUpFinishOutput>, _: ()) -> Self::Output {
        Ok(present_signup_finish_output(data))
    }
}
#[async_trait]
impl Present<Result<SignInOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignInOutput>, _: ()) -> Self::Output {
        Ok(present_signin_output(data))
    }
}
#[async_trait]
impl Present<Result<SignOutOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignOutOutput>, _: ()) -> Self::Output {
        Ok(present_signout_output(data))
    }
}
#[async_trait]
impl Present<Result<ForgetPasswordOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<ForgetPasswordOutput>, _: ()) -> Self::Output {
        Ok(present_forget_password_output(data))
    }
}
#[async_trait]
impl Present<Result<ResetPasswordOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<ResetPasswordOutput>, _: ()) -> Self::Output {
        Ok(present_status_ok(data))
    }
}
#[async_trait]
impl Present<Result<PublishOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<PublishOutput>, _: ()) -> Self::Output {
        Ok(present_status_ok(data))
    }
}
#[async_trait]
impl Present<Result<SubscribeOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SubscribeOutput>, _: ()) -> Self::Output {
        Ok(present_subscribe_output(data))
    }
}

#[async_trait]
impl Present<Result<PubSubOutput>, Sender<Vec<u8>>> for OpenAPIServerPresenter {
    type Output = ();

    async fn present(
        &self,
        data: Result<PubSubOutput>,
        attachment: Sender<Vec<u8>>,
    ) -> Self::Output {
        let Ok(output) = data else {
            return;
        };
        tokio::spawn(async move {
            let mut stream = ReceiverStream::new(output.rx);
            while let Some(msg) = stream.next().await {
                let _ = attachment.send(msg).await;
            }
        });
    }
}

fn present_status_output(data: Result<StatusOutput>) -> Response {
    match data {
        Ok(s) => ok_response_with_message(StatusResponse::new(
            status_reason(StatusCode::OK),
            s.version.unwrap_or_else(|| "-".to_string()),
            s.build_timestamp,
        )),
        Err(e) => convert_server_error(&e),
    }
}

fn present_get_account_output(data: Result<GetAccountOutput>) -> Response {
    match data {
        Ok(output) => {
            let Some(account) = output.account else  {
                return not_found(&Codes::CommonNotFound, Some("account not found".to_string()));
            };
            ok_response_with_message(Account::new(account.id, account.name, account.display_name))
        }
        Err(e) => convert_server_error(&e),
    }
}

fn present_signin_output(data: Result<SignInOutput>) -> Response {
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
            status_ok_response().with_cookie(set_session_cookie(session_id, max_age))
        }
        Err(e) => convert_server_error(&e),
    }
}

fn present_signup_finish_output(data: Result<SignUpFinishOutput>) -> Response {
    match data {
        Ok(output) => {
            let SignUpFinishOutput { session_id } = output;
            status_ok_response().with_cookie(set_session_cookie(session_id, None))
        }
        Err(e) => convert_server_error(&e),
    }
}
fn present_signout_output(data: Result<SignOutOutput>) -> Response {
    match data {
        Ok(_) => status_ok_response().with_cookie(delete_session_cookie()),
        Err(e) => convert_server_error(&e),
    }
}
fn present_forget_password_output(data: Result<ForgetPasswordOutput>) -> Response {
    match data {
        Ok(_) => status_ok_response().with_cookie(delete_session_cookie()),
        Err(e) => convert_server_error(&e),
    }
}
fn present_subscribe_output(data: Result<SubscribeOutput>) -> Response {
    match data {
        Ok(output) => {
            let (sender, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(1000);
            tokio::spawn(async move {
                let mut stream = ReceiverStream::new(output.rx);
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
            Sse::new(ReceiverStream::new(rx))
                .keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(1))
                        .text("keep-alive-text"),
                )
                .into_response()
        }
        Err(e) => convert_server_error(&e),
    }
}

fn present_status_ok<A>(data: Result<A>) -> Response {
    match data {
        Ok(_) => status_ok_response(),
        Err(e) => convert_server_error(&e),
    }
}

fn convert_server_error(err: &anyhow::Error) -> Response {
    if let Some(usecase_err) = err.downcast_ref::<kernel::Error>() {
        match usecase_err {
            kernel::Error::BadRequest(type_code, message) => {
                log::warn!("{}", err);
                bad_request(type_code, message.clone())
            }
            kernel::Error::Unauthorized(type_code, message) => {
                log::warn!("{}", err);
                unauthorized(type_code, message.clone())
            }
            kernel::Error::Forbidden(type_code, message) => {
                log::warn!("{}", err);
                forbidden(type_code, message.clone())
            }
            kernel::Error::NotFound(type_code, message) => {
                log::warn!("{}", err);
                not_found(type_code, message.clone())
            }
            kernel::Error::Unexpected(type_code, message) => {
                log::error!("{:?}", err);
                internal_server_error(type_code, message.clone())
            }
        }
    } else {
        log::error!("{:?}", err);
        internal_server_error(
            &Codes::CommonUnexpected,
            Some(status_reason(StatusCode::INTERNAL_SERVER_ERROR)),
        )
    }
}

fn bad_request(type_code: &Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::BAD_REQUEST, type_code, message)
}
fn unauthorized(type_code: &Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::UNAUTHORIZED, type_code, message)
}
fn forbidden(type_code: &Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::FORBIDDEN, type_code, message)
}
fn not_found(type_code: &Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::NOT_FOUND, type_code, message)
}
fn internal_server_error(type_code: &Codes, message: Option<String>) -> Response {
    server_error_message(StatusCode::INTERNAL_SERVER_ERROR, type_code, message)
}

fn server_error_message(code: StatusCode, type_code: &Codes, message: Option<String>) -> Response {
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
        builder = builder.max_age(max_age);
    }
    builder
        .secure(true)
        .path("/")
        .same_site(SameSite::Strict)
        .finish()
}
