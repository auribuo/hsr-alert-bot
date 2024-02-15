use anyhow::{anyhow, Result};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use serenity::all::{
    ChannelId, Context, CreateMessage, Guild, GuildChannel, GuildId, PartialGuild, RoleId,
    UnavailableGuild,
};
use std::collections::HashMap;

static CONFIG_FILE: &'static str = "cfg.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    codes: Vec<RedeemCode>,
    guilds: Vec<GuildInfo>,
}

impl Config {
    pub fn read() -> Result<Self> {
        match std::fs::read_to_string(CONFIG_FILE) {
            Ok(content) => match serde_json::from_str::<Config>(&content) {
                Ok(cfg) => Ok(cfg),
                Err(err) => Err(anyhow!("Could not parse config: {err}")),
            },
            Err(err) => Err(anyhow!("Could not read config file: {err}")),
        }
    }

    pub fn diff_guild_codes(
        &mut self,
        new_codes: &Vec<(String, bool)>,
    ) -> HashMap<GuildId, (GuildInfo, Vec<RedeemCode>)> {
        let mut uid = RedeemCode::next_uid(self);
        let mut actually_new = new_codes
            .iter()
            .filter(|&code| {
                !self
                    .codes
                    .iter()
                    .map(|c| &c.code)
                    .collect::<Vec<_>>()
                    .contains(&&code.0)
            })
            .map(|code| {
                let code = RedeemCode::new(code.0.clone(), code.1, uid);
                uid = uid + 1;
                dbg!(uid);
                code
            })
            .collect::<Vec<_>>();
        self.codes.append(&mut actually_new);

        let mut diff_codes = HashMap::<GuildId, (GuildInfo, Vec<RedeemCode>)>::new();
        for guild in self.guilds.iter_mut() {
            diff_codes.insert(
                guild.id,
                (
                    guild.clone(),
                    Self::codes_from(&mut self.codes, guild.last_code),
                ),
            );
            guild.last_code = Self::last_code(&self.codes)
        }

        diff_codes
    }

    pub fn last_code(codes: &Vec<RedeemCode>) -> u64 {
        codes
            .iter()
            .max_by(|code, other| code.uid.cmp(&other.uid))
            .map_or_else(|| 0, |opt| opt.uid)
    }

    pub fn codes_from(codes: &mut Vec<RedeemCode>, uid: u64) -> Vec<RedeemCode> {
        codes.sort_by(|code, other| code.uid.cmp(&other.uid));
        let mut new_codes: Vec<RedeemCode> = vec![];
        for code in codes.iter().skip_while(|&code| code.uid <= uid) {
            if code.is_valid() {
                new_codes.push(code.clone())
            }
        }
        new_codes
    }

    pub fn save(&mut self) -> Result<()> {
        match serde_json::to_string(&self) {
            Ok(json) => match std::fs::write(CONFIG_FILE, json) {
                Ok(()) => Ok(()),
                Err(err) => Err(anyhow!("Could not write content: {err}")),
            },
            Err(err) => Err(anyhow!("Could not serialize config: {err}")),
        }
    }

    pub fn set_guild_state(&mut self, guild: GuildId, enabled: bool) -> Result<()> {
        return if let Some(guild_info) = self.guilds.iter_mut().find(|g| g.id == guild) {
            guild_info.enabled = enabled;
            Ok(self.save()?)
        } else {
            Err(anyhow!("Guild {guild} not found."))
        };
    }

    pub fn set_guild_alert_role(&mut self, guild: GuildId, role: Option<RoleId>) -> Result<()> {
        return if let Some(guild_info) = self.guilds.iter_mut().find(|g| g.id == guild) {
            guild_info.alert_role = role;
            Ok(self.save()?)
        } else {
            Err(anyhow!("Guild {guild} not found."))
        };
    }

    pub fn guild_alert_role(&self, guild: GuildId) -> Option<RoleId> {
        return if let Some(guild_info) = self.guilds.iter().find(|g| g.id == guild) {
            guild_info.alert_role
        } else {
            None
        };
    }

    pub fn set_guild_alert_channel(
        &mut self,
        guild: GuildId,
        channel: Option<ChannelId>,
    ) -> Result<()> {
        return if let Some(guild_info) = self.guilds.iter_mut().find(|g| g.id == guild) {
            guild_info.alert_channel = channel;
            Ok(self.save()?)
        } else {
            Err(anyhow!("Guild {guild} not found."))
        };
    }

    pub fn guild_alert_channel(&self, guild: GuildId) -> Option<ChannelId> {
        return if let Some(guild_info) = self.guilds.iter().find(|g| g.id == guild) {
            guild_info.alert_channel
        } else {
            None
        };
    }

    pub fn update_guilds(&mut self, guilds: &Vec<UnavailableGuild>) -> Result<()> {
        for guild in guilds {
            if self
                .guilds
                .iter()
                .find(|info| info.id == guild.id)
                .is_none()
            {
                warn!(guild=?&guild.id, "New guild found. Adding to config");
                self.guilds.push(GuildInfo {
                    id: guild.id,
                    alert_channel: None,
                    alert_role: None,
                    enabled: true,
                    last_code: 0,
                });
            }
        }
        Ok(self.save()?)
    }

