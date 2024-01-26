use crate::config;
use serenity::all::{
    CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption,
    ResolvedValue,
};

pub const CMD_NAME: &'static str = "alert-channel";

pub fn run(interaction: &CommandInteraction) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Channel(channel),
        ..
    }) = interaction.data.options().first()
    {
        return if let Some(guild_id) = interaction.guild_id {
            // TODO check if channel text channel
            if let Err(error) = config::set_guild_alert_channel(guild_id, Some(channel.id)) {
                tracing::error!("{error}");
                "Could not set alert channel due to an internal error".to_string()
            } else {
                tracing::info!(
                    "Set alert channel for guild {guild_id} to {} on request of {}",
                    channel.name.as_ref().unwrap(),
                    interaction.user.name
                );
                format!(
                    "Set alert channel to: {}",
                    channel.name.as_ref().expect("No name?")
                )
            }
        } else {
            "Command run from something that is not a guild".to_string()
        };
    } else {
        return if let Some(guild_id) = interaction.guild_id {
            if let Err(err) = config::set_guild_alert_channel(guild_id, None) {
                tracing::error!("Error: {}", err);
                "Could not remove the alert channel because of an internal error".to_string()
            } else {
                tracing::info!(
                    "Removed alerts channel at request of {}",
                    interaction.user.name
                );
                "Successfully removed the alert channel".to_string()
            }
        } else {
            "Command run from something that is not a guild".to_string()
        };
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new(CMD_NAME)
        .description("Enable alerts for this server")
        .add_option(CreateCommandOption::new(
            CommandOptionType::Channel,
            "channel",
            "The channel to use as an alert channel",
        ))
}
