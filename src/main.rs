use std::sync::Arc;

use anyhow::anyhow;
use lazy_static::lazy_static;
use libsql::Connection;
use serenity::{all::GatewayIntents, Client};
use shuttle_runtime::{main, SecretStore, Secrets};
use shuttle_turso::Turso;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::db::TursoDb;

mod commands;
mod db;
mod handler;
mod scraper;

lazy_static! {
    static ref CODE_CHAN: Mutex<Option<Receiver<Vec<String>>>> = Mutex::new(None);
    static ref DB: RwLock<Option<TursoDb>> = RwLock::new(None);
}

#[macro_use]
extern crate tracing;

static SCRAPER_INTERVAL: u64 = 3600;

#[main]
async fn app(
    #[Secrets] secrets: SecretStore,
    #[Turso(
        addr = "libsql://hsr-alert-bot-auribuo.turso.io",
        token = "{secrets.TURSO_TOKEN}"
    )]
    client: Connection,
) -> shuttle_serenity::ShuttleSerenity {
    let token = secrets.get("DISCORD_TOKEN").expect("No token provided");

    info!("Initializing db");
    if let Err(err) = client.execute(include_str!("../sql/guilds.sql"), ()).await {
        return Err(anyhow!(
            "Cannot initialize db. Failed to set up table guilds: {}",
            err.to_string()
        )
        .into());
    }

    if let Err(err) = client.execute(include_str!("../sql/codes.sql"), ()).await {
        return Err(anyhow!(
            "Cannot initialize db. Failed to set up table codes: {}",
            err.to_string()
        )
        .into());
    }

    *DB.write().await = Some(TursoDb::new(Arc::new(client)).await.unwrap());

    let (tx, rx) = mpsc::channel::<Vec<String>>(32);

    let mut glob_chan = CODE_CHAN.lock().await;
    *glob_chan = Some(rx);
    drop(glob_chan);

    tokio::spawn(async move {
        info!(interval = SCRAPER_INTERVAL, "Starting scraper");
        scraper::run(tx, SCRAPER_INTERVAL).await
    });

    let client = Client::builder(token, GatewayIntents::empty())
        .event_handler(handler::Handler {
            admin: secrets.get("ADMIN").expect("Admin should be set"),
        })
        .await
        .expect("Error creating client");
    Ok(client.into())
}
