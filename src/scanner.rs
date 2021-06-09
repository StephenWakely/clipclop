use crate::client::{connect, send_clipboard};
use copypasta::{ClipboardContext, ClipboardProvider};
use tokio::{
    sync::mpsc::Receiver,
    time::{sleep, Duration},
};
use tonic::transport::{ClientTlsConfig, Uri};
use tracing::{debug, error, info};

/// Scans the clipboard once per second. If it changes, we send it to the server.
pub async fn clipboard(tls: ClientTlsConfig, mut rx: Receiver<String>, server: Uri) {
    let mut clipboard = ClipboardContext::new().unwrap();
    let mut contents = clipboard.get_contents().ok();

    info!("Connecting to {}", server);
    let mut client = connect(tls, &server).await;

    info!("Scanning clipboard");
    loop {
        tokio::select! {
                _ = async {
                    sleep(Duration::from_secs(1)).await;
                    match clipboard.get_contents() {
                        Ok(next) => {
                            let update = match contents {
                                Some(ref contents) if &next == contents => {
                                    false
                                }
                                _ => true

                            };

                            if update {
                                debug!("**{}**", next);
                                contents = Some(next);
                                match contents {
                                    Some(ref contents) if !contents.is_empty() => {
                                        send_clipboard(&mut client, contents.clone()).await;
                                    },
                                    _ => ()
                                }
                            }
                        }
                        Err(err) => error!("Error reading clipboard {:?}", err),
                    }
                } => {}

                next = rx.recv() => { contents = next }
        }
    }
}
