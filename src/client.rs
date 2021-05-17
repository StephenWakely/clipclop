use crate::clipclop::{clip_clop_client::ClipClopClient, Clipboard};
use tokio::time::{sleep, Duration};
use tonic::transport::{Channel, Uri};
use tracing::{error, info};

/// Sets up a connection to the other server.
pub async fn connect(server: &Uri) -> ClipClopClient<Channel> {
    loop {
        match ClipClopClient::connect(server.clone()).await {
            Ok(connection) => return connection,
            Err(err) => {
                error!("Error connection {}", err);
                sleep(Duration::from_secs(6)).await;
            }
        }
    }
}

/// Sends the clipboard contents to the given connection.
pub async fn send_clipboard(connection: &mut ClipClopClient<Channel>, contents: String) {
    let request = tonic::Request::new(Clipboard { contents });
    let response = connection.send_clipboard(request).await;
    info!("Response {:?}", response);
}
