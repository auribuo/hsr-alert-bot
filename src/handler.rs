use anyhow::{anyhow, Result};
use serenity::all::{CreateMessage, Guild, GuildChannel, UnavailableGuild};
use serenity::{
    all::{Interaction, Ready},
    async_trait,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::{Context, EventHandler},
};

use crate::commands::CreateCommandVecExt;
use crate::{commands, SHUTDOWN_RECV};

pub struct Handler {}

impl Handler {
    pub fn new() -> Self {
        Self {}
    }

    async fn run_alerts(guilds: Vec<&UnavailableGuild>, ctx: Context) {
        loop {
            let mut lock = SHUTDOWN_RECV.lock().await;
            if *lock.as_mut().expect("ERROR!").borrow_and_update() {
                return;
            }
            drop(lock);

            tracing::info!("Updating guild information");
            tracing::info!("Waiting for current codes");
            let new_codes = crate::CODE_CHAN.lock().await.as_mut().unwrap().recv().await;
            if let Some(codes) = new_codes {
                for guild in guilds.iter() {
                    match crate::config::get_codes_to_send(guild.id, &codes) {
                        Ok(send_codes) => match Self::get_alert_channel(*guild, &ctx).await {
                            Ok(chan) => {
                                let role_str = crate::config::guild_alert_role(guild.id)
                                    .map_or("".to_string(), |opt| format!("<@&{}>", opt));

                                let codes_str = send_codes
                                    .iter()
                                    .map(|c| format!("`{}`", c))
                                    .fold("".to_string(), |acc, e| acc + e.as_str() + "\n");

                                let msg = format!(
                                    "New Star Rail codes available {}\n{}",
                                    role_str, codes_str
                                );

                                if !send_codes.is_empty() {
                                    if let Err(err) = chan
                                        .send_message(&ctx.http, CreateMessage::new().content(&msg))
                                        .await
                                    {
                                        tracing::error!(
                                        "Could not send message to channel {}: {}",
                                        chan.name,
                                        err
                                    )
                                    } else {
                                        tracing::info!(
                                        "Sent message: {} to channel {}",
                                        &msg,
                                        chan.name
                                    );
                                        if let Err(err) = crate::config::update_sent_codes(guild.id, &send_codes) {
                                            tracing::error!("Error: {}", err);
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                tracing::error!(
                                    "Could not determine alert channel for guild {}: {}",
                                    guild.id,
                                    err
                                );
                            }
                        },
                        Err(error) => {
                            tracing::error!("Could not get new codes to send: {}", error);
                        }
                    }
                }
            }
        }
    }

    async fn get_alert_channel(
        guild_maybe: &UnavailableGuild,
        ctx: &Context,
    ) -> Result<GuildChannel> {
        if let Ok(guild) = Guild::get(&ctx.http, guild_maybe.id).await {
            if let Some(chan_id) = crate::config::guild_alert_channel(guild.id) {
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
                return Ok(alert_channel.clone());
            }
        }
        return Err(anyhow!("Guild not found"));
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} has connected!", ready.user.name);

        let commands = vec![
            commands::enable::register(),
            commands::disable::register(),
            commands::set_alert_channel::register(),
            commands::set_alert_role::register(),
            commands::subscribe::register(),
        ];

        commands.global_register_all(&ctx.http).await;

        tokio::spawn(async move {
            Self::run_alerts(ready.guilds.iter().collect(), ctx.clone()).await;
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            tracing::info!("Received interaction from {}", command.user.name);

            let content = match command.data.name.as_str() {
                commands::enable::CMD_NAME => Some(commands::enable::run(&command)),
                commands::disable::CMD_NAME => Some(commands::disable::run(&command)),
                commands::set_alert_channel::CMD_NAME => Some(commands::set_alert_channel::run(&command)),
                commands::set_alert_role::CMD_NAME => Some(commands::set_alert_role::run(&command)),
                commands::subscribe::CMD_NAME => Some(commands::subscribe::run(&command, &ctx).await),
                _ => {
                    tracing::warn!("Received invalid command");
                    None
                }
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    tracing::error!("Cannot respond to slash command: {why}")
                }
            }
        }
    }
}
