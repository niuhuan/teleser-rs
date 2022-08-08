use crate::Result;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{new_message, Handler, InnerClient, Module};

#[new_message]
async fn proc_new_message(_: &InnerClient, message: &Message) -> Result<bool> {
    println!("PROC : A NEW MESSAGE : {}", message.text());
    Ok(false)
}

pub(crate) fn module() -> Module {
    Module {
        id: "proc_new_message".to_owned(),
        name: "proc_new_message".to_owned(),
        handlers: vec![Handler {
            id: "proc_new_message".to_owned(),
            process: proc_new_message {}.into(),
        }],
    }
}
