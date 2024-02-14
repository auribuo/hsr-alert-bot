use crate::CONFIG;
use serenity::all::{
    CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption,
    ResolvedValue,
};

pub const CMD_NAME: &'static str = "alert-role";

pub async fn run(interaction: &CommandInteraction) -> String {
    let mut cfg = CONFIG.write().await;
    return if let Some(ResolvedOption {
        value: ResolvedValue::Role(role),
        ..
    }) = interaction.data.options().first()
    {
        if let Some(guild_id) = interaction.guild_id {
            if let Err(error) = cfg.set_guild_alert_role(guild_id, Some(role.id)) {
                tracing::error!("{error}");
                "Could not set alert role due to an internal error".to_string()
            } else {
                tracing::info!(
                    "Set alert role for guild {guild_id} to {} on request of {}",
                    role.name,
                    interaction.user.name
                );
                format!("Set alert role to: {}", role.name)
            }
        } else {
            "Command run from something that is not a guild".to_string()
        }
    } else {
        if let Some(guild_id) = interaction.guild_id {
            if let Err(err) = cfg.set_guild_alert_role(guild_id, None) {
                tracing::error!("Error: {}", err);
                "Could not remove the alert role because of an internal error".to_string()
            } else {
                tracing::info!(
                    "Removed alerts role at request of {}",
                    interaction.user.name
                );
                "Successfully removed the alert role".to_string()
            }
        } else {
            "Command run from something that is not a guild".to_string()
        }
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new(CMD_NAME)
        .description("Set the alert role for this server")
        .add_option(CreateCommandOption::new(
            CommandOptionType::Role,
            "role",
            "The role to use as an alert role",
        ))
}
