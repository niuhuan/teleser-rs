use crate::Result;
use std::cmp::min;

use crate::handler::Module;
use anyhow::anyhow;
use async_trait::async_trait;
use grammers_client::{Config, InitParams, SignInError, Update};
use grammers_session::Session;
use grammers_tl_types as tl;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

pub struct Client {
    pub inner_client: Mutex<Option<grammers_client::Client>>,
    pub modules: Arc<Vec<Module>>,
    api_id: i32,
    api_hash: String,
    auth: Auth,
    session_store: Box<dyn SessionStore + Sync + Send>,
    init_params: Option<InitParams>,
}

enum MapResult<'a> {
    None,
    Process(&'a str, &'a str),
    Exception(&'a str, &'a str),
}

macro_rules! map_modules {
    ($hs:expr, $cp:expr $(,$event:expr, $process:path)* $(,)?) => {{
        let mut result = MapResult::None;
            for m in $hs {
                for h in &m.handlers {
                    match &h.process {
                    $(
                    $process(e) => match e.handle($cp, $event).await {
                        Ok(b) => {
                            if b {
                                result = MapResult::Process(&m.id, &h.id);
                            }
                        }
                        Err(err) => {
                            tracing::error!("error : {:?}", err);
                            result = MapResult::Exception(&m.id, &h.id);
                        }
                    },
                    )*
                    _ => (),
                }
                }
                if let MapResult::None = result {
                } else {
                    break;
                }
            }
            match result {
                MapResult::None => tracing::debug!("not process"),
                MapResult::Process(m, h) => tracing::debug!("process by : {m} : {h}"),
                MapResult::Exception(m, h) => tracing::debug!("process by : {m} : {h}"),
            }
        result
    }};
}

