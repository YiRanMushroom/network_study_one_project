mod channel_message;

use crate::channel_message::{MainToThreadsMessage, ThreadsToMainMessage};
use futures_util::{SinkExt, StreamExt};
use networking::communication::common_message::{ClientToServerMessage, ServerToClientMessage};
use networking::logic::input_parser::{parse_input, InputToken};
use std::io;
use std::io::BufRead;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let mut input = String::new();
    println!("Enter the address to bind to: ");
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input = std::mem::take(&mut input).trim().to_string();

    let listener = TcpListener::bind(&input).await.expect("Failed to bind");

    println!("Listening on: {}", input);

    println!("Please follow the instructions to interact with the server.");

    input.clear();

    let (thread_to_main_tx, mut thread_to_main_rx) = broadcast::channel(1);

    let thread_to_main_tx_clone = thread_to_main_tx.clone();
    tokio::spawn(async move {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                let tokens = parse_input(&line);
                match tokens {
                    Ok(mut tokens) => match std::mem::take(&mut tokens[0]) {
                        InputToken::General(str) if str == "close" => {
                            thread_to_main_tx_clone
                                .send(ThreadsToMainMessage::Shutdown)
                                .expect("Failed to send shutdown signal");
                            break;
                        }
                        _ => {
                            println!("Invalid command: {:?}", tokens[0]);
                        }
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    });

    let mut username_to_uuid_map: std::collections::HashMap<String, Uuid> =
        std::collections::HashMap::new();

    struct UserEssential {
        uuid: Uuid,
        main_to_thread_tx: tokio::sync::mpsc::Sender<MainToThreadsMessage>,
        username: Option<String>,
    }

    let mut uuid_to_user_essential_map: std::collections::HashMap<Uuid, UserEssential> =
        std::collections::HashMap::new();

    loop {
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                let connection_id = Uuid::new_v4();
                let (main_to_thread_tx, main_to_thread_rx) = tokio::sync::mpsc::channel(1);
                uuid_to_user_essential_map.insert(connection_id, UserEssential {
                    uuid : connection_id,
                    main_to_thread_tx,
                    username : None,
                });
                tokio::spawn(handle_connection(stream, connection_id,
                    main_to_thread_rx, thread_to_main_tx.clone()));
            },

            message = thread_to_main_rx.recv() => {
                match message {
                    Ok(ThreadsToMainMessage::Shutdown) => {
                        for (uuid, user_essential) in uuid_to_user_essential_map.iter() {
                            user_essential.main_to_thread_tx.send(MainToThreadsMessage::Shutdown)
                                .await.expect("Failed to send shutdown signal");
                        }
                        println!("Shutting down server");
                        break;
                    }
                    Ok(ThreadsToMainMessage::ReceivedFromClient(message, requester_uuid)) => {

                        println!("Received message from {}: {:?}", requester_uuid, message);

                        match message {

                            ClientToServerMessage::SetUsername(username) => {
                                let requester_essential : &mut UserEssential =
                                    uuid_to_user_essential_map.get_mut(&requester_uuid)
                                    .expect("Failed to find user essential");

                                if username_to_uuid_map.contains_key(&username) {
                                    requester_essential.main_to_thread_tx.send(MainToThreadsMessage::SendToClient(
                                    ServerToClientMessage::Response(Err("Username already exists!".to_string()))))
                                    .await
                                    .unwrap_or_else(|e|
                                        println!("Failed to send message to client: {}", e));

                                } else {

                                    username_to_uuid_map.insert(username.clone(), requester_uuid);

                                    (*requester_essential).username = Some(username.clone());

                                    requester_essential.main_to_thread_tx.send(MainToThreadsMessage::SendToClient(
                                    ServerToClientMessage::Response(Ok(format!("Set username {} successfully!",
                                        username)))))
                                    .await
                                    .unwrap_or_else(|e|
                                        println!("Failed to send message to client: {}", e));
                                }
                            }

                            ClientToServerMessage::GetUsernames => {
                                let mut usernames = Vec::new();

                                for (username, _) in username_to_uuid_map.iter() {
                                    usernames.push(username.clone());
                                }

                                let user_essential = uuid_to_user_essential_map.get(&requester_uuid)
                                    .expect("Failed to find user essential");

                                user_essential.main_to_thread_tx
                                    .send(MainToThreadsMessage::SendToClient(ServerToClientMessage::Usernames(usernames)))
                                    .await
                                    .expect("Failed to send message to client");
                            }

                            ClientToServerMessage::TextTo(username, text) => {

                                let user_essential = uuid_to_user_essential_map
                                    .get(&requester_uuid)
                                    .expect("Failed to find user essential");

                                let recipient_uuid = username_to_uuid_map.get(&username);

                                if recipient_uuid.is_none() {
                                    user_essential.main_to_thread_tx.send(MainToThreadsMessage::SendToClient(
                                    ServerToClientMessage::Response(Err("Recipient does not exist!".to_string()))))
                                    .await
                                    .unwrap_or_else(|e|
                                        println!("Failed to send message to client: {}", e));
                                    continue;
                                }

                                let recipient_uuid = recipient_uuid.unwrap();

                                let recipient_user_essential = uuid_to_user_essential_map.get(recipient_uuid)
                                    .expect("Failed to find recipient user essential");

                                recipient_user_essential.main_to_thread_tx.send(MainToThreadsMessage::SendToClient(
                                    ServerToClientMessage::TextFrom(user_essential.username.clone().unwrap(), text)))
                                    .await
                                    .unwrap_or_else(|e|
                                        println!("Failed to send message to client: {}", e));

                                user_essential.main_to_thread_tx.send(MainToThreadsMessage::SendToClient(
                                    ServerToClientMessage::Response(Ok(format!("Sent message to {}", username)))))
                                    .await
                                    .unwrap_or_else(|e|
                                        println!("Failed to send message to client: {}", e));

                            }
                            _ => {}
                        }
                    }

                    Ok(ThreadsToMainMessage::ConnectionClosed(uuid)) => {
                        let user_essential = uuid_to_user_essential_map.remove(&uuid)
                            .expect("Failed to find user essential");
                        username_to_uuid_map.remove(&user_essential.username.unwrap());
                    }

                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    connection_id: Uuid,
    mut main_to_thread_rx: Receiver<MainToThreadsMessage>,
    mut thread_to_main_tx: Sender<ThreadsToMainMessage>,
) {
    match accept_async(stream).await {
        Ok(ws_stream) => {

            println!("New WebSocket connection: {}", connection_id);

            let (mut write, mut read) = ws_stream.split();

            loop {
                tokio::select! {
                    message = read.next() => {
                        match message {
                            Some(Ok(msg)) => {
                                match msg {
                                    Message::Close(_) => {
                                        println!("Connection {} closing", connection_id);
                                        break;
                                    }
                                    Message::Text(text) => {
                                        let message : ClientToServerMessage = serde_json::from_str(&text)
                                            .expect("Failed to parse message");
                                        thread_to_main_tx.send(ThreadsToMainMessage::ReceivedFromClient(message, connection_id)).expect("Failed to send message to main thread");
                                    }
                                _ => {
                                    println!("Received non-text message from connection {}", connection_id);
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                println!("Error on connection {}: {}", connection_id, e);
                                break;
                            }
                            None => {
                                println!("Connection {} closed by client", connection_id);
                                break;
                            }
                        }
                    }
                    channel_message = main_to_thread_rx.recv() => {
                        match channel_message {
                            Some(MainToThreadsMessage::Shutdown) => {
                                println!("Shutting down connection {}", connection_id);
                                write.send(Message::Close(None)).await.expect("Failed to send close message");
                                break;
                            }
                            Some(MainToThreadsMessage::SendToClient(message)) => {
                                println!("Sending message to client: {:?}", message);
                                write.send(Message::Text(Utf8Bytes::from(serde_json::to_string(&message).expect("Failed to serialize message")))).await.expect("Failed to send message to client");
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!(
                "Error during the websocket handshake for connection {}: {:?}",
                connection_id, e
            );
        }
    }
    thread_to_main_tx
        .send(ThreadsToMainMessage::ConnectionClosed(connection_id))
        .expect("Failed to send shutdown signal");
    println!("Connection {} closed", connection_id);
    return;
}
