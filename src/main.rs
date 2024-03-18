use lazy_static::lazy_static;
use libsql::Connection;
use serenity::{all::GatewayIntents, Client};
use shuttle_secrets::{SecretStore, Secrets};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, Mutex, RwLock};

mod commands;
mod config;
mod handler;
mod scraper;

lazy_static! {
    static ref CODE_CHAN: Mutex<Option<Receiver<Vec<(String, bool)>>>> = Mutex::new(None);
    static ref CONFIG: RwLock<config::Config> = RwLock::new(config::Config::read().unwrap());
}

#[macro_use]
extern crate tracing;

static SCRAPER_INTERVAL: u64 = 3600;

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secrets: SecretStore,
    #[shuttle_turso::Turso(
        addr = "libsql://hsr-alert-bot-auribuo.turso.io",
        token = "{secrets.TURSO_TOKEN}"
    )]
    client: Connection,
) -> shuttle_serenity::ShuttleSerenity {
    let token = secrets.get("DISCORD_TOKEN").expect("No token provided");

    let (tx, rx) = mpsc::channel::<Vec<(String, bool)>>(32);

    let mut glob_chan = CODE_CHAN.lock().await;
    *glob_chan = Some(rx);
    drop(glob_chan);

    tokio::spawn(async move {
        info!(interval = SCRAPER_INTERVAL, "Starting scraper");
        scraper::run(tx, SCRAPER_INTERVAL).await
    });

    let handler = handler::Handler::new();
    let client = Client::builder(token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Error creating client");
    Ok(client.into())
}
