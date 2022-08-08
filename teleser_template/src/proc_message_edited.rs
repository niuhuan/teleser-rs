use teleser::re_exports::anyhow::Result;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{message_edited, Handler, InnerClient, Module};

#[message_edited]
async fn proc_message_edited(_: &InnerClient, message: &Message) -> Result<bool> {
    println!("PROC : MESSAGE EDITED : {}", message.text());
    Ok(false)
}

pub(crate) fn module() -> Module {
    Module {
        id: "proc_message_edited".to_owned(),
        name: "proc_message_edited".to_owned(),
        handlers: vec![Handler {
            id: "proc_message_edited".to_owned(),
            process: proc_message_edited {}.into(),
        }],
    }
}
