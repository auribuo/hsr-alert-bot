use anyhow::{anyhow, Result};
use serenity::all::{CreateMessage, Guild, GuildId, PartialGuild};
use serenity::{
    all::{Interaction, Ready},
    async_trait,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::{Context, EventHandler},
};

use crate::commands::CreateCommandVecExt;
use crate::db::{GuildUpdate, TursoDb};
use crate::{commands, DB};

pub struct Handler;

impl Handler {
    async fn run_alerts(ctx: Context) {
        loop {
            info!("Validating guild information");

            Self::validate_info(&ctx, DB.read().await.as_ref().unwrap()).await;
            info!("Waiting for current codes from scaper");
            let new_codes = crate::CODE_CHAN.lock().await.as_mut().unwrap().recv().await;
            if let Some(codes) = new_codes {
                if let Err(err) = Self::handle_new_codes(&ctx, &codes).await {
                    error!(reason = err.to_string(), "Failed to handle new codes")
                }
            }
        }
    }

    async fn handle_new_codes(ctx: &Context, codes: &Vec<String>) -> Result<()> {
        let db_opt = DB.read().await;
        let db = db_opt.as_ref().unwrap();
        for guild_diff in db.diff_guild_codes(codes, ctx).await? {
            if let Err(err) = Self::send_new_codes(&guild_diff.1, &ctx).await {
                error!(reason=err.to_string(), guild=?guild_diff.0, "Could not send codes");
            } else {
                info!(guild=?guild_diff.0, "Sent codes to guild");
                db.set_codes_sent(guild_diff.0, guild_diff.1.codes).await?;
            }
        }
        Ok(())
    }

    async fn send_new_codes(update: &GuildUpdate, ctx: &Context) -> Result<()> {
        if !update.has_codes() {
            info!(guild=?update.id, "No new codes to send");
            return Ok(());
        }
        let header = if let Some(role) = update.role {
            format!("New Star Rail codes available <@&{role}>")
        } else {
            "New Star Rail codes available".to_string()
        };

        let body = update
            .codes
            .as_ref()
            .unwrap()
            .iter()
            .map(|code| {
                format!(
                    "> [{0}](https://hsr.hoyoverse.com/gift?code={0})",
                    code.code
                )
            })
            .fold(header, |acc, elem| acc + "\n" + elem.as_str());
        let Some(alert_chan) = update.chan else {
            return Err(anyhow!("No alert channel set"));
        };
        alert_chan
            .send_message(&ctx.http, CreateMessage::new().content(body))
            .await?;
        info!(guild=?update.id, "Sent codes to guild");
        Ok(())
    }

    async fn validate_info(ctx: &Context, db: &TursoDb) {
        match db.validate_info(&ctx).await {
            Ok(data) => {
                for reason in data.iter() {
                    if let Err(err) = db.alert_guild_invalid_info(&ctx, reason).await {
                        error!(
                            "Could not alert guild {} of invalid info: {}",
                            (*reason).0.clone(),
                            err
                        )
                    }
                }
            }
            Err(err) => {
                error!("Could not validate if guild info is up-to-date: {err}");
            }
        }
    }
}

async fn resolve_guilds(guild_id: &GuildId, ctx: &Context) -> Result<PartialGuild> {
    return match Guild::get(&ctx.http, guild_id).await {
        Ok(guild) => Ok(guild),
        Err(error) => Err(anyhow::Error::new(error)),
    };
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, _: Context, guild: Guild, _: Option<bool>) {
        match DB
            .read()
            .await
            .as_ref()
            .unwrap()
            .try_add_guild(guild.id)
            .await
        {
            Ok(inserted) => {
                if !inserted {
                    warn!(guild=?guild.id, "Guild was already in db");
                }
            }
            Err(err) => {
                error!(reason = err.to_string(), "Could not add guild");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} has connected!", ready.user.name);

        let mut joined_guilds: Vec<PartialGuild> = vec![];

        for guild in ready.guilds.iter() {
            if let Ok(guild) = resolve_guilds(&guild.id, &ctx).await {
                joined_guilds.push(guild);
            }
        }

        info!(
            guild_count = joined_guilds.len(),
            "{} connected", ready.user.name,
        );

        info!("Updating guild info");

        if let Err(err) = DB
            .read()
            .await
            .as_ref()
            .unwrap()
            .update_guilds(&ready.guilds)
            .await
        {
            error!(reason = err.to_string(), "Could not update guilds");
        }

        let commands = vec![
            commands::enable::register(),
            commands::disable::register(),
            commands::set_alert_channel::register(),
            commands::set_alert_role::register(),
            commands::subscribe::register(),
        ];

        commands.global_register_all(&ctx.http).await;

        tokio::spawn(async move {
            Self::run_alerts(ctx.clone()).await;
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!("Received interaction from {}", command.user.name);

            let content = match command.data.name.as_str() {
                commands::enable::CMD_NAME => Some(commands::enable::run(&command).await),
                commands::disable::CMD_NAME => Some(commands::disable::run(&command).await),
                commands::set_alert_channel::CMD_NAME => {
                    Some(commands::set_alert_channel::run(&command).await)
                }
                commands::set_alert_role::CMD_NAME => {
                    Some(commands::set_alert_role::run(&command).await)
                }
                commands::subscribe::CMD_NAME => {
                    Some(commands::subscribe::run(&command, &ctx).await)
                }
                _ => {
                    warn!("Received invalid command");
                    None
                }
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    error!("Cannot respond to slash command: {why}")
                }
            }
        }
    }
}
