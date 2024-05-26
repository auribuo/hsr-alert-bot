use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};

use crate::DB;

pub const CMD_NAME: &'static str = "announcement";

pub async fn run(interaction: &CommandInteraction, ctx: &Context, admin: &String) -> String {
    return if let Some(ResolvedOption {
        value: ResolvedValue::String(msg),
        ..
    }) = interaction.data.options().first()
    {
        if admin == &interaction.user.id.to_string() {
            if let Err(error) = DB
                .read()
                .await
                .as_ref()
                .unwrap()
                .send_to_all_guilds(msg.to_string(), &ctx)
                .await
            {
                error!("{error}");
                "Failed to send announcement".to_string()
            } else {
                info!("Sent announcement on request of {}", interaction.user.name);
                "Done!".to_string()
            }
        } else {
            warn!("User {} tried to send an announcement", interaction.user.id);
            "Announcements can only be sent from the admin.".to_string()
        }
    } else {
        "Provide a message please".to_string()
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new(CMD_NAME)
        .description("[ADMIN] Send an announcement")
        .add_option(CreateCommandOption::new(
            serenity::all::CommandOptionType::String,
            "message",
            "The message to send",
        ))
}
