use crate::Result;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{message_edited, Handler, InnerClient};

#[message_edited]
async fn proc_message_edited(_: &InnerClient, message: &Message) -> Result<bool> {
    println!("PROC : MESSAGE EDITED : {}", message.text());
    Ok(false)
}

pub(crate) fn handler() -> Handler {
    proc_message_edited {}.into()
}
