use crate::client::{connect, send_clipboard};
use copypasta::{ClipboardContext, ClipboardProvider};
use tokio::time::{sleep, Duration};
use tonic::transport::Uri;
use tracing::{debug, info};

/// Scans the clipboard once per second. If it changes, we send it to the server.
pub async fn clipboard(server: Uri) {
    let mut clipboard = ClipboardContext::new().unwrap();
    let mut contents = clipboard.get_contents().unwrap();

    info!("Connecting to {}", server);
    let mut client = connect(&server).await;

    info!("Scanning clipboard");
    loop {
        sleep(Duration::from_secs(1)).await;
        let next = clipboard.get_contents().unwrap();
        if next != contents {
            contents = next;
            debug!("**{}**", contents);
            if !contents.is_empty() {
                send_clipboard(&mut client, contents.clone()).await;
            }
        }
    }
}
