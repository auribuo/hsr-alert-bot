use std::process::exit;
use std::sync::Arc;

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
    if let Err(err) = client
        .execute(
            r#"
                CREATE TABLE IF NOT EXISTS guilds (
                    id integer primary key autoincrement,
                    guild_id int unique not null,
                    enabled integer not null default 1,
                    last_code int not null default 0,
                    alert_channel int null default null,
                    alert_role int null default null
                );
            "#,
            (),
        )
        .await
    {
        error!(reason = err.to_string(), "Cannot initialize db");
        exit(1);
    }

    if let Err(err) = client
        .execute(
            r#"
                CREATE TABLE IF NOT EXISTS codes (
                    id integer primary key autoincrement,
                    code varchar(50) not null unique,
                    valid integer not null
                );
            "#,
            (),
        )
        .await
    {
        error!(reason = err.to_string(), "Cannot initialize db");
        exit(1);
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

    let handler = handler::Handler::new();
    let client = Client::builder(token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Error creating client");
    Ok(client.into())
}
