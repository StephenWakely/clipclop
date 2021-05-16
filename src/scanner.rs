use crate::client::client;
use copypasta::{ClipboardContext, ClipboardProvider};
use tokio::time::{sleep, Duration};
use tracing::info;

pub async fn clipboard(server: String) {
    let mut clipboard = ClipboardContext::new().unwrap();
    let mut contents = clipboard.get_contents().unwrap();
    info!("Scanning clipboard");
    loop {
        sleep(Duration::from_secs(6)).await;
        let next = clipboard.get_contents().unwrap();
        if next != contents {
            contents = next;
            println!("{}", contents);
            // tx.send(contents.clone()).await.unwrap();
            client(server.clone(), contents.clone()).await;
        }
    }
}
