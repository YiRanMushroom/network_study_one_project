use networking::communication::common_message::{ClientToServerMessage, ServerToClientMessage};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub enum MainToThreadsMessage {
    #[default]
    Shutdown,
    SendToClient(ServerToClientMessage, String),
    Usernames(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub enum ThreadsToMainMessage {
    #[default]
    Shutdown,
    ReceivedFromClient(ClientToServerMessage, String),
    RequestUsernames(String),
}