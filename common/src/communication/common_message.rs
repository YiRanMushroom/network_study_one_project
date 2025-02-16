use serde::{Deserialize, Serialize};

// send and sync are required for the broadcast channel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub enum ClientToServerMessage {
    #[default]
    None,
    TextTo(String, String),
    GetUsernames,
    SetUsername(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub enum ServerToClientMessage {
    #[default]
    None,
    TextFrom(String, String),
    Usernames(Vec<String>),
    Response(Result<String, String>),
}
