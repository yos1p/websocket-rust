use crate::websocket::WsCallback;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub struct MessageHandler();

#[async_trait]
impl WsCallback for MessageHandler {
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
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Send initial message
        ws_sender
            .send(Message::Text(String::from("Ready to receive message...")))
            .await?;

        loop {
            match ws_receiver.next().await {
                Some(msg) => {
                    let msg = msg?;
                    if msg.is_text() {
                        ws_sender
                            .send(Message::Text(format!("Received: {}", msg)))
                            .await?;
                    } else if msg.is_ping() {
                        ws_sender
                            .send(Message::Pong(vec![]))
                            .await?
                    } else if msg.is_close() {
                        println!("Closing Websocket");
                        break;
                    }
                }
                None => break,
            }
        }

        Ok(())
    }
}
