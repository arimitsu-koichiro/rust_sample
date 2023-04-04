use crate::http::server::api::{Mods, Presenter};
use crate::http::server::middleware::session::RequireSession;
use crate::{dispatch, dispatch_with};
use application::usecase::channel::{PubSubInput, PublishInput, SubscribeInput};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use futures::{SinkExt, StreamExt};
use helper::uuid;
use helper::uuid::ToBase62;
use kernel::Result;
use log;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;

pub(crate) fn route<M: Mods<P>, P: Presenter>(_: M) -> Router<M> {
    Router::new()
        .route("/channel/:channel_id", get(subscribe_channel::<M, P>))
        .route("/channel/:channel_id", post(publish_channel::<M, P>))
        .route("/channel/:channel_id/socket", get(channel_socket::<M, P>))
}

async fn channel_socket<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Path(channel_id): Path<String>,
    RequireSession(_): RequireSession,
    ws: WebSocketUpgrade,
) -> Response {
    ws.protocols(["x-protocol"])
        .on_upgrade(move |socket| async { handle_socket(mods, channel_id, socket).await })
}

async fn subscribe_channel<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Path(channel_id): Path<String>,
    RequireSession(_): RequireSession,
) -> Result<Response, ()> {
    dispatch(SubscribeInput::new(channel_id), mods).await
}

async fn publish_channel<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Path(channel_id): Path<String>,
    RequireSession(_): RequireSession,
    message: String,
) -> Result<Response, ()> {
    let input = PublishInput::new(channel_id, message.as_bytes().to_vec());
    dispatch(input, mods).await
}

async fn handle_socket<M: Mods<P>, P: Presenter>(mods: M, channel_id: String, socket: WebSocket) {
    let (outbound, mut inbound) = socket.split();
    let outbound = Arc::new(Mutex::new(outbound));
    let (exchange_sender, receiver) = channel::<Vec<u8>>(1000);
    let (sender, exchange_receiver) = channel::<Vec<u8>>(1000);
    let ping_message = uuid::new_v4().to_base62().as_bytes().to_vec();
    let cloned_outbound = outbound.clone();
    let cloned_ping_message = ping_message.clone();
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
                    if let Err(err) = cloned_outbound.lock().await.send(Message::Pong(x)).await {
                        log::debug!("send pong error: {}", err);
                    }
                }
                Message::Pong(x) => {
                    if x != cloned_ping_message {
                        log::error!(
                            "invalid pong message. send {}, receive: {}",
                            String::from_utf8_lossy(cloned_ping_message.as_slice()),
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
    let cloned_outbound = outbound.clone();
    tokio::spawn(async move {
        let mut stream = ReceiverStream::new(exchange_receiver);
        while let Some(msg) = stream.next().await {
            match cloned_outbound
                .lock()
                .await
                .send(Message::Binary(msg))
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    log::debug!("send outbound message error: {}", e);
                    break;
                }
            }
        }
    });

    let cloned_outbound = outbound.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10000));
        loop {
            interval.tick().await;
            match cloned_outbound
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
    let input = PubSubInput::new(channel_id, receiver);
    dispatch_with(input, sender, mods).await;
}
