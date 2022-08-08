use crate::Result;
use std::cmp::min;

use crate::handler::Handler;
use grammers_client::{Config, InitParams, Update};
use grammers_session::Session;
use grammers_tl_types as tl;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

pub struct Client {
    pub inner_client: Mutex<Option<grammers_client::Client>>,
    pub handlers: Arc<Vec<Handler>>,
    api_id: i32,
    api_hash: String,
    auth: Auth,
    on_save_session: Pin<Box<fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>>>,
    on_load_session:
        Pin<Box<fn() -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>>> + Send>>>>,
    proxy: Option<String>,
}

enum MapResult<'a> {
    None,
    Process(&'a str),
    Exception(&'a str),
}

macro_rules! map_handlers {
    ($hs:expr, $cp:expr $(,$event:expr, $process:path)* $(,)?) => {{
        let mut result = MapResult::None;
            for h in $hs {
                match &h.process {
                    $(
                    $process(e) => match e.handle($cp, $event).await {
                        Ok(b) => {
                            if b {
                                result = MapResult::Process(&h.id);
                            }
                        }
                        Err(err) => {
                            tracing::error!("error : {:?}", err);
                            result = MapResult::Exception(&h.id);
                        }
                    },
                    )*
                    _ => (),
                }
                if let MapResult::None = result {
                } else {
                    break;
                }
            }
        result
    }};
}

async fn hand(handlers: Arc<Vec<Handler>>, client: grammers_client::Client, update: Update) {
    let client_point = &client;
    let update_point = &update;
    match update_point {
        Update::NewMessage(message) => {
            let _ = map_handlers!(
                handlers.deref(),
                client_point,
                message,
                crate::handler::Process::NewMessageProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::MessageEdited(message) => {
            let _ = map_handlers!(
                handlers.deref(),
                client_point,
                message,
                crate::handler::Process::MessageEditedProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::MessageDeleted(deletion) => {
            let _ = map_handlers!(
                handlers.deref(),
                client_point,
                deletion,
                crate::handler::Process::MessageDeletedProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::CallbackQuery(callback_query) => {
            let _ = map_handlers!(
                handlers.deref(),
                client_point,
                callback_query,
                crate::handler::Process::CallbackQueryProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::InlineQuery(inline_query) => {
            let _ = map_handlers!(
                handlers.deref(),
                client_point,
                inline_query,
                crate::handler::Process::InlineQueryProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::Raw(update) => {
            let _ = map_handlers!(
                handlers.deref(),
                client_point,
                update,
                crate::handler::Process::RawProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        _ => {}
    }
}

impl Client {
    async fn load_session(&self) -> Result<Session> {
        Ok(if let Some(data) = (self.on_load_session)().await? {
            Session::load(&data)?
        } else {
            Session::new()
        })
    }

    async fn set_client(&self, inner_client: Option<grammers_client::Client>) {
        let mut lock = self.inner_client.lock().await;
        *lock = inner_client;
        drop(lock);
    }

    async fn connect(&self) -> Result<grammers_client::Client> {
        let connect = grammers_client::Client::connect(Config {
            session: self.load_session().await?,
            api_id: self.api_id.clone(), // not actually logging in, but has to look real
            api_hash: self.api_hash.clone(),
            params: {
                let mut params = InitParams::default();
                params.proxy_url = self.proxy.clone();
                params
            },
        })
        .await;
        let client = connect?;
        self.set_client(Some(client.clone())).await;
        Ok(client)
    }
}

pub async fn run_client_and_reconnect<S: Into<Arc<Client>>>(client: S) -> Result<()> {
    let client = client.into();
    let mut inner_client = client.connect().await?;
    tracing::info!("Connected! (first)");
    tracing::info!("Sending ping...");
    tracing::info!(
        "{:?}",
        inner_client
            .invoke(&tl::functions::Ping { ping_id: 0 })
            .await?
    );
    if !inner_client.is_authorized().await? {
        let usr = match &client.auth {
            Auth::AuthWithPhoneAndCode(auth) => {
                let token = inner_client
                    .request_login_code(
                        (auth.input_phone)().await?.as_str(),
                        client.api_id.clone(),
                        client.api_hash.as_str(),
                    )
                    .await?;
                inner_client
                    .sign_in(&token, (auth.input_code)().await?.as_str())
                    .await?
            }
            Auth::AuthWithBotToken(auth) => {
                inner_client
                    .bot_sign_in(
                        (auth.input_bot_token)().await?.as_str(),
                        client.api_id.clone(),
                        client.api_hash.as_str(),
                    )
                    .await?
            }
        };
        tracing::info!("login with id : {}", usr.id());
        (client.on_save_session)(inner_client.session().save()).await?;
    } else {
        let usr = inner_client.get_me().await?;
        tracing::info!("session with id : {}", usr.id());
    }

    let mut error_counter = 0;

    tracing::info!("Waiting for messages...");

    // loop
    loop {
        // reconnect
        if error_counter > 0 {
            match client.connect().await {
                Ok(client_new) => {
                    inner_client = client_new;
                    match inner_client.is_authorized().await {
                        Ok(auth) => {
                            if !auth {
                                tracing::warn!("logged out");
                                break;
                            }
                        }
                        Err(e) => {
                            error_counter += 1;
                            let sleep_sec = 2_u64.pow(min(10, error_counter));
                            println!("reconnect auth error : sleep {sleep_sec} sec : {e}");
                            sleep(Duration::from_secs(sleep_sec)).await;
                        }
                    }
                }
                Err(e) => {
                    error_counter += 1;
                    let sleep_sec = 2_u64.pow(min(10, error_counter));
                    println!("reconnect error : sleep {sleep_sec} sec : {e}");
                    sleep(Duration::from_secs(sleep_sec)).await;
                }
            }
        }
        tokio::select! {
            result = inner_client.next_update() => match result {
                Ok(update)=> {
                    error_counter = 0;
                    if let Some(update) = update {
                        task::spawn(hand(client.handlers.clone(),inner_client.clone(), update));
                    }
                }
                Err(e)=>{
                    error_counter+=1;
                    let sleep_sec = 2_u64.pow(min(10,error_counter));
                    println!("next_update error : sleep {sleep_sec} sec : {e}");
                    sleep(Duration::from_secs(sleep_sec)).await;
                }
            },
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    Ok(())
}

pub struct ClientBuilder {
    api_id: Option<i32>,
    api_hash: Option<String>,
    auth: Option<Auth>,
    on_save_session:
        Option<Pin<Box<fn(Vec<u8>) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>>>>,
    on_load_session: Option<
        Pin<Box<fn() -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Vec<u8>>>> + Send>>>>,
    >,
    handlers: Option<Arc<Vec<Handler>>>,
    proxy: Option<String>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            api_id: None,
            api_hash: None,
            auth: None,
            on_save_session: None,
            on_load_session: None,
            handlers: None,
            proxy: None,
        }
    }

    pub fn set_api_id(&mut self, api_id: i32) {
        self.api_id = Some(api_id)
    }

    pub fn with_api_id(mut self, api_id: i32) -> Self {
        self.set_api_id(api_id);
        self
    }

    pub fn set_api_hash<S: Into<String>>(&mut self, api_hash: S) {
        self.api_hash = Some(api_hash.into())
    }

    pub fn with_api_hash<S: Into<String>>(mut self, api_hash: S) -> Self {
        self.set_api_hash(api_hash);
        self
    }

    pub fn set_auth(&mut self, auth: Auth) {
        self.auth = Some(auth)
    }

    pub fn with_auth(mut self, auth: Auth) -> Self {
        self.set_auth(auth);
        self
    }

    pub fn set_on_save_session(
        &mut self,
        on_save_session: Pin<
            Box<fn(Vec<u8>) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>>,
        >,
    ) {
        self.on_save_session = Some(on_save_session)
    }

    pub fn with_on_save_session(
        mut self,
        on_save_session: Pin<
            Box<fn(Vec<u8>) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>>,
        >,
    ) -> Self {
        self.set_on_save_session(on_save_session);
        self
    }

    pub fn set_on_load_session(
        &mut self,
        on_load_session: Pin<
            Box<fn() -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Vec<u8>>>> + Send>>>,
        >,
    ) {
        self.on_load_session = Some(on_load_session)
    }

    pub fn with_on_load_session(
        mut self,
        on_load_session: Pin<
            Box<fn() -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Vec<u8>>>> + Send>>>,
        >,
    ) -> Self {
        self.set_on_load_session(on_load_session);
        self
    }

    pub fn set_handlers<S: Into<Arc<Vec<Handler>>>>(&mut self, s: S) {
        self.handlers = Some(s.into())
    }

    pub fn with_handlers<S: Into<Arc<Vec<Handler>>>>(mut self, s: S) -> Self {
        self.set_handlers(s);
        self
    }

    pub fn set_proxy(&mut self, s: Option<String>) {
        self.proxy = s
    }

    pub fn with_proxy(mut self, s: Option<String>) -> Self {
        self.set_proxy(s);
        self
    }

    pub fn build(self) -> Result<Client> {
        return Ok(Client {
            handlers: self.handlers.expect("must set handlers"),
            inner_client: Mutex::new(None),
            api_id: self.api_id.expect("must set api_id"),
            api_hash: self.api_hash.expect("must set api_hash"),
            auth: self.auth.expect("must set auth"),
            on_save_session: self.on_save_session.expect("must set on_save_session"),
            on_load_session: self.on_load_session.expect("must set on_load_session"),
            proxy: self.proxy,
        });
    }
}

pub enum Auth {
    AuthWithBotToken(AuthWithBotToken),
    AuthWithPhoneAndCode(AuthWithPhoneAndCode),
}

pub struct AuthWithBotToken {
    pub input_bot_token: Pin<Box<fn() -> Pin<Box<dyn Future<Output = Result<String>> + Send>>>>,
}

pub struct AuthWithPhoneAndCode {
    pub input_phone: Pin<Box<fn() -> Pin<Box<dyn Future<Output = Result<String>> + Send>>>>,
    pub input_code: Pin<Box<fn() -> Pin<Box<dyn Future<Output = Result<String>> + Send>>>>,
}
