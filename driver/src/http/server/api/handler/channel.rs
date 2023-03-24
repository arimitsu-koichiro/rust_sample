use crate::http::server::api::{ApiMods, ServerPresenter};

use crate::{dispatch, dispatch_with};
use application::interface::Component;
use application::usecase::channel::{
    PubSubInput, PubSubUseCase, PublishInput, PublishUseCase, SubscribeInput, SubscribeUseCase,
};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use helper::uuid;
use helper::uuid::ToBase62;
use kernel::Result;
use log;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;

pub(crate) async fn channel_socket<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    Path(channel_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<Response, ()> {
    Ok(ws
        .protocols(["x-protocol"])
        .on_upgrade(move |socket| handle_socket(api, channel_id, socket)))
}

pub(crate) async fn subscribe_channel<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    Path(channel_id): Path<String>,
) -> Result<Response, ()> {
    dispatch(
        SubscribeUseCase::new(api.clone()),
        SubscribeInput::new(channel_id),
        api.presenter(),
    )
    .await
}

pub(crate) async fn publish_channel<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    Path(channel_id): Path<String>,
    message: String,
) -> Result<Response, ()> {
    dispatch(
        PublishUseCase::new(api.clone()),
        PublishInput::new(channel_id, message.as_bytes().to_vec()),
        api.presenter(),
    )
    .await
}

async fn handle_socket<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    api: M,
    channel_id: String,
    socket: WebSocket,
) {
    let (outbound, mut inbound) = socket.split();
    let outbound = Arc::new(Mutex::new(outbound));
    let _outbound = outbound.clone();
    let (exchange_sender, receiver) = channel::<Vec<u8>>(1000);
    let (sender, exchange_receiver) = channel::<Vec<u8>>(1000);
    let ping_message = uuid::new_v4().to_base62().as_bytes().to_vec();
    let _ping_message = ping_message.clone();
    tokio::spawn(async move {
        while let Some(msg) = inbound.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(err) => {
                    log::debug!("receive inbound message error: {}", err);
                    break;
                }
            };
            match msg {
                Message::Text(msg) => {
                    if let Err(err) = exchange_sender.send(msg.as_bytes().to_vec()).await {
                        log::debug!("send inbound text message error: {}", err);
                    }
                }
                Message::Binary(data) => {
                    if let Err(err) = exchange_sender.send(data).await {
                        log::debug!("send inbound binary message error: {}", err);
                    }
                }
                Message::Ping(x) => {
                    if let Err(err) = _outbound.lock().await.send(Message::Pong(x)).await {
                        log::debug!("send pong error: {}", err);
                    }
                }
                Message::Pong(x) => {
                    if x != _ping_message {
                        log::error!(
                            "invalid pong message. send {}, receive: {}",
                            String::from_utf8_lossy(_ping_message.as_slice()),
                            String::from_utf8_lossy(x.as_slice())
                        );
                        break;
                    }
                }
                Message::Close(_) => {
                    break;
                }
            }
        }
    });
    let _outbound = outbound.clone();
    tokio::spawn(async move {
        let mut stream = tokio_stream::wrappers::ReceiverStream::new(exchange_receiver);
        while let Some(msg) = stream.next().await {
            match _outbound.lock().await.send(Message::Binary(msg)).await {
                Ok(_) => (),
                Err(e) => {
                    log::debug!("send outbound message error: {}", e);
                    break;
                }
            }
        }
    });

    let _outbound = outbound.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10000));
        loop {
            interval.tick().await;
            match _outbound
                .lock()
                .await
                .send(Message::Ping(ping_message.clone()))
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    log::debug!("send heartbeat error: {}", e);
                    break;
                }
            }
        }
    });
    let interactor = PubSubUseCase::new(api.clone());
    let input = PubSubInput::new(channel_id, receiver);
    dispatch_with(interactor, input, api.presenter(), sender).await
}
