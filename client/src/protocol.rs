use std::collections::HashSet;
use std::io::{self, Error, ErrorKind};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, WriteHalf};

use tokio::io::{split, AsyncBufReadExt, BufReader};

use common::messages::{
    DataPacket, Message, MulticastData, NotificationRequest, SubscriptionRequest,
};

pub async fn communicate<S>(
    stream: S,
    mode: &String,
    username: &Option<String>,
    password: &Option<String>,
) where
    S: AsyncRead + AsyncWrite,
{
    println!("connected");

    let (skt_read_half, mut skt_write_half) = split(stream);
    let mut skt_reader = BufReader::new(skt_read_half);

    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    authenticate(&mut skt_write_half, mode, username, password)
        .await
        .unwrap();

    loop {
        let mut request_line = String::new();

        println!("Enter request:");
        println!("\tpublish <topic> <entitlements> <message>");
        println!("\tsubscribe <topic>");
        println!("\tnotify <pattern>");

        tokio::select! {
            // request
            result = stdin_reader.read_line(&mut request_line) => {
                result.unwrap();
                match parse_message(request_line.as_str()) {
                    Ok(message) => {
                        message.write(&mut skt_write_half).await.unwrap();
                    },
                    Err(message) => {
                        println!("{message}");
                    }
                }
            }
            // response
            result = Message::read(&mut skt_reader) => {
                let message = result.unwrap();
                println!("Received message {message:?}");
            }
        }
    }
}

async fn authenticate<S>(
    skt_write_half: &mut WriteHalf<S>,
    mode: &String,
    username: &Option<String>,
    password: &Option<String>,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite,
{
    // Mode
    skt_write_half.write(mode.as_bytes()).await?;
    skt_write_half.write("\n".as_bytes()).await?;

    if mode == "none" {
        log::info!("Authenticate with {}", mode.as_str());
    } else if mode == "htpasswd" {
        log::info!("Authenticate with {}", mode.as_str());
        let Some(username) = username else {
            return Err(Error::new(ErrorKind::Other, "missing username"));
        };
        let Some(password) = password else {
            return Err(Error::new(ErrorKind::Other, "missing password"));
        };
        // User
        skt_write_half.write(username.as_bytes()).await?;
        skt_write_half.write("\n".as_bytes()).await?;

        // Password
        skt_write_half.write(password.as_bytes()).await?;
        skt_write_half.write("\n".as_bytes()).await?;
    } else {
        log::error!("Invalid mode {}", mode.as_str());
        return Err(Error::new(ErrorKind::Other, "invalid mode"));
    }

    skt_write_half.flush().await?;

    Ok(())
}

fn parse_message(line: &str) -> Result<Message, &'static str> {
    let parts: Vec<&str> = line.trim().split(' ').collect();
    match parts[0] {
        "publish" => {
            if parts.len() < 4 || parts.len() % 2 == 1 {
                Err("usage: publish <topic> (<entitlements> <message>)+")
            } else {
                let topic = parts[1];
                let mut i = 2;
                let mut data_packets: Vec<DataPacket> = Vec::new();
                while i < parts.len() {
                    let entitlements: HashSet<i32> = parts[i]
                        .split(',')
                        .map(|x| x.parse().expect("should be an integer"))
                        .collect();
                    i += 1;

                    let message = parts[i];
                    data_packets.push(DataPacket::new(entitlements, Vec::from(message.as_bytes())));
                    i += 1;
                }
                let message = MulticastData {
                    topic: topic.to_string(),
                    content_type: String::from("text/plain"),
                    data_packets,
                };
                Ok(Message::MulticastData(message))
            }
        }
        "subscribe" => {
            if parts.len() != 2 {
                Err("usage: subscribe <topic>")
            } else {
                let message = create_subscription_message(parts[1]);
                Ok(Message::SubscriptionRequest(message))
            }
        }
        "notify" => {
            if parts.len() != 2 {
                Err("usage: subscribe <topic>")
            } else {
                let message = create_notification_message(parts[1]);
                Ok(Message::NotificationRequest(message))
            }
        }
        _ => Err("usage: publish/subscribe/notify"),
    }
}

fn create_subscription_message(topic: &str) -> SubscriptionRequest {
    SubscriptionRequest {
        topic: topic.to_string(),
        is_add: true,
    }
}

fn create_notification_message(pattern: &str) -> NotificationRequest {
    NotificationRequest {
        pattern: pattern.to_string(),
        is_add: true,
    }
}
