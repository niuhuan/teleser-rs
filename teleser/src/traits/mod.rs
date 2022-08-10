use async_trait::async_trait;
use grammers_client::types::{Chat, Message};

pub trait ChatExt {
    fn is_user(&self) -> bool;

    fn is_group(&self) -> bool;

    fn is_channel(&self) -> bool;
}

impl ChatExt for Chat {
    fn is_user(&self) -> bool {
        match self {
            Chat::User(_) => true,
            _ => false,
        }
    }

    fn is_group(&self) -> bool {
        match self {
            Chat::Group(_) => true,
            _ => false,
        }
    }

    fn is_channel(&self) -> bool {
        match self {
            Chat::Channel(_) => true,
            _ => false,
        }
    }
}

#[async_trait]
pub trait MessageExt {
    fn has_sender(&self) -> bool;
}

impl ChatExt for Message {
    fn is_user(&self) -> bool {
        self.chat().is_user()
    }

    fn is_group(&self) -> bool {
        self.chat().is_group()
    }

    fn is_channel(&self) -> bool {
        self.chat().is_channel()
    }
}

#[async_trait]
impl MessageExt for Message {
    fn has_sender(&self) -> bool {
        self.sender().is_some()
    }
}
