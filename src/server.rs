use crate::clipclop::clip_clop_server::ClipClop;
use crate::clipclop::{Clipboard, ClipboardReply};
use copypasta::{ClipboardContext, ClipboardProvider};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, Default)]
pub struct MyClipClop;

#[tonic::async_trait]
impl ClipClop for MyClipClop {
    async fn send_clipboard(
        &self,
        request: Request<Clipboard>,
    ) -> Result<Response<ClipboardReply>, Status> {
        info!("Received a clipboard from the client");

        let mut clipboard = ClipboardContext::new().unwrap();
        clipboard
            .set_contents(request.into_inner().contents)
            .unwrap();

        let reply = ClipboardReply {
            message: "groovy".to_string(),
        };
        Ok(Response::new(reply))
    }
}
