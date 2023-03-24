use futures::{future, future::BoxFuture, future::FutureExt, stream, stream::TryStreamExt, Stream};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use log::warn;
#[allow(unused_imports)]
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
pub use swagger::auth::Authorization;
use swagger::auth::Scopes;
use swagger::{ApiError, BodyExt, Has, RequestParser, XSpanIdString};
use url::form_urlencoded;

use crate::header;
#[allow(unused_imports)]
use crate::models;

pub use crate::context;

type ServiceFuture = BoxFuture<'static, Result<Response<Body>, crate::ServiceError>>;

use crate::{
    Api, ChannelCocketResponse, ForgetPasswordResponse, GetAccountResponse, GetAuthStatusResponse,
    GetStatusResponse, PublishChannelResponse, ResetPasswordResponse, SigninResponse,
    SignoutResponse, SignupFinishResponse, SignupResponse, SubscribeChannelResponse,
};

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/api/v1/account/(?P<account_id>[^/?#]*)$",
            r"^/api/v1/auth/forget_password$",
            r"^/api/v1/auth/reset_password$",
            r"^/api/v1/auth/signin$",
            r"^/api/v1/auth/signout$",
            r"^/api/v1/auth/signup$",
            r"^/api/v1/auth/signup/finish$",
            r"^/api/v1/auth/status$",
            r"^/api/v1/channel/(?P<channel_id>[^/?#]*)$",
            r"^/api/v1/channel/(?P<channel_id>[^/?#]*)/socket$",
            r"^/api/v1/status$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_API_V1_ACCOUNT_ACCOUNT_ID: usize = 0;
    lazy_static! {
        pub static ref REGEX_API_V1_ACCOUNT_ACCOUNT_ID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v1/account/(?P<account_id>[^/?#]*)$")
                .expect("Unable to create regex for API_V1_ACCOUNT_ACCOUNT_ID");
    }
    pub(crate) static ID_API_V1_AUTH_FORGET_PASSWORD: usize = 1;
    pub(crate) static ID_API_V1_AUTH_RESET_PASSWORD: usize = 2;
    pub(crate) static ID_API_V1_AUTH_SIGNIN: usize = 3;
    pub(crate) static ID_API_V1_AUTH_SIGNOUT: usize = 4;
    pub(crate) static ID_API_V1_AUTH_SIGNUP: usize = 5;
    pub(crate) static ID_API_V1_AUTH_SIGNUP_FINISH: usize = 6;
    pub(crate) static ID_API_V1_AUTH_STATUS: usize = 7;
    pub(crate) static ID_API_V1_CHANNEL_CHANNEL_ID: usize = 8;
    lazy_static! {
        pub static ref REGEX_API_V1_CHANNEL_CHANNEL_ID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v1/channel/(?P<channel_id>[^/?#]*)$")
                .expect("Unable to create regex for API_V1_CHANNEL_CHANNEL_ID");
    }
    pub(crate) static ID_API_V1_CHANNEL_CHANNEL_ID_SOCKET: usize = 9;
    lazy_static! {
        pub static ref REGEX_API_V1_CHANNEL_CHANNEL_ID_SOCKET: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v1/channel/(?P<channel_id>[^/?#]*)/socket$")
                .expect("Unable to create regex for API_V1_CHANNEL_CHANNEL_ID_SOCKET");
    }
    pub(crate) static ID_API_V1_STATUS: usize = 10;
}

pub struct MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        MakeService {
            api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C, Target> hyper::service::Service<Target> for MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    type Response = Service<T, C>;
    type Error = crate::ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: Target) -> Self::Future {
        futures::future::ok(Service::new(self.api_impl.clone()))
    }
}

fn method_not_allowed() -> Result<Response<Body>, crate::ServiceError> {
    Ok(Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .expect("Unable to create Method Not Allowed response"))
}

pub struct Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        Service {
            api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C> Clone for Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Service {
            api_impl: self.api_impl.clone(),
            marker: self.marker,
        }
    }
}

impl<T, C> hyper::service::Service<(Request<Body>, C)> for Service<T, C>
where
    T: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = crate::ServiceError;
    type Future = ServiceFuture;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.api_impl.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        async fn run<T, C>(
            mut api_impl: T,
            req: (Request<Body>, C),
        ) -> Result<Response<Body>, crate::ServiceError>
        where
            T: Api<C> + Clone + Send + 'static,
            C: Has<XSpanIdString> + Send + Sync + 'static,
        {
            let (request, context) = req;
            let (parts, body) = request.into_parts();
            let (method, uri, headers) = (parts.method, parts.uri, parts.headers);
            let path = paths::GLOBAL_REGEX_SET.matches(uri.path());

            match method {
                // GetAccount - GET /api/v1/account/{account_id}
                hyper::Method::GET if path.matched(paths::ID_API_V1_ACCOUNT_ACCOUNT_ID) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_API_V1_ACCOUNT_ACCOUNT_ID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE API_V1_ACCOUNT_ACCOUNT_ID in set but failed match against \"{}\"", path, paths::REGEX_API_V1_ACCOUNT_ACCOUNT_ID.as_str())
                    );

                    let param_account_id = match percent_encoding::percent_decode(path_params["account_id"].as_bytes()).decode_utf8() {
                    Ok(param_account_id) => match param_account_id.parse::<String>() {
                        Ok(param_account_id) => param_account_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter account_id: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["account_id"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let result = api_impl.get_account(param_account_id, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                GetAccountResponse::OK(body) => {
                                    *response.status_mut() = StatusCode::from_u16(200)
                                        .expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_ACCOUNT_OK"));
                                    let body = serde_json::to_string(&body)
                                        .expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                GetAccountResponse::Status0(body) => {
                                    *response.status_mut() = StatusCode::from_u16(0)
                                        .expect("Unable to turn 0 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_ACCOUNT_STATUS0"));
                                    let body = serde_json::to_string(&body)
                                        .expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ForgetPassword - POST /api/v1/auth/forget_password
                hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_FORGET_PASSWORD) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_forget_password_request: Option<models::ForgetPasswordRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_forget_password_request) => param_forget_password_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ForgetPasswordRequest - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ForgetPasswordRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_forget_password_request = match param_forget_password_request {
                                    Some(param_forget_password_request) => param_forget_password_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ForgetPasswordRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ForgetPasswordRequest")),
                                };

                                let result = api_impl.forget_password(
                                            param_forget_password_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {unused_elements:?}").as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ForgetPasswordResponse::OK
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for FORGET_PASSWORD_OK"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ForgetPasswordResponse::Status0
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for FORGET_PASSWORD_STATUS0"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ForgetPasswordRequest: {e}")))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ForgetPasswordRequest")),
                        }
                }

                // GetAuthStatus - GET /api/v1/auth/status
                hyper::Method::GET if path.matched(paths::ID_API_V1_AUTH_STATUS) => {
                    let result = api_impl.get_auth_status(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            GetAuthStatusResponse::OK(body) => {
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_AUTH_STATUS_OK"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetAuthStatusResponse::Status0(body) => {
                                *response.status_mut() = StatusCode::from_u16(0)
                                    .expect("Unable to turn 0 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_AUTH_STATUS_STATUS0"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ResetPassword - POST /api/v1/auth/reset_password
                hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_RESET_PASSWORD) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_reset_password_request: Option<models::ResetPasswordRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_reset_password_request) => param_reset_password_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ResetPasswordRequest - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ResetPasswordRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_reset_password_request = match param_reset_password_request {
                                    Some(param_reset_password_request) => param_reset_password_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ResetPasswordRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ResetPasswordRequest")),
                                };

                                let result = api_impl.reset_password(
                                            param_reset_password_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {unused_elements:?}").as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ResetPasswordResponse::OK
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RESET_PASSWORD_OK"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ResetPasswordResponse::Status0
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RESET_PASSWORD_STATUS0"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ResetPasswordRequest: {e}")))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ResetPasswordRequest")),
                        }
                }

                // Signin - POST /api/v1/auth/signin
                hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNIN) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_signin_request: Option<models::SigninRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_signin_request) => param_signin_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SigninRequest - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SigninRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_signin_request = match param_signin_request {
                                    Some(param_signin_request) => param_signin_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SigninRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SigninRequest")),
                                };

                                let result = api_impl.signin(
                                            param_signin_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {unused_elements:?}").as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                SigninResponse::OK
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SIGNIN_OK"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SigninResponse::Status0
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SIGNIN_STATUS0"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter SigninRequest: {e}")))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SigninRequest")),
                        }
                }

                // Signout - POST /api/v1/auth/signout
                hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNOUT) => {
                    let result = api_impl.signout(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                SignoutResponse::OK(body) => {
                                    *response.status_mut() = StatusCode::from_u16(200)
                                        .expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for SIGNOUT_OK",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body)
                                        .expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                SignoutResponse::Status0(body) => {
                                    *response.status_mut() = StatusCode::from_u16(0)
                                        .expect("Unable to turn 0 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SIGNOUT_STATUS0"));
                                    let body = serde_json::to_string(&body)
                                        .expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // Signup - POST /api/v1/auth/signup
                hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNUP) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sign_up_request: Option<models::SignUpRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sign_up_request) => param_sign_up_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SignUpRequest - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SignUpRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_sign_up_request = match param_sign_up_request {
                                    Some(param_sign_up_request) => param_sign_up_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SignUpRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SignUpRequest")),
                                };

                                let result = api_impl.signup(
                                            param_sign_up_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {unused_elements:?}").as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                SignupResponse::OK
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SIGNUP_OK"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SignupResponse::Status0
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SIGNUP_STATUS0"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter SignUpRequest: {e}")))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SignUpRequest")),
                        }
                }

                // SignupFinish - POST /api/v1/auth/signup/finish
                hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNUP_FINISH) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sign_up_finish_request: Option<models::SignUpFinishRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sign_up_finish_request) => param_sign_up_finish_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SignUpFinishRequest - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SignUpFinishRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_sign_up_finish_request = match param_sign_up_finish_request {
                                    Some(param_sign_up_finish_request) => param_sign_up_finish_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SignUpFinishRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SignUpFinishRequest")),
                                };

                                let result = api_impl.signup_finish(
                                            param_sign_up_finish_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {unused_elements:?}").as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                SignupFinishResponse::OK
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SIGNUP_FINISH_OK"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SignupFinishResponse::Status0
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SIGNUP_FINISH_STATUS0"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter SignUpFinishRequest: {e}")))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SignUpFinishRequest")),
                        }
                }

                // ChannelCocket - GET /api/v1/channel/{channel_id}/socket
                hyper::Method::GET if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID_SOCKET) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_API_V1_CHANNEL_CHANNEL_ID_SOCKET
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE API_V1_CHANNEL_CHANNEL_ID_SOCKET in set but failed match against \"{}\"", path, paths::REGEX_API_V1_CHANNEL_CHANNEL_ID_SOCKET.as_str())
                    );

                    let param_channel_id = match percent_encoding::percent_decode(path_params["channel_id"].as_bytes()).decode_utf8() {
                    Ok(param_channel_id) => match param_channel_id.parse::<String>() {
                        Ok(param_channel_id) => param_channel_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channel_id: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["channel_id"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let result = api_impl.channel_cocket(param_channel_id, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            ChannelCocketResponse::OK(body) => {
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNEL_COCKET_OK"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelCocketResponse::Status0(body) => {
                                *response.status_mut() = StatusCode::from_u16(0)
                                    .expect("Unable to turn 0 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNEL_COCKET_STATUS0"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // PublishChannel - POST /api/v1/channel/{channel_id}
                hyper::Method::POST if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_API_V1_CHANNEL_CHANNEL_ID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE API_V1_CHANNEL_CHANNEL_ID in set but failed match against \"{}\"", path, paths::REGEX_API_V1_CHANNEL_CHANNEL_ID.as_str())
                    );

                    let param_channel_id = match percent_encoding::percent_decode(path_params["channel_id"].as_bytes()).decode_utf8() {
                    Ok(param_channel_id) => match param_channel_id.parse::<String>() {
                        Ok(param_channel_id) => param_channel_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channel_id: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["channel_id"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_channel_message: Option<models::ChannelMessage> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_channel_message) => param_channel_message,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ChannelMessage - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ChannelMessage due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_channel_message = match param_channel_message {
                                    Some(param_channel_message) => param_channel_message,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ChannelMessage"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ChannelMessage")),
                                };

                                let result = api_impl.publish_channel(
                                            param_channel_id,
                                            param_channel_message,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {unused_elements:?}").as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                PublishChannelResponse::OK
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PUBLISH_CHANNEL_OK"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PublishChannelResponse::Status0
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PUBLISH_CHANNEL_STATUS0"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ChannelMessage: {e}")))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ChannelMessage")),
                        }
                }

                // SubscribeChannel - GET /api/v1/channel/{channel_id}
                hyper::Method::GET if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_API_V1_CHANNEL_CHANNEL_ID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE API_V1_CHANNEL_CHANNEL_ID in set but failed match against \"{}\"", path, paths::REGEX_API_V1_CHANNEL_CHANNEL_ID.as_str())
                    );

                    let param_channel_id = match percent_encoding::percent_decode(path_params["channel_id"].as_bytes()).decode_utf8() {
                    Ok(param_channel_id) => match param_channel_id.parse::<String>() {
                        Ok(param_channel_id) => param_channel_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channel_id: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["channel_id"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let result = api_impl.subscribe_channel(param_channel_id, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            SubscribeChannelResponse::OK(body) => {
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_CHANNEL_OK"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SubscribeChannelResponse::Status0(body) => {
                                *response.status_mut() = StatusCode::from_u16(0)
                                    .expect("Unable to turn 0 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_CHANNEL_STATUS0"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // GetStatus - GET /api/v1/status
                hyper::Method::GET if path.matched(paths::ID_API_V1_STATUS) => {
                    let result = api_impl.get_status(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                GetStatusResponse::OK(body) => {
                                    *response.status_mut() = StatusCode::from_u16(200)
                                        .expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_STATUS_OK"));
                                    let body = serde_json::to_string(&body)
                                        .expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                GetStatusResponse::Status0(body) => {
                                    *response.status_mut() = StatusCode::from_u16(0)
                                        .expect("Unable to turn 0 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_STATUS_STATUS0"));
                                    let body = serde_json::to_string(&body)
                                        .expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                _ if path.matched(paths::ID_API_V1_ACCOUNT_ACCOUNT_ID) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_AUTH_FORGET_PASSWORD) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_AUTH_RESET_PASSWORD) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_AUTH_SIGNIN) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_AUTH_SIGNOUT) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_AUTH_SIGNUP) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_AUTH_SIGNUP_FINISH) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_AUTH_STATUS) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID) => method_not_allowed(),
                _ if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID_SOCKET) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_API_V1_STATUS) => method_not_allowed(),
                _ => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .expect("Unable to create Not Found response")),
            }
        }
        Box::pin(run(self.api_impl.clone(), req))
    }
}

