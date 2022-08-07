mod proc_plugin;
mod raw_plugin;

use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::sync::Arc;
use teleser::re_exports::tokio;
use teleser::re_exports::tokio::runtime;
use teleser::re_exports::tracing::Level;
use teleser::Result;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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

async fn async_main() -> Result<()> {
    let client = Arc::new(
        teleser::ClientBuilder::new()
            .with_api_id(env!("API_ID").parse()?)
            .with_api_hash(env!("API_HASH").to_string())
            .with_input_phone(Box::pin(|| {
                Box::pin(async { input("Input your phone number ( like +112345678 )") })
            }))
            .with_input_code(Box::pin(|| {
                Box::pin(async { input("Input your device or sms code ( like 12345 )") })
            }))
            .with_on_save_session(Box::pin(|data| {
                Box::pin(async move {
                    tokio::fs::write("teleser.session", data).await?;
                    Ok(())
                })
            }))
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
            .with_handlers(vec![raw_plugin::build(), proc_plugin::handler()])
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

fn input(tip: &str) -> Result<String> {
    let mut s = String::new();
    print!("{tip}: ");
    let _ = stdout().flush();
    stdin().read_line(&mut s)?;
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Ok(s)
}
