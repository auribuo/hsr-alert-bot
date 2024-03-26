use crate::DB;
use serenity::{all::CommandInteraction, builder::CreateCommand};

pub const CMD_NAME: &'static str = "disable";

pub async fn run(interaction: &CommandInteraction) -> String {
    return if let Some(guild_id) = interaction.guild_id {
        if let Err(error) = DB
            .read()
            .await
            .as_ref()
            .unwrap()
            .set_guild_state(guild_id, true)
            .await
        {
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
