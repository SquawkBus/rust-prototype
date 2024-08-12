use tokio::sync::mpsc::Sender;

use uuid::Uuid;

use common::messages::Message;

pub enum ClientEvent {
    OnConnect(Uuid, String, String, Sender<ServerEvent>),
    OnClose(Uuid),
    OnMessage(Uuid, Message),
}

pub enum ServerEvent {
    OnMessage(Message),
}
