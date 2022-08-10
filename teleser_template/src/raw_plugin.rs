use teleser::re_exports::async_trait::async_trait;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{Handler, InnerClient, Module, NewMessageProcess, Process};

pub(crate) struct RawPlugin {}

#[async_trait]
impl NewMessageProcess for RawPlugin {
    async fn handle(&self, _: &mut InnerClient, event: &Message) -> crate::Result<bool> {
        println!("RAW : A NEW MESSAGE : {}", event.text());
        Ok(false)
    }
}

pub(crate) fn module() -> Module {
    Module {
        id: "RawPlugin".to_owned(),
        name: "RawPlugin".to_owned(),
        handlers: vec![Handler {
            id: "RawPlugin".to_owned(),
            process: Process::NewMessageProcess(Box::new(RawPlugin {})),
        }],
    }
}
