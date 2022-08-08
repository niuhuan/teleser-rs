use crate::Result;
use teleser::re_exports::grammers_client::types::MessageDeletion;
use teleser::{message_deleted, Handler, InnerClient};

#[message_deleted]
async fn proc_message_deleted(_: &InnerClient, message: &MessageDeletion) -> Result<bool> {
    println!("PROC : MESSAGE deleted : {:?}", message.messages());
    Ok(false)
}

pub(crate) fn handler() -> Handler {
    proc_message_deleted {}.into()
}
