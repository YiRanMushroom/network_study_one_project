use futures_util::{SinkExt, StreamExt};
use networking::communication::common_message::ClientToServerMessage;
use serde::Serialize;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Utf8Bytes;

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:8080";
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("WebSocket handshake has been successfully completed");

    let set_name_message = ClientToServerMessage::SetUsername("Yiran".to_string());

    let set_name_message_text = serde_json::to_string(&set_name_message).unwrap();

    ws_stream
        .send(Message::Text(Utf8Bytes::from(set_name_message_text)))
        .await
        .expect("Failed to send message");

    if let Some(msg) = ws_stream.next().await {
        let msg = msg.expect("Failed to read message");
        println!("Received: {}", msg);
    }

    ws_stream
        .send(Message::Close(None))
        .await
        .expect("Failed to send close message");
}
