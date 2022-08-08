use teleser::re_exports::async_trait::async_trait;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{Handler, InnerClient, NewMessageProcess, Process};

pub(crate) struct RawPlugin {}

#[async_trait]
impl NewMessageProcess for RawPlugin {
    async fn handle(&self, _: &InnerClient, event: &Message) -> crate::Result<bool> {
        println!("RAW : A NEW MESSAGE : {}", event.text());
        Ok(false)
    }
}

pub(crate) fn build() -> Handler {
    Handler {
        id: "raw".to_string(),
        name: "raw".to_string(),
        process: Process::NewMessageProcess(Box::new(RawPlugin {})),
    }
}