async fn hand(modules: Arc<Vec<Module>>, mut client: grammers_client::Client, update: Update) {
    let client_point = &mut client;
    let update_point = &update;
    match update_point {
        Update::NewMessage(message) => {
            tracing::debug!(
                "New Message : {} : {} : {}",
                message.chat().id(),
                message.id(),
                message.text()
            );
            let _ = map_modules!(
                modules.deref(),
                client_point,
                message,
                crate::handler::Process::NewMessageProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::MessageEdited(message) => {
            tracing::debug!("Message Edited : {}", message.id());
            let _ = map_modules!(
                modules.deref(),
                client_point,
                message,
                crate::handler::Process::MessageEditedProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::MessageDeleted(deletion) => {
            tracing::debug!("Message Deleted : {:?}", deletion.messages());
            let _ = map_modules!(
                modules.deref(),
                client_point,
                deletion,
                crate::handler::Process::MessageDeletedProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::CallbackQuery(callback_query) => {
            tracing::debug!("Callback Query : {:?}", callback_query.chat().id());
            let _ = map_modules!(
                modules.deref(),
                client_point,
                callback_query,
                crate::handler::Process::CallbackQueryProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::InlineQuery(inline_query) => {
            tracing::debug!("Inline Query : {:?}", inline_query.text());
            let _ = map_modules!(
                modules.deref(),
                client_point,
                inline_query,
                crate::handler::Process::InlineQueryProcess,
                update_point,
                crate::handler::Process::UpdateProcess,
            );
        }
        Update::Raw(update) => {
            tracing::debug!("Raw : {:?}", update);
            let _ = map_modules!(
                modules.deref(),
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
        Ok(
            if let Some(data) = self.session_store.on_load_session().await? {
                Session::load(&data)?
            } else {
                Session::new()
            },
        )
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
            params: match self.init_params.clone() {
                None => InitParams::default(),
                Some(params) => params,
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
                        auth.input_phone().await?.as_str(),
                        client.api_id.clone(),
                        client.api_hash.as_str(),
                    )
                    .await?;
                match inner_client
                    .sign_in(&token, auth.input_code().await?.as_str())
                    .await
                {
                    Err(SignInError::PasswordRequired(password_token)) => {
                        inner_client
                            .check_password(password_token, auth.input_password().await?.as_str())
                            .await?
                    }
                    Ok(usr) => usr,
                    Err(err) => return Err(anyhow!(err)),
                }
            }
            Auth::AuthWithBotToken(auth) => {
                inner_client
                    .bot_sign_in(
                        auth.input_bot_token().await?.as_str(),
                        client.api_id.clone(),
                        client.api_hash.as_str(),
                    )
                    .await?
            }
        };
        tracing::info!("login with id : {}", usr.id());
        client
            .session_store
            .on_save_session(inner_client.session().save())
            .await?;
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
                    tracing::warn!("reconnected");
                    inner_client = client_new;
                    match inner_client.is_authorized().await {
                        Ok(auth) => {
                            if !auth {
                                tracing::error!("logged out, exit");
                                break;
                            }
                        }
                        Err(e) => {
                            error_counter += 1;
                            let sleep_sec = 2_u64.pow(min(10, error_counter));
                            tracing::error!("reconnect auth error : sleep {sleep_sec} sec : {e}");
                            sleep(Duration::from_secs(sleep_sec)).await;
                        }
                    }
                }
                Err(e) => {
                    error_counter += 1;
                    let sleep_sec = 2_u64.pow(min(10, error_counter));
                    tracing::error!("reconnect error : sleep {sleep_sec} sec : {e}");
                    sleep(Duration::from_secs(sleep_sec)).await;
                }
            }
        }
        tokio::select! {
            result = inner_client.next_update() => match result {
                Ok(update)=> {
                    error_counter = 0;
                    if let Some(update) = update {
                        task::spawn(hand(client.modules.clone(),inner_client.clone(), update));
                    }
                }
                Err(e)=>{
                    error_counter+=1;
                    let sleep_sec = 2_u64.pow(min(10,error_counter));
                    tracing::error!("next_update error : sleep {sleep_sec} sec : {e}");
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
    session_store: Option<Box<dyn SessionStore + Sync + Send>>,
    modules: Option<Arc<Vec<Module>>>,
    init_params: Option<InitParams>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            api_id: None,
            api_hash: None,
            auth: None,
            session_store: None,
            modules: None,
            init_params: None,
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

    pub fn set_session_store(&mut self, session_store: Box<dyn SessionStore + Sync + Send>) {
        self.session_store = Some(session_store)
    }

    pub fn with_session_store(
        mut self,
        session_store: Box<dyn SessionStore + Sync + Send>,
    ) -> Self {
        self.set_session_store(session_store);
        self
    }

    pub fn set_modules<S: Into<Arc<Vec<Module>>>>(&mut self, s: S) {
        self.modules = Some(s.into())
    }

    pub fn with_modules<S: Into<Arc<Vec<Module>>>>(mut self, s: S) -> Self {
        self.set_modules(s);
        self
    }

    pub fn set_init_params(&mut self, s: Option<InitParams>) {
        self.init_params = s
    }

    pub fn with_init_params(mut self, s: Option<InitParams>) -> Self {
        self.set_init_params(s);
        self
    }

    pub fn build(self) -> Result<Client> {
        return Ok(Client {
            modules: self.modules.expect("must set modules"),
            inner_client: Mutex::new(None),
            api_id: self.api_id.expect("must set api_id"),
            api_hash: self.api_hash.expect("must set api_hash"),
            auth: self.auth.expect("must set auth"),
            session_store: self.session_store.expect("must set session_store"),
            init_params: self.init_params,
        });
    }
}

pub enum Auth {
    AuthWithBotToken(Box<dyn AuthWithBotToken + Send + Sync>),
    AuthWithPhoneAndCode(Box<dyn AuthWithPhoneAndCode + Send + Sync>),
}

#[async_trait]
pub trait AuthWithBotToken {
    async fn input_bot_token(&self) -> Result<String>;
}

#[async_trait]
pub trait AuthWithPhoneAndCode {
    async fn input_phone(&self) -> Result<String>;
    async fn input_code(&self) -> Result<String>;
    async fn input_password(&self) -> Result<String>;
}

#[async_trait]
pub trait SessionStore {
    async fn on_save_session(&self, data: Vec<u8>) -> Result<()>;
    async fn on_load_session(&self) -> Result<Option<Vec<u8>>>;
}

pub struct StaticBotToken {
    pub token: String,
}

#[async_trait]
impl AuthWithBotToken for StaticBotToken {
    async fn input_bot_token(&self) -> Result<String> {
        return Ok(self.token.clone());
    }
}

pub struct FileSessionStore {
    pub path: String,
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn on_save_session(&self, data: Vec<u8>) -> Result<()> {
        tokio::fs::write(self.path.as_str(), data).await?;
        Ok(())
    }
    async fn on_load_session(&self) -> Result<Option<Vec<u8>>> {
        let path = Path::new(self.path.as_str());
        if path.exists() {
            Ok(Some(tokio::fs::read(path).await?))
        } else {
            Ok(None)
        }
    }
}
