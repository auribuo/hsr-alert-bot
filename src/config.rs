use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, PartialGuild, RoleId, UnavailableGuild, Guild, GuildChannel};

#[derive(Serialize, Deserialize)]
pub struct GuildInfo {
    id: GuildId,
    alert_role: Option<RoleId>,
    alert_channel: Option<ChannelId>,
    enabled: bool,
    codes: Vec<CodeInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct CodeInfo {
    code: String,
    added_at: String,
}

pub fn read() -> Result<Vec<GuildInfo>> {
    if let Ok(content) = std::fs::read_to_string("cfg.json") {
        if let Ok(cfg) = serde_json::from_str(&content) {
            Ok(cfg)
        } else {
            Err(anyhow!("Could not parse config"))
        }
    } else {
        Err(anyhow!("Could not read config file"))
    }
}

pub fn write(cfg: Vec<GuildInfo>) -> Result<()> {
    Ok(std::fs::write(
        "cfg.json",
        serde_json::to_string(&cfg).expect("Shouldn't this unwrap?"),
    )?)
}

pub fn set_guild_state(guild: GuildId, enabled: bool) -> Result<()> {
    let mut config = read()?;
    return if let Some(guild_info) = config.iter_mut().find(|g| g.id == guild) {
        guild_info.enabled = enabled;
        Ok(write(config)?)
    } else {
        Err(anyhow!("Guild {guild} not found."))
    };
}

pub fn set_guild_alert_role(guild: GuildId, role: Option<RoleId>) -> Result<()> {
    let mut config = read()?;
    return if let Some(guild_info) = config.iter_mut().find(|g| g.id == guild) {
        guild_info.alert_role = role;
        Ok(write(config)?)
    } else {
        Err(anyhow!("Guild {guild} not found."))
    };
}

pub fn guild_alert_role(guild: GuildId) -> Option<RoleId> {
    if let Ok(config) = read() {
        if let Some(guild_info) = config.iter().find(|g| g.id == guild) {
            return guild_info.alert_role;
        }
    }
    return None;
}

pub fn set_guild_alert_channel(guild: GuildId, channel: Option<ChannelId>) -> Result<()> {
    let mut config = read()?;
    return if let Some(guild_info) = config.iter_mut().find(|g| g.id == guild) {
        guild_info.alert_channel = channel;
        Ok(write(config)?)
    } else {
        Err(anyhow!("Guild {guild} not found."))
    };
}

pub fn guild_alert_channel(guild: GuildId) -> Option<ChannelId> {
    if let Ok(config) = read() {
        if let Some(guild_info) = config.iter().find(|g| g.id == guild) {
            return guild_info.alert_channel;
        }
    }
    None
}

pub fn update_sent_codes(guild: GuildId, codes: &Vec<String>) -> Result<()> {
    let mut config = read()?;
    return if let Some(guild_info) = config.iter_mut().find(|g| g.id == guild) {
        for code in codes.iter() {
            if guild_info
                .codes
                .iter()
                .find(|&x| (*x.code) == *code)
                .is_none()
            {
                guild_info.codes.push(CodeInfo {
                    code: code.clone(),
                    added_at: chrono::Local::now().to_rfc3339(),
                })
            }
        }
        Ok(write(config)?)
    } else {
        Err(anyhow!("Guild {guild} not found."))
    };
}

pub fn get_codes_to_send(guild: GuildId, new_codes: &Vec<String>) -> Result<Vec<String>> {
    let mut config = read()?;
    return if let Some(guild_info) = config.iter_mut().find(|g| g.id == guild) {
        Ok(new_codes
            .iter()
            .filter(|c| guild_info.codes.iter().find(|ci| *ci.code == **c).is_none())
            .map(|s| s.clone())
            .collect())
    } else {
        Err(anyhow!("Guild {guild} not found."))
    };
}

pub fn update_guilds(guilds: &Vec<UnavailableGuild>) -> Result<()> {
    let mut config = read()?;
    for guild in guilds {
        if config.iter().find(|info| info.id == guild.id).is_none() {
            config.push(GuildInfo {
                id: guild.id,
                alert_channel: None,
                alert_role: None,
                enabled: true,
                codes: vec![],
            });
        }
    }
    Ok(write(config)?)
}

pub fn update_guild(guild: &Guild) -> Result<()> {
    let mut config = read()?;
    if config.iter().find(|info| info.id == guild.id).is_none() {
        config.push(GuildInfo {
            id: guild.id,
            alert_channel: None,
            alert_role: None,
            enabled: true,
            codes: vec![],
        });
    }
    Ok(write(config)?)
}

#[derive(Debug)]
pub enum InvalidInfo {
    Channel(Option<ChannelId>),
    Role(RoleId),
    Both(Option<ChannelId>, RoleId),
}

pub async fn validate_info(ctx: &Context) -> Result<Vec<(GuildId, InvalidInfo)>> {
    let config = read()?;
    let mut invalid_guilds: Vec<(GuildId, InvalidInfo)> = vec![];
    for guild in config.iter() {
        let g = resolve_guild(&guild.id, &ctx).await?;
        let channel_valid = guild.alert_channel.is_some() && g.channels(&ctx.http).await?.iter().find(|(id, _)| **id == guild.alert_channel.unwrap()).is_some();
        let mut role_valid = true;
        if guild.alert_role.is_some() {
            role_valid = g.roles.iter().find(|(id, _)| **id == guild.alert_role.unwrap()).is_some();
        }
        if !channel_valid && !role_valid {
            invalid_guilds.push((guild.id, InvalidInfo::Both(guild.alert_channel, guild.alert_role.unwrap())))
        }
        if !channel_valid {
            invalid_guilds.push((guild.id, InvalidInfo::Channel(guild.alert_channel)))
        }
        if !role_valid {
            invalid_guilds.push((guild.id, InvalidInfo::Role(guild.alert_role.unwrap())))
        }
    }

    Ok(invalid_guilds)
}

pub async fn alert_guild_invalid_info(ctx: &Context, reason: &(GuildId, InvalidInfo)) -> Result<()> {
    let default_chan = get_default_channel(reason.0, &ctx).await?;
    match reason.1 {
        InvalidInfo::Channel(chan_id) => {
            if let Some(id) = chan_id {
                default_chan.send_message(&ctx.http, CreateMessage::new().content(format!("The channel (id={}) you set for the alerts is not valid anymore. Please set it again", id))).await?;
            } else {
                default_chan.send_message(&ctx.http, CreateMessage::new().content("No alert channel found. You might want to set the channel using: `/alert-channel`")).await?;
            }
        }
        InvalidInfo::Role(role_id) => {
            default_chan.send_message(&ctx.http, CreateMessage::new().content(format!("The role (id={}) you set for the alerts is not valid anymore. Please set it again", role_id))).await?;
        }
        InvalidInfo::Both(chan_id, role_id) => {
            if let Some(id) = chan_id {
                default_chan.send_message(&ctx.http, CreateMessage::new().content(format!("The channel (id={}) you set for the alerts is not valid anymore. Please set it again", id))).await?;
            } else {
                default_chan.send_message(&ctx.http, CreateMessage::new().content("No alert channel found. You might want to set the channel using: `/alert-channel`")).await?;
            }
            default_chan.send_message(&ctx.http, CreateMessage::new().content(format!("The role (id={}) you set for the alerts is not valid anymore. Please set it again", role_id))).await?;
        }
    }
    tracing::info!("Sent alert matching {:#?} to guild {}", (*reason).1, (*reason).0);
    Ok(())
}

pub async fn get_default_channel(guild_id: GuildId, ctx: &Context) -> Result<GuildChannel> {
    return if let Ok(guild) = resolve_guild(&guild_id, &ctx).await {
        return if let Ok(channels) = guild.channels(&ctx.http).await {
            let default_chan: GuildChannel;
            if let Some(system_channel_id) = guild.system_channel_id {
                default_chan = channels.iter().find(|(id, _)| **id == system_channel_id).expect("Should exist").1.clone();
            } else {
                if let Some((_, first_channel)) = channels.iter().find(|_| true) {
                    default_chan = first_channel.clone();
                } else {
                    return Err(anyhow!("Could not get any channel for guild: {}", guild_id));
                }
            }
            return Ok(default_chan);
        } else {
            Err(anyhow!("Could not get channels for guild: {}", guild_id))
        };
    } else {
        Err(anyhow!("Could not retrieve info for guild {}", guild_id))
    };
}

pub async fn resolve_guild(id: &GuildId, ctx: &Context) -> Result<PartialGuild> {
    Ok(Guild::get(&ctx.http, id).await?)
}