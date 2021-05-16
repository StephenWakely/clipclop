use crate::clipclop::{clip_clop_client::ClipClopClient, Clipboard};
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use tracing::{error, info};

async fn connect(server: String) -> ClipClopClient<Channel> {
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

pub async fn client(server: String, contents: String) {
    info!("Connecting to {}", server);

    let mut client = connect(server.clone()).await;
    let request = tonic::Request::new(Clipboard { contents });

    let response = client.send_clipboard(request).await;
    info!("Response {:?}", response);
}
