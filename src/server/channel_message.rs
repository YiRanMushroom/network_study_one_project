use uuid::Uuid;
use networking::communication::common_message::{ClientToServerMessage, ServerToClientMessage};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub enum MainToThreadsMessage {
    #[default]
    Shutdown,
    SendToClient(ServerToClientMessage),
    Usernames(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub enum ThreadsToMainMessage {
    #[default]
    Shutdown,
    ReceivedFromClient(ClientToServerMessage, Uuid),
    ConnectionClosed(Uuid),
}