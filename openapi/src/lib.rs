#![allow(clippy::pedantic)]
#![allow(clippy::all)]
#![allow(
    missing_docs,
    trivial_casts,
    unused_variables,
    unused_mut,
    unused_imports,
    unused_extern_crates,
    non_camel_case_types
)]
#![allow(unused_imports, unused_attributes)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::disallowed_names)]

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::task::{Context, Poll};
use swagger::{ApiError, ContextWrapper};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "";
pub const API_VERSION: &str = "1.0.0";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum GetAccountResponse {
    /// OK
    OK(models::Account),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ForgetPasswordResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum GetAuthStatusResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ResetPasswordResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SigninResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SignoutResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SignupResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SignupFinishResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelCocketResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PublishChannelResponse {
    /// OK
    OK(models::StatusOk),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SubscribeChannelResponse {
    /// OK
    OK(models::ChannelMessage),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum GetStatusResponse {
    /// OK
    OK(models::StatusResponse),
    /// デフォルトのエラーレスポンス
    Status0(models::ErrorMessage),
}

/// API
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    async fn get_account(
        &self,
        account_id: String,
        context: &C,
    ) -> Result<GetAccountResponse, ApiError>;

    async fn forget_password(
        &self,
        forget_password_request: models::ForgetPasswordRequest,
        context: &C,
    ) -> Result<ForgetPasswordResponse, ApiError>;

    async fn get_auth_status(&self, context: &C) -> Result<GetAuthStatusResponse, ApiError>;

    async fn reset_password(
        &self,
        reset_password_request: models::ResetPasswordRequest,
        context: &C,
    ) -> Result<ResetPasswordResponse, ApiError>;

    async fn signin(
        &self,
        signin_request: models::SigninRequest,
        context: &C,
    ) -> Result<SigninResponse, ApiError>;

    async fn signout(&self, context: &C) -> Result<SignoutResponse, ApiError>;

    async fn signup(
        &self,
        sign_up_request: models::SignUpRequest,
        context: &C,
    ) -> Result<SignupResponse, ApiError>;

    async fn signup_finish(
        &self,
        sign_up_finish_request: models::SignUpFinishRequest,
        context: &C,
    ) -> Result<SignupFinishResponse, ApiError>;

    async fn channel_cocket(
        &self,
        channel_id: String,
        context: &C,
    ) -> Result<ChannelCocketResponse, ApiError>;

    async fn publish_channel(
        &self,
        channel_id: String,
        channel_message: models::ChannelMessage,
        context: &C,
    ) -> Result<PublishChannelResponse, ApiError>;

    async fn subscribe_channel(
        &self,
        channel_id: String,
        context: &C,
    ) -> Result<SubscribeChannelResponse, ApiError>;

    async fn get_status(&self, context: &C) -> Result<GetStatusResponse, ApiError>;
}

/// API where `Context` isn't passed on every API call
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    async fn get_account(&self, account_id: String) -> Result<GetAccountResponse, ApiError>;

    async fn forget_password(
        &self,
        forget_password_request: models::ForgetPasswordRequest,
    ) -> Result<ForgetPasswordResponse, ApiError>;

    async fn get_auth_status(&self) -> Result<GetAuthStatusResponse, ApiError>;

    async fn reset_password(
        &self,
        reset_password_request: models::ResetPasswordRequest,
    ) -> Result<ResetPasswordResponse, ApiError>;

    async fn signin(
        &self,
        signin_request: models::SigninRequest,
    ) -> Result<SigninResponse, ApiError>;

    async fn signout(&self) -> Result<SignoutResponse, ApiError>;

    async fn signup(
        &self,
        sign_up_request: models::SignUpRequest,
    ) -> Result<SignupResponse, ApiError>;

    async fn signup_finish(
        &self,
        sign_up_finish_request: models::SignUpFinishRequest,
    ) -> Result<SignupFinishResponse, ApiError>;

    async fn channel_cocket(&self, channel_id: String) -> Result<ChannelCocketResponse, ApiError>;

    async fn publish_channel(
        &self,
        channel_id: String,
        channel_message: models::ChannelMessage,
    ) -> Result<PublishChannelResponse, ApiError>;

    async fn subscribe_channel(
        &self,
        channel_id: String,
    ) -> Result<SubscribeChannelResponse, ApiError>;

    async fn get_status(&self) -> Result<GetStatusResponse, ApiError>;
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    async fn get_account(&self, account_id: String) -> Result<GetAccountResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_account(account_id, &context).await
    }

    async fn forget_password(
        &self,
        forget_password_request: models::ForgetPasswordRequest,
    ) -> Result<ForgetPasswordResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .forget_password(forget_password_request, &context)
            .await
    }

    async fn get_auth_status(&self) -> Result<GetAuthStatusResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_auth_status(&context).await
    }

    async fn reset_password(
        &self,
        reset_password_request: models::ResetPasswordRequest,
    ) -> Result<ResetPasswordResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .reset_password(reset_password_request, &context)
            .await
    }

    async fn signin(
        &self,
        signin_request: models::SigninRequest,
    ) -> Result<SigninResponse, ApiError> {
        let context = self.context().clone();
        self.api().signin(signin_request, &context).await
    }

    async fn signout(&self) -> Result<SignoutResponse, ApiError> {
        let context = self.context().clone();
        self.api().signout(&context).await
    }

    async fn signup(
        &self,
        sign_up_request: models::SignUpRequest,
    ) -> Result<SignupResponse, ApiError> {
        let context = self.context().clone();
        self.api().signup(sign_up_request, &context).await
    }

    async fn signup_finish(
        &self,
        sign_up_finish_request: models::SignUpFinishRequest,
    ) -> Result<SignupFinishResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .signup_finish(sign_up_finish_request, &context)
            .await
    }

    async fn channel_cocket(&self, channel_id: String) -> Result<ChannelCocketResponse, ApiError> {
        let context = self.context().clone();
        self.api().channel_cocket(channel_id, &context).await
    }

    async fn publish_channel(
        &self,
        channel_id: String,
        channel_message: models::ChannelMessage,
    ) -> Result<PublishChannelResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .publish_channel(channel_id, channel_message, &context)
            .await
    }

    async fn subscribe_channel(
        &self,
        channel_id: String,
    ) -> Result<SubscribeChannelResponse, ApiError> {
        let context = self.context().clone();
        self.api().subscribe_channel(channel_id, &context).await
    }

    async fn get_status(&self) -> Result<GetStatusResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_status(&context).await
    }
}

#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(feature = "server")]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;
