use serenity::{all::CommandInteraction, builder::CreateCommand};

use crate::config;

pub const CMD_NAME: &'static str = "disable";

pub fn run(interaction: &CommandInteraction) -> String {
    return if let Some(guild_id) = interaction.guild_id {
        if let Err(error) = config::set_guild_state(guild_id, true) {
            tracing::error!("{error}");
            "Failed to disable alerts.".to_string()
        } else {
            tracing::info!(
                "Disabled guild {guild_id} on request of {}",
                interaction.user.name
            );
            "Alerts disabled!".to_string()
        }
    } else {
        "Command run from something that is not a guild".to_string()
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new(CMD_NAME).description("Disable alerts for this server")
}
