use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use pki_types::{CertificateDer, PrivateKeyDer};

use rustls_pemfile::{certs, private_key};

use tokio_rustls::{rustls, TlsAcceptor};

pub fn create_acceptor(certfile: &PathBuf, keyfile: &PathBuf) -> io::Result<TlsAcceptor> {
    // Ensure we have all the arguments.
    let certs = load_certs(certfile)?;
    let key = load_key(keyfile)?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    Ok(acceptor)
}

fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        ))?)
}
