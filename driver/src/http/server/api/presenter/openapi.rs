use crate::{Present, Presenter};
use application::usecase::account::GetAccountOutput;
use application::usecase::auth::{
    ForgetPasswordOutput, GetAuthStatusOutput, ResetPasswordOutput, SignInOutput, SignOutOutput,
    SignUpFinishOutput, SignUpOutput,
};
use application::usecase::channel::{PubSubOutput, PublishOutput, SubscribeOutput};
use application::usecase::status::StatusOutput;
use async_trait::async_trait;
use axum::response::Response;
use futures::StreamExt;
use kernel::Result;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Default)]
pub struct OpenAPIServerPresenter;

impl Presenter for OpenAPIServerPresenter {}

#[async_trait]
impl Present<Result<StatusOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;
    async fn present(&self, data: Result<StatusOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_status_output(data)
    }
}
#[async_trait]
impl Present<Result<GetAccountOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<GetAccountOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_get_account_output(data)
    }
}
#[async_trait]
impl Present<Result<GetAuthStatusOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<GetAuthStatusOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_status_ok(data)
    }
}
#[async_trait]
impl Present<Result<SignUpOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignUpOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_status_ok(data)
    }
}
#[async_trait]
impl Present<Result<SignUpFinishOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignUpFinishOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_signup_finish_output(data)
    }
}
#[async_trait]
impl Present<Result<SignInOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignInOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_signin_output(data)
    }
}
#[async_trait]
impl Present<Result<SignOutOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SignOutOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_signout_output(data)
    }
}
#[async_trait]
impl Present<Result<ForgetPasswordOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<ForgetPasswordOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_forget_password_output(data)
    }
}
#[async_trait]
impl Present<Result<ResetPasswordOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<ResetPasswordOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_status_ok(data)
    }
}
#[async_trait]
impl Present<Result<PublishOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<PublishOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_status_ok(data)
    }
}
#[async_trait]
impl Present<Result<SubscribeOutput>> for OpenAPIServerPresenter {
    type Output = Result<Response, ()>;

    async fn present(&self, data: Result<SubscribeOutput>, _: ()) -> Self::Output {
        crate::http::server::api::presenter::present_subscribe_output(data)
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
            let mut stream = tokio_stream::wrappers::ReceiverStream::new(output.rx);
            while let Some(msg) = stream.next().await {
                let _ = attachment.send(msg).await;
            }
        });
    }
}
