use lazy_static::lazy_static;
use serenity::{all::GatewayIntents, Client};
use std::env;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, Mutex};

mod commands;
mod config;
mod handler;
mod scraper;

lazy_static! {
    static ref CODE_CHAN: Mutex<Option<Receiver<Vec<String>>>> = Mutex::new(None);
    static ref SHUTDOWN_RECV: Mutex<Option<tokio::sync::watch::Receiver<bool>>> = Mutex::new(None);
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Should be able to load dotenv!");
    tracing_subscriber::fmt().init();
    let token = env::var("DISCORD_TOKEN").expect("Discord token should be loaded in env");

    let (tx, rx) = mpsc::channel::<Vec<String>>(32);
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

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown_tx.send(true).unwrap()
        }
        _ = scraper::run(tx, shutdown_rx.clone()) => {}
    }
}
