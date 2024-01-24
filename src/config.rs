use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId, RoleId};

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

pub fn read() -> anyhow::Result<Vec<GuildInfo>> {
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
    if let Some(guild_info) = config.iter_mut().find(|g| g.id == guild) {
        guild_info.enabled = enabled;
        return Ok(write(config)?);
    } else {
        return Err(anyhow!("Guild {guild} not found."));
    }
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