/// Request parser for `Api`.
pub struct ApiRequestParser;
impl<T> RequestParser<T> for ApiRequestParser {
    fn parse_operation_id(request: &Request<T>) -> Option<&'static str> {
        let path = paths::GLOBAL_REGEX_SET.matches(request.uri().path());
        match *request.method() {
            // GetAccount - GET /api/v1/account/{account_id}
            hyper::Method::GET if path.matched(paths::ID_API_V1_ACCOUNT_ACCOUNT_ID) => {
                Some("GetAccount")
            }
            // ForgetPassword - POST /api/v1/auth/forget_password
            hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_FORGET_PASSWORD) => {
                Some("ForgetPassword")
            }
            // GetAuthStatus - GET /api/v1/auth/status
            hyper::Method::GET if path.matched(paths::ID_API_V1_AUTH_STATUS) => {
                Some("GetAuthStatus")
            }
            // ResetPassword - POST /api/v1/auth/reset_password
            hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_RESET_PASSWORD) => {
                Some("ResetPassword")
            }
            // Signin - POST /api/v1/auth/signin
            hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNIN) => Some("Signin"),
            // Signout - POST /api/v1/auth/signout
            hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNOUT) => Some("Signout"),
            // Signup - POST /api/v1/auth/signup
            hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNUP) => Some("Signup"),
            // SignupFinish - POST /api/v1/auth/signup/finish
            hyper::Method::POST if path.matched(paths::ID_API_V1_AUTH_SIGNUP_FINISH) => {
                Some("SignupFinish")
            }
            // ChannelCocket - GET /api/v1/channel/{channel_id}/socket
            hyper::Method::GET if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID_SOCKET) => {
                Some("ChannelCocket")
            }
            // PublishChannel - POST /api/v1/channel/{channel_id}
            hyper::Method::POST if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID) => {
                Some("PublishChannel")
            }
            // SubscribeChannel - GET /api/v1/channel/{channel_id}
            hyper::Method::GET if path.matched(paths::ID_API_V1_CHANNEL_CHANNEL_ID) => {
                Some("SubscribeChannel")
            }
            // GetStatus - GET /api/v1/status
            hyper::Method::GET if path.matched(paths::ID_API_V1_STATUS) => Some("GetStatus"),
            _ => None,
        }
    }
}
