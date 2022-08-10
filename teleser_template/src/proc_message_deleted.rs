use crate::Result;
use teleser::re_exports::grammers_client::types::MessageDeletion;
use teleser::{message_deleted, Handler, InnerClient, Module};

#[message_deleted]
async fn proc_message_deleted(_: &mut InnerClient, message: &MessageDeletion) -> Result<bool> {
    println!("PROC : MESSAGE deleted : {:?}", message.messages());
    Ok(false)
}

pub(crate) fn module() -> Module {
    Module {
        id: "proc_message_deleted".to_owned(),
        name: "proc_message_deleted".to_owned(),
        handlers: vec![Handler {
            id: "proc_message_deleted".to_owned(),
            process: proc_message_deleted {}.into(),
        }],
    }
}
