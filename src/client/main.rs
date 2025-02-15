use futures_util::{SinkExt, StreamExt};
use networking::communication::common_message::{ClientToServerMessage, ServerToClientMessage};
use networking::logic::input_parser::{parse_input, InputToken};
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Utf8Bytes;

#[tokio::main]
async fn main() {
    let mut url: String = "ws://".to_string();
    println!("Please enter the server address: ws:// is already included");
    std::io::stdin()
        .read_line(&mut url)
        .expect("Failed to read line");
    let res = connect_async(url.trim()).await;
    if res.is_err() {
        println!("Failed to connect to server: {}, program exits", res.err().unwrap());
        return;
    }
    let (mut ws_stream, _) = res.unwrap();
    let mut reader = BufReader::new(io::stdin());

    println!("Successfully connected to server");

    let username: Option<String> = None;

    loop {
        tokio::select! {
            console_input = get_console_input_tokens(&mut reader) => {
                if let Ok(tokens) = console_input {
                    if tokens.is_empty() {
                        continue;
                    }
                    if let InputToken::General(instruction) = &tokens[0] {
                        match instruction.as_str() {
                            "send" => {
                                if tokens.len() != 3 {
                                    println!("Grammar is not correct, should be: send \"<username>\" \"<message>\"");
                                    continue;
                                }
                                if let (InputToken::String(username), InputToken::String(message))
                                        = (&tokens[1], &tokens[2]) {
                                    let message = ClientToServerMessage::TextTo(username.to_string(), message.to_string());
                                    let message_text = serde_json::to_string(&message).unwrap();

                                    ws_stream
                                        .send(Message::Text(Utf8Bytes::from(message_text)))
                                        .await
                                        .expect("Failed to send message");
                                } else {
                                    println!("Grammar is not correct, should be: send \"<username>\" \"<message>\"");
                                }
                            }

                            "set_name" => {
                                if tokens.len() != 2 {
                                    println!("Grammar is not correct, should be: set_name \"<username>\"");
                                    continue;
                                }
                                if let InputToken::String(username) = &tokens[1] {
                                    let message = ClientToServerMessage::SetUsername(username.to_string());
                                    let message_text = serde_json::to_string(&message).unwrap();

                                    ws_stream
                                        .send(Message::Text(Utf8Bytes::from(message_text)))
                                        .await
                                        .expect("Failed to send message");
                                } else {
                                    println!("Grammar is not correct, should be: set_name \"<username>\"");
                                }
                            }
                            "close" => {
                                ws_stream.close(None).await.expect("Failed to close connection");
                                break;
                            }
                            "usernames" => {
                                let message = ClientToServerMessage::GetUsernames;
                                let message_text = serde_json::to_string(&message).unwrap();

                                ws_stream
                                    .send(Message::Text(Utf8Bytes::from(message_text)))
                                    .await
                                    .expect("Failed to send message");
                            }
                            _ => {
                                println!("Invalid instruction, available instructions are: send, set_name, usernames, close");
                            }
                        }
                    } else {
                        println!("Instruction is not grammatically correct, please try again");
                    }
                } else {
                    println!("Instruction is not grammatically correct, please try again");
                }
            }
            msg = ws_stream.next() => {
                if msg.is_none() {
                    println!("Connection closed: remote host closed abruptly");
                    println!("Press any key to exit");
                    break;
                }
                let msg = msg.unwrap();
                match msg {
                    Ok(Message::Text(text)) => {
                        let text = text.to_string();
                        let message: ServerToClientMessage = serde_json::from_str(&text).unwrap();
                        match message {
                            ServerToClientMessage::TextFrom(username, message) => {
                                println!("Message from {}: {}", username, message);
                            }
                            ServerToClientMessage::Usernames(usernames) => {
                                println!("Usernames: {:?}", usernames);
                            }
                            ServerToClientMessage::Response(Ok(_)) =>{
                                println!("Operation successful");
                            }
                            ServerToClientMessage::Response(Err(e)) =>{
                                println!("Operation failed: {}", e);
                            }
                        _ => {}}
                    }
                    Ok(Message::Close(_)) => {
                        println!("Connection closed: remote host closed the connection");
                        println!("Press any key to exit");
                        break;
                    }
                    Ok(_) => {
                        println!("Received non-text message from server, the client does not know how to parse it");
                    }
                    Err(e) => {
                        println!("Unexpected error: {}", e);
                    }
                }
            }
        }
    }
}

async fn get_console_input_tokens(
    reader: &mut BufReader<io::Stdin>,
) -> Result<Vec<InputToken>, String> {
    let mut input = String::new();

    if reader.read_line(&mut input).await.is_ok() {
        parse_input(input.trim())
    } else {
        Err(String::from("Failed to read line"))
    }
}
