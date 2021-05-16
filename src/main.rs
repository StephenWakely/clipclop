use crate::clipclop::clip_clop_server::ClipClopServer;
use clap::{App, Arg};
use tonic::transport::Server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod client;
mod clipclop;
mod scanner;
mod server;

async fn server(port: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Listening on {}", port);
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let server = server::MyClipClop::default();

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
    let only = matches.value_of("only");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match only {
        Some("server") => server(port).await?,
        Some("client") => scanner::clipboard(server_addr.to_string()).await,
        _ => {
            async {
                let _ = tokio::join!(server(port), scanner::clipboard(server_addr.to_string()));
            }
            .await
        }
    }

    Ok(())
}
