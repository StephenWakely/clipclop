use crate::clipclop::clip_clop_server::ClipClopServer;
use clap::{App, Arg};
use tokio::sync::mpsc::{self, Sender};
use tonic::transport::{Server, Uri};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod client;
mod clipclop;
mod scanner;
mod server;

async fn server(tx: Sender<String>, port: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Listening on {}", port);
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let server = server::MyClipClop { tx };

    Server::builder()
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
        parts.scheme = Some("http".parse().expect("http should be valid"));
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

    // Create an mpsc channel so the server can tell the scanner when it is changing the clipboard
    // to prevent us sending back a clipboard we have just received.
    let (tx, rx) = mpsc::channel(10);

    match only {
        Some("server") => server(tx, port).await?,
        Some("client") => scanner::clipboard(rx, server_uri).await,
        _ => {
            async {
                let _ = tokio::join!(server(tx, port), scanner::clipboard(rx, server_uri));
            }
            .await
        }
    }

    Ok(())
}