    pub fn update_on_join(&mut self, guild: Guild) -> Result<()> {
        if self
            .guilds
            .iter()
            .find(|info| info.id == guild.id)
            .is_none()
        {
            warn!(guild=?&guild.id, "New guild joined. Adding to config");
            self.guilds.push(GuildInfo {
                id: guild.id,
                alert_channel: None,
                alert_role: None,
                enabled: true,
                last_code: 0,
            });
        }
        Ok(())
    }

    pub async fn validate_info(&self, ctx: &Context) -> Result<Vec<(GuildId, InvalidInfo)>> {
        let mut invalid_guilds: Vec<(GuildId, InvalidInfo)> = vec![];
        for guild in self.guilds.iter() {
            let g = Self::get_guild(&guild.id, &ctx).await?;
            let channel_valid = guild.alert_channel.is_some()
                && g.channels(&ctx.http)
                    .await?
                    .iter()
                    .find(|(id, _)| **id == guild.alert_channel.unwrap())
                    .is_some();
            let mut role_valid = true;
            if guild.alert_role.is_some() {
                role_valid = g
                    .roles
                    .iter()
                    .find(|(id, _)| **id == guild.alert_role.unwrap())
                    .is_some();
            }
            if !channel_valid && !role_valid {
                invalid_guilds.push((
                    guild.id,
                    InvalidInfo::Both(guild.alert_channel, guild.alert_role.unwrap()),
                ))
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

    pub async fn alert_guild_invalid_info(
        &self,
        ctx: &Context,
        reason: &(GuildId, InvalidInfo),
    ) -> Result<()> {
        let default_chan = self.get_default_channel(reason.0, &ctx).await?;
        warn!(guild=?&reason.0, invalid=?&reason.1, "Guild has invalid info");
        match reason.1 {
            InvalidInfo::Channel(chan_id) => {
                Self::alert_invalid_channel(&ctx, chan_id, &default_chan).await?;
            }
            InvalidInfo::Role(role_id) => {
                Self::alert_invalid_role(&ctx, role_id, &default_chan).await?;
            }
            InvalidInfo::Both(chan_id, role_id) => {
                Self::alert_invalid_channel(&ctx, chan_id, &default_chan).await?;
                Self::alert_invalid_role(&ctx, role_id, &default_chan).await?;
            }
        }
        tracing::info!(
            "Sent alert matching {:#?} to guild {}",
            (*reason).1,
            (*reason).0
        );
        Ok(())
    }

    async fn alert_invalid_channel(
        ctx: &Context,
        chan_id: Option<ChannelId>,
        default_chan: &GuildChannel,
    ) -> Result<()> {
        if let Some(id) = chan_id {
            default_chan.send_message(&ctx.http, CreateMessage::new().content(format!("The channel (id={}) you set for the alerts is not valid anymore. Please set it again", id))).await?;
        } else {
            default_chan.send_message(&ctx.http, CreateMessage::new().content("No alert channel found. You might want to set the channel using: `/alert-channel`")).await?;
        }
        Ok(())
    }

    async fn alert_invalid_role(
        ctx: &Context,
        role_id: RoleId,
        default_chan: &GuildChannel,
    ) -> Result<()> {
        default_chan.send_message(&ctx.http, CreateMessage::new().content(format!("The role (id={}) you set for the alerts is not valid anymore. Please set it again", role_id))).await?;
        Ok(())
    }

    async fn get_default_channel(&self, guild_id: GuildId, ctx: &Context) -> Result<GuildChannel> {
        return if let Ok(guild) = Self::get_guild(&guild_id, &ctx).await {
            return if let Ok(channels) = guild.channels(&ctx.http).await {
                let default_chan: GuildChannel;
                if let Some(system_channel_id) = guild.system_channel_id {
                    default_chan = channels
                        .iter()
                        .find(|(id, _)| **id == system_channel_id)
                        .expect("Should exist")
                        .1
                        .clone();
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

    pub async fn get_guild(id: &GuildId, ctx: &Context) -> Result<PartialGuild> {
        Ok(Guild::get(&ctx.http, id).await?)
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        self.save().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedeemCode {
    pub(crate) code: String,
    pub(crate) added: chrono::NaiveDateTime,
    pub(crate) uid: u64,
    pub(crate) is_version_code: bool,
}

impl RedeemCode {
    fn new(code: String, is_version_code: bool, uid: u64) -> Self {
        Self {
            code,
            uid,
            added: chrono::Utc::now().naive_utc(),
            is_version_code,
        }
    }

    fn next_uid(config: &Config) -> u64 {
        config
            .codes
            .iter()
            .max_by(|code, other| code.uid.cmp(&other.uid))
            .map_or_else(|| 0, |code| code.uid + 1)
    }

    fn is_valid(&self) -> bool {
        if self.is_version_code {
            (chrono::Utc::now().naive_utc() - self.added) <= Duration::days(1)
        } else {
            (chrono::Utc::now().naive_utc() - self.added) <= Duration::days(40)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuildInfo {
    pub(crate) id: GuildId,
    pub(crate) alert_role: Option<RoleId>,
    pub(crate) alert_channel: Option<ChannelId>,
    pub(crate) enabled: bool,
    pub(crate) last_code: u64,
}

#[derive(Debug)]
pub enum InvalidInfo {
    Channel(Option<ChannelId>),
    Role(RoleId),
    Both(Option<ChannelId>, RoleId),
}
