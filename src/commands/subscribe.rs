use crate::DB;
use serenity::all::Context;
use serenity::{all::CommandInteraction, builder::CreateCommand};

pub const CMD_NAME: &'static str = "subscribe";

pub async fn run(interaction: &CommandInteraction, ctx: &Context) -> String {
    return if let Some(guild_id) = interaction.guild_id {
        return if let Some(member) = &interaction.member {
            if let Ok(Some(role)) = DB
                .read()
                .await
                .as_ref()
                .unwrap()
                .guild_alert_role(guild_id)
                .await
            {
                if let Ok(()) = member.add_role(&ctx, role).await {
                    "Subscribed you to the alerts!".to_string()
                } else {
                    "Could not add the role!".to_string()
                }
            } else {
                "The alert role is not enabled on your server. You might want to add one using `/alert-role`".to_string()
            }
        } else {
            "Apparently you are not member of this server???".to_string()
        };
    } else {
        "Command run from something that is not a guild".to_string()
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new(CMD_NAME).description("Enable alerts for this server")
}
