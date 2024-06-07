use std::io;

use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod events;
use events::ClientEvent;

mod hub;
use hub::Hub;

mod interactor;
use interactor::Interactor;

mod clients;
mod notifications;
mod publishing;
mod subscriptions;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);
    let mut hub = Hub::new();

    // Create a hub that listens to clients
    tokio::spawn(async move {
        hub.run(server_rx).await.unwrap();
    });

    loop {
        let (socket, addr) = listener.accept().await?;

        let client_tx = client_tx.clone();
        let interactor = Interactor::new();

        tokio::spawn(async move {
            match interactor.run(socket, addr, client_tx).await {
                Ok(()) => println!("Client exited normally"),
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        println!("Client closed connection")
                    } else {
                        println!("Client exited with {e}")
                    }
                }
            }
        });
    }
}
