use lazy_static::lazy_static;
use serenity::{all::GatewayIntents, Client};
use std::env;
use std::env::args;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::Level;

mod commands;
mod config;
mod handler;
mod scraper;

lazy_static! {
    static ref CODE_CHAN: Mutex<Option<Receiver<Vec<(String, bool)>>>> = Mutex::new(None);
    static ref SHUTDOWN_RECV: Mutex<Option<tokio::sync::watch::Receiver<bool>>> = Mutex::new(None);
    static ref CONFIG: RwLock<config::Config> = RwLock::new(config::Config::read().unwrap());
}

#[macro_use]
extern crate tracing;

static SCRAPER_INTERVAL: u64 = 3600;

#[tokio::main]
async fn main() {
    let token_env = args()
        .skip(1)
        .find(|a| a.as_str() == "dev")
        .map_or_else(|| "DISCORD_TOKEN", |_| "TOKEN_DEV");

    dotenv::dotenv().expect("Should be able to load dotenv!");
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let token = env::var(token_env).expect("Discord token should be loaded in env");

    let (tx, rx) = mpsc::channel::<Vec<(String, bool)>>(32);
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let mut glob_chan = CODE_CHAN.lock().await;
    *glob_chan = Some(rx);
    drop(glob_chan);

    let mut glob_shutdown_chan = SHUTDOWN_RECV.lock().await;
    *glob_shutdown_chan = Some(shutdown_rx.clone());
    drop(glob_shutdown_chan);

    let handler = handler::Handler::new();
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Error creating client");

    tokio::spawn(async move {
        client.start().await.unwrap();
    });

    info!("Starting scraper with an interval of {}s", SCRAPER_INTERVAL);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown_tx.send(true).unwrap()
        }
        _ = scraper::run(tx, shutdown_rx.clone(), SCRAPER_INTERVAL) => {}
    }
}
