use crate::clipclop::clip_clop_server::ClipClopServer;
use clap::{App, Arg};
use tokio::sync::mpsc::{self, Sender};
use tonic::transport::{Certificate, ClientTlsConfig, Identity, Server, ServerTlsConfig, Uri};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod client;
mod clipclop;
mod scanner;
mod server;

async fn server(
    tls: ServerTlsConfig,
    tx: Sender<String>,
    port: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Listening on {}", port);
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let server = server::MyClipClop { tx };

    Server::builder()
        .tls_config(tls)?
        .add_service(ClipClopServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("clipclop")
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("only")
                .short("o")
                .long("only")
                .takes_value(true),
        )
        .get_matches();

    let port = matches
        .value_of("port")
        .unwrap_or("9998")
        .parse::<usize>()
        .expect("port must be a number");
    let server_addr = matches.value_of("server").expect("Need a server");
    let mut server_uri: Uri = server_addr.parse()?;
    if server_uri.scheme().is_none() {
        // Default the scheme to http.
        let mut parts = server_uri.into_parts();
        parts.scheme = Some("https".parse().expect("https should be valid"));
        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().expect("root should be valid"));
        }
        server_uri = Uri::from_parts(parts)?;
    }

    let only = matches.value_of("only");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load certs
    let cert = tokio::fs::read("cert/machine1-cert.pem").await?;
    let key = tokio::fs::read("cert/machine1-key.pem").await?;
    let server_identity = Identity::from_pem(cert, key);

    let server_root_ca_cert = tokio::fs::read("cert/ca-cert.pem").await?;
    let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);

    let client_ca_cert = tokio::fs::read("cert/ca-cert.pem").await?;
    let client_ca_cert = Certificate::from_pem(client_ca_cert);

    let client_cert = tokio::fs::read("cert/machine1-cert.pem").await?;
    let client_key = tokio::fs::read("cert/machine1-key.pem").await?;
    let client_identity = Identity::from_pem(client_cert, client_key);

    let clienttls = ClientTlsConfig::new()
        .domain_name("localhost")
        .ca_certificate(server_root_ca_cert)
        .identity(client_identity);

    let servertls = ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_cert);

    // Create an mpsc channel so the server can tell the scanner when it is changing the clipboard
    // to prevent us sending back a clipboard we have just received.
    let (tx, rx) = mpsc::channel(10);

    match only {
        Some("server") => server(servertls, tx, port).await?,
        Some("client") => scanner::clipboard(clienttls, rx, server_uri).await,
        _ => {
            async {
                let _ = tokio::join!(
                    server(servertls, tx, port),
                    scanner::clipboard(clienttls, rx, server_uri)
                );
            }
            .await
        }
    }

    Ok(())
}
