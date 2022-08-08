use crate::Result;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{new_message, Handler, InnerClient};

#[new_message]
async fn proc_new_message(_: &InnerClient, message: &Message) -> Result<bool> {
    println!("PROC : A NEW MESSAGE : {}", message.text());
    Ok(false)
}

pub(crate) fn handler() -> Handler {
    proc_new_message {}.into()
}
