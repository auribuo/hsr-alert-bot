use anyhow::{anyhow, Result};
use serenity::all::{CreateMessage, Guild, GuildChannel, GuildId, PartialGuild, RoleId};
use serenity::{
    all::{Interaction, Ready},
    async_trait,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::{Context, EventHandler},
};

use crate::commands::CreateCommandVecExt;
use crate::config::{Config, RedeemCode};
use crate::{commands, CONFIG};

pub struct Handler {}

impl Handler {
    pub fn new() -> Self {
        Self {}
    }

    async fn run_alerts(ctx: Context) {
        loop {
            info!("Updating guild information");

            let config = CONFIG.read().await;
            Self::validate_info(&ctx, &config).await;
            drop(config);
            info!("Waiting for current codes");
            let new_codes = crate::CODE_CHAN.lock().await.as_mut().unwrap().recv().await;
            if let Some(codes) = new_codes {
                Self::handle_new_codes(&ctx, &codes).await
            }
        }
    }

    async fn handle_new_codes(ctx: &Context, codes: &Vec<(String, bool)>) {
        let mut config = CONFIG.write().await;
        for guild_diff in config.diff_guild_codes(codes) {
            if let Err(err) = Self::send_new_codes(
                guild_diff.0,
                guild_diff.1 .1,
                guild_diff.1 .0.alert_role,
                &ctx,
            )
            .await
            {
                error!(reason=err.to_string(), guild=?guild_diff.0, "Could not send codes");
            }
        }
    }

    async fn send_new_codes(
        guild_id: GuildId,
        codes: Vec<RedeemCode>,
        alert_role: Option<RoleId>,
        ctx: &Context,
    ) -> Result<()> {
        if codes.is_empty() {
            info!(guild=?guild_id, "No new codes to send");
            return Ok(());
        }
        let header = if let Some(role) = alert_role {
            format!("New Star Rail codes available <@&{role}>")
        } else {
            "New Star Rail codes available".to_string()
        };
        let body = codes
            .iter()
            .map(|code| {
                format!(
                    "> [{0}](https://hsr.hoyoverse.com/gift?code={0})",
                    code.code
                )
            })
            .fold(header, |acc, elem| acc + "\n" + elem.as_str());
        let alert_chan = Self::get_alert_channel(&guild_id, &ctx).await?;
        alert_chan
            .send_message(&ctx.http, CreateMessage::new().content(body))
            .await?;
        Ok(())
    }

    async fn validate_info(ctx: &Context, config: &Config) {
        match config.validate_info(&ctx).await {
            Ok(data) => {
                for reason in data.iter() {
                    if let Err(err) = config.alert_guild_invalid_info(&ctx, reason).await {
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

    async fn get_alert_channel(guild_maybe: &GuildId, ctx: &Context) -> Result<GuildChannel> {
        if let Ok(guild) = Guild::get(&ctx.http, guild_maybe).await {
            let config = Config::read()?;
            return if let Some(chan_id) = config.guild_alert_channel(guild.id) {
                let channel_result = guild.channels(&ctx.http).await;
                if let Err(err) = channel_result {
                    return Err(anyhow!("Error: {}", err));
                }
                let channels = channel_result.unwrap();
                let alert_channel_result = channels.iter().find(|(id, _)| **id == chan_id);
                if let None = alert_channel_result {
                    return Err(anyhow!("Alert channel does not exist"));
                }
                let (_, alert_channel) = alert_channel_result.unwrap();
                Ok(alert_channel.clone())
            } else {
                Err(anyhow!("No alert channel set"))
            };
        }
        return Err(anyhow!("Guild not found"));
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
        if let Err(err) = CONFIG.write().await.update_on_join(guild.id) {
            error!(reason = err.to_string(), "Could not update config");
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
            "{} connected to guilds:\n{:?}",
            ready.user.name,
            joined_guilds
                .iter()
                .map(|g| g.name.clone())
                .collect::<Vec<String>>()
        );

        info!("Updating guild info in config");

        if let Err(err) = CONFIG.write().await.update_guilds(&ready.guilds) {
            error!("Could not update guilds: {}", err);
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
