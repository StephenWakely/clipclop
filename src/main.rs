use crate::clipclop::clip_clop_server::ClipClopServer;
use clap::{App, Arg};
use tokio::sync::mpsc::{self, Sender};
use tonic::transport::{Certificate, ClientTlsConfig, Identity, Server, ServerTlsConfig, Uri};
use tracing::info;

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
                .help("The address, including the port of the other machine to sync to")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cacert")
                .long("cacert")
                .help("The cacert file that has signed both machines certs")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cert")
                .short("c")
                .long("cert")
                .help("The certificate for this machine")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .help("The private key for this machines certificate")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("The port to listen on")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("only")
                .short("o")
                .long("only")
                .help("If we only want the server or client part")
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
        // Default the scheme to https.
        let mut parts = server_uri.into_parts();
        parts.scheme = Some("https".parse().expect("https should be valid"));
        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().expect("root should be valid"));
        }
        server_uri = Uri::from_parts(parts)?;
    }

    let only = matches.value_of("only");

    env_logger::init();

    // Load certs
    let cert = tokio::fs::read(matches.value_of("cert").expect("Need a cert")).await?;
    let key = tokio::fs::read(matches.value_of("key").expect("Need a key")).await?;
    let cacert = tokio::fs::read(matches.value_of("cacert").expect("Need a cacert")).await?;
    let cacert = Certificate::from_pem(cacert);

    let identity = Identity::from_pem(cert, key);

    let clienttls = ClientTlsConfig::new()
        .domain_name(server_uri.host().ok_or("Server must specify the host")?)
        .ca_certificate(cacert.clone())
        .identity(identity.clone());

    let servertls = ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(cacert);

    // Create an mpsc channel so the server can tell the scanner when it is changing the clipboard
    // to prevent us sending back a clipboard we have just received.
    let (tx, rx) = mpsc::channel(10);

    match only {
        Some("server") => server(servertls, tx, port).await?,
        Some("client") => scanner::clipboard(clienttls, rx, server_uri).await,
        _ => {
            tokio::join!(
                server(servertls, tx, port),
                scanner::clipboard(clienttls, rx, server_uri)
            )
            .0?;
        }
    }

    Ok(())
}
