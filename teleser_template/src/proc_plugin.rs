use crate::Result;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{event, Handler, InnerClient};

#[event]
async fn proc_plugin(_: &InnerClient, message: &Message) -> Result<bool> {
    println!("PROC : A NEW MESSAGE : {}", message.text());
    Ok(false)
}

pub(crate) fn handler() -> Handler {
    proc_plugin {}.into()
}
