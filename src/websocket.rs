use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use hyper::header::HeaderValue;
use hyper::upgrade::Upgraded;
use hyper::{Body, Response, Result};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

#[async_trait]
pub trait WsCallback {
    async fn handshake(
        self,
        upgraded: Upgraded,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
}

pub async fn upgrade_connection(
    req: hyper::Request<Body>,
    callback: impl WsCallback + Send + 'static,
) -> Result<Response<Body>> {
    let mut res = Response::new(Body::empty());

    if !req.headers().contains_key(hyper::header::UPGRADE) {
        *res.status_mut() = hyper::StatusCode::BAD_REQUEST;
        return Ok(res);
    }

    // Follow specification of Headers stated in the WebSocket protocol
    // https://tools.ietf.org/html/rfc6455#section-4.2.2
    *res.status_mut() = hyper::StatusCode::SWITCHING_PROTOCOLS;

    res.headers_mut().insert(
        hyper::header::UPGRADE,
        HeaderValue::from_static("websocket"),
    );
    res.headers_mut().insert(
        hyper::header::CONNECTION,
        HeaderValue::from_static("Upgrade"),
    );
    let sec_websocket_key = req.headers().get(hyper::header::SEC_WEBSOCKET_KEY);
    if let Some(header_value) = sec_websocket_key {
        let mut accept_key = header_value.as_bytes().to_vec();
        let mut websocket_key = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes().to_vec();
        accept_key.append(&mut websocket_key);

        let mut sha1 = sha1::Sha1::new();
        sha1.update(&accept_key[..]);
        let sha1_digest = sha1.digest().bytes();
        let base64encoded = base64::encode(&sha1_digest);
        let sec_websocket_accept = base64encoded.as_str();

        res.headers_mut().insert(
            hyper::header::SEC_WEBSOCKET_ACCEPT,
            HeaderValue::from_str(sec_websocket_accept).unwrap(),
        );
    }

    tokio::task::spawn(async move {
        match req.into_body().on_upgrade().await {
            Ok(upgraded) => {
                if let Err(e) = callback.handshake(upgraded).await {
                    eprintln!("Server error: {}", e);
                }
            }
            Err(e) => eprintln!("upgrade error: {}", e),
        }
    });

    Ok(res)
}

pub struct VersionHandler();

#[async_trait]
impl WsCallback for VersionHandler {
    async fn handshake(
        self,
        upgraded: Upgraded,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let ws_stream = WebSocketStream::from_raw_socket(
            upgraded,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let (mut ws_sender, mut _receiver) = ws_stream.split();

        // Send initial message
        ws_sender
            .send(Message::Text(String::from("Version 0.1.0")))
            .await?;
        ws_sender.send(Message::Close(None)).await?;
        Ok(())
    }
}
