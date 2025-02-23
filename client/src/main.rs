use std::error::Error;
use std::net::ToSocketAddrs;

use protocol::communicate;
use tls::create_tls_stream;

use options::Options;
use tokio::net::TcpStream;

mod authentication;
mod client;
mod options;
mod protocol;
mod tls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();

    log::info!("Starting client");

    let options = Options::load();

    let endpoint = format!("{}:{}", options.host.as_str(), options.port);

    let addr = endpoint
        .to_socket_addrs()?
        .next()
        .ok_or(format!("failed to resolve {}", options.host.as_str()))?;

    let socket = TcpStream::connect(&addr).await?;
    match options.tls {
        true => {
            let stream = create_tls_stream(options.host.as_str(), &options.cafile, socket).await?;
            communicate(
                stream,
                &options.authentication_mode,
                &options.username,
                &options.password,
            )
            .await;
        }
        false => {
            communicate(
                socket,
                &options.authentication_mode,
                &options.username,
                &options.password,
            )
            .await;
        }
    }

    Ok(())
}
