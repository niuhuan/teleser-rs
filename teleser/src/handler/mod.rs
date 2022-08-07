use async_trait::async_trait;
use grammers_client::types::{CallbackQuery, InlineQuery, Message, MessageDeletion};
use grammers_client::Update;
use grammers_tl_types as tl;

macro_rules! process_trait {
    ($name:ident, $event:path) => {
        #[async_trait]
        pub trait $name: Sync + Send {
            async fn handle(
                &self,
                client: &grammers_client::Client,
                event: &$event,
            ) -> anyhow::Result<bool>;
        }
    };
}

process_trait!(UpdateProcess, Update);
process_trait!(NewMessageProcess, Message);
process_trait!(MessageEditedProcess, Message);
process_trait!(MessageDeletedProcess, MessageDeletion);
process_trait!(CallbackQueryProcess, CallbackQuery);
process_trait!(InlineQueryProcess, InlineQuery);
process_trait!(RawProcess, tl::enums::Update);

pub enum Process {
    UpdateProcess(Box<dyn UpdateProcess>),
    NewMessageProcess(Box<dyn NewMessageProcess>),
    MessageEditedProcess(Box<dyn MessageEditedProcess>),
    MessageDeletedProcess(Box<dyn MessageDeletedProcess>),
    CallbackQueryProcess(Box<dyn CallbackQueryProcess>),
    InlineQueryProcess(Box<dyn InlineQueryProcess>),
    RawProcess(Box<dyn RawProcess>),
}

pub struct Handler {
    pub id: String,
    pub process: Process,
}
