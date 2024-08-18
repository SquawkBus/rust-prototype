use std::{collections::HashMap, io};

use tokio::sync::mpsc::{Receiver, Sender};

use uuid::Uuid;

use common::messages::Message;

use crate::{
    authorization::Authorization,
    authorization::AuthorizationManager,
    clients::ClientManager,
    events::{ClientEvent, ServerEvent},
    notifications::NotificationManager,
    publishing::PublisherManager,
    subscriptions::SubscriptionManager,
};

pub struct Hub {
    client_manager: ClientManager,
    subscription_manager: SubscriptionManager,
    notification_manager: NotificationManager,
    publisher_manager: PublisherManager,
    entitlement_manager: AuthorizationManager,
}

impl Hub {
    pub fn new(entitlement_manager: AuthorizationManager) -> Hub {
        Hub {
            client_manager: ClientManager::new(),
            subscription_manager: SubscriptionManager::new(),
            notification_manager: NotificationManager::new(),
            publisher_manager: PublisherManager::new(),
            entitlement_manager,
        }
    }

    pub async fn run(
        authorizations: HashMap<String, HashMap<String, Authorization>>,
        mut server_rx: Receiver<ClientEvent>,
    ) -> io::Result<()> {
        let entitlement_manager: AuthorizationManager =
            AuthorizationManager::from_config(authorizations).expect("Should load authorizations");

        let mut hub = Self::new(entitlement_manager);
        loop {
            let msg = server_rx.recv().await.unwrap();
            match msg {
                ClientEvent::OnMessage(id, msg) => hub.handle_message(id, msg).await?,
                ClientEvent::OnConnect(id, host, user, server_tx) => {
                    hub.handle_connect(id, host, user, server_tx)
                }
                ClientEvent::OnClose(id) => hub.handle_close(id).await?,
            }
        }
    }

    fn handle_connect(
        &mut self,
        id: Uuid,
        host: String,
        user: String,
        server_tx: Sender<ServerEvent>,
    ) {
        self.client_manager
            .handle_connect(id, host, user, server_tx)
    }

    async fn handle_close(&mut self, id: Uuid) -> io::Result<()> {
        self.client_manager
            .handle_close(
                &id,
                &mut self.subscription_manager,
                &mut self.notification_manager,
                &mut self.publisher_manager,
            )
            .await
    }

    async fn handle_message(&mut self, id: Uuid, msg: Message) -> io::Result<()> {
        log::debug!("Received message from {id}: \"{msg:?}\"");

        match msg {
            Message::AuthorizationRequest(_) => todo!(),
            Message::AuthorizationResponse(_) => todo!(),
            Message::ForwardedMulticastData(_) => todo!(),
            Message::ForwardedSubscriptionRequest(_) => todo!(),
            Message::ForwardedUnicastData(_) => todo!(),
            Message::MulticastData(msg) => {
                self.publisher_manager
                    .send_multicast_data(
                        &id,
                        msg.topic,
                        msg.content_type,
                        msg.data_packets,
                        &self.subscription_manager,
                        &self.client_manager,
                        &self.entitlement_manager,
                    )
                    .await
            }
            Message::NotificationRequest(msg) => {
                self.notification_manager
                    .handle_notification_request(
                        &id,
                        msg,
                        &self.client_manager,
                        &self.subscription_manager,
                    )
                    .await
            }
            Message::SubscriptionRequest(msg) => {
                self.subscription_manager
                    .handle_subscription_request(
                        &id,
                        msg,
                        &self.client_manager,
                        &self.notification_manager,
                    )
                    .await
            }
            Message::UnicastData(msg) => {
                self.publisher_manager
                    .send_unicast_data(
                        id,
                        msg.client_id,
                        msg.topic,
                        msg.content_type,
                        msg.data_packets,
                        &self.client_manager,
                        &self.entitlement_manager,
                    )
                    .await
            }
        }
    }
}
