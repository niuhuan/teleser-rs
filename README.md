Teleser
=======

A telegram's **MTProto** bot framework based [grammers](https://github.com/Lonami/grammers), supported login with phone number and bot token

## Make a bot

```toml
teleser = { git = "https://github.com/niuhuan/teleser-rs.git" }
```

And look up [bot template source code](https://github.com/niuhuan/teleser-rs/tree/master/teleser_template/src)

### Main

- Set logger
- Using `tokio` async runtime run async main

```rust
fn main() -> Result<()> {
    init_tracing_subscriber();
    return runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(30)
        .worker_threads(50)
        .build()
        .unwrap()
        .block_on(async_main());
}
```

### Logger

`tracing_subscriber` to set log level, your need modify `"teleser_template"` to your self crate name.

```rust
fn init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("teleser", Level::DEBUG)
                .with_target("teleser_template", Level::DEBUG),
        )
        .init();
}
```

### Async_main

```rust

async fn async_main() -> Result<()> {
    // Using `ClientBuilder` build a bot client
    let client = Arc::new(
        teleser::ClientBuilder::new()
            // read api id from env (on build)
            .with_api_id(env!("API_ID").parse()?)
            // read api hash from env (on build)
            .with_api_hash(env!("API_HASH").to_string())
            // on need input phone
            .with_input_phone(Box::pin(|| {
                Box::pin(async { input("Input your phone number ( like +112345678 )") })
            }))
            // on need input code
            .with_input_code(Box::pin(|| {
                Box::pin(async { input("Input your device or sms code ( like 12345 )") })
            }))
            // save session to file teleser.session if login
            .with_on_save_session(Box::pin(|data| {
                Box::pin(async move {
                    tokio::fs::write("teleser.session", data).await?;
                    Ok(())
                })
            }))
            // load session file on startup
            .with_on_load_session(Box::pin(|| {
                Box::pin(async move {
                    let path = Path::new("teleser.session");
                    if path.exists() {
                        Ok(Some(tokio::fs::read(path).await?))
                    } else {
                        Ok(None)
                    }
                })
            }))
            // handlers
            .with_handlers(vec![raw_plugin::build(), proc_plugin::handler()])
            // connect to server via proxy url, like socks5://127.0.0.1:1080 (runtime)
            .with_proxy(match std::env::var("TELESER_PROXY") {
                Ok(url) => Some(url),
                Err(_) => None,
            })
            .build()?,
    );
    //////////////////////////////////////
    // can start some timer task like this
    let copy_client = client.clone();
    tokio::spawn(async move {
        let lock = copy_client.inner_client.lock().await;
        let ic = lock.clone();
        drop(lock);
        if let Some(_ic) = ic {
            // _ic.send_message();
        }
    });
    //////////////////////////////////////
    // run client
    teleser::run_client_and_reconnect(client).await?;
    /////////////////////////////////////
    Ok(())
}
```

### Input 

look up source code, only read line from console

### Handler

This example has handler grammers `Update::OnNewMessage`. You can use these handlers:

`new_message` / `message_edited` / `message_deleted` / `callback_query` / `inline_query` / `raw`

But I can't got message_edited event on grammers, and got the raw.

```rust
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
```

### Manually write a handler

```rust
use teleser::re_exports::async_trait::async_trait;
use teleser::re_exports::grammers_client::types::Message;
use teleser::{Handler, InnerClient, NewMessageProcess, Process};

pub(crate) struct RawPlugin {}

#[async_trait]
impl NewMessageProcess for RawPlugin {
    async fn handle(&self, _: &InnerClient, event: &Message) -> crate::Result<bool> {
        println!("RAW : A NEW MESSAGE : {}", event.text());
        Ok(false)
    }
}

pub(crate) fn build() -> Handler {
    Handler {
        id: "raw".to_string(),
        process: Process::NewMessageProcess(Box::new(RawPlugin {})),
    }
}
```

