use std::sync::Arc;

use anyhow::{anyhow, Result};
use libsql::{params, Connection, Row, ValueType};
use serde::{Deserialize, Serialize};
use serenity::all::{
    ChannelId, Context, CreateMessage, Guild, GuildChannel, GuildId, PartialGuild, RoleId,
    UnavailableGuild,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TursoGuild {
    pub id: i64,
    pub guild_id: GuildId,
    pub enabled: i64,
    pub last_code: i64,
    pub alert_channel: Option<ChannelId>,
    pub alert_role: Option<RoleId>,
}

impl TursoGuild {
    pub fn from_row(row: Row) -> Result<Self> {
        let id: i64;
        let guild_id: GuildId;
        let enabled: i64;
        let last_code: i64;
        let alert_channel: Option<ChannelId>;
        let alert_role: Option<RoleId>;

        if let Some("id") = row.column_name(0) {
            if let Ok(ValueType::Integer) = row.column_type(0) {
                id = row.get(0)?;
            } else {
                return Err(anyhow!(
                    "Expected field 0 to be of type Integer. Was {:?}",
                    row.column_type(0)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 0 to be named 'id'. Was {:?}",
                row.column_name(0)
            ));
        }

        if let Some("guild_id") = row.column_name(1) {
            if let Ok(ValueType::Integer) = row.column_type(1) {
                guild_id = GuildId::new(
                    row.get::<i64>(1)?
                        .try_into()
                        .expect("Guild Id should never positive"),
                );
            } else {
                return Err(anyhow!(
                    "Expected field 1 to be of type Integer. Was {:?}",
                    row.column_type(1)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 1 to be named 'guild_id'. Was {:?}",
                row.column_name(1)
            ));
        }

        if let Some("enabled") = row.column_name(2) {
            if let Ok(ValueType::Integer) = row.column_type(2) {
                enabled = row.get(2)?;
            } else {
                return Err(anyhow!(
                    "Expected field 2 to be of type Integer. Was {:?}",
                    row.column_type(2)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 2 to be named 'enabled'. Was {:?}",
                row.column_name(2)
            ));
        }

        if let Some("last_code") = row.column_name(3) {
            if let Ok(ValueType::Integer) = row.column_type(3) {
                last_code = row.get(3)?;
            } else {
                return Err(anyhow!(
                    "Expected field 3 to be of type Integer. Was {:?}",
                    row.column_type(3)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 3 to be named 'last_code'. Was {:?}",
                row.column_name(3)
            ));
        }

        if let Some("alert_channel") = row.column_name(4) {
            if let Ok(ValueType::Integer) = row.column_type(4) {
                alert_channel = Some(ChannelId::new(
                    row.get::<i64>(4)?
                        .try_into()
                        .expect("Channel Id should be positive"),
                ));
            } else if let Ok(ValueType::Null) = row.column_type(4) {
                alert_channel = None;
            } else {
                return Err(anyhow!(
                    "Expected field 4 to be of type Integer. Was {:?}",
                    row.column_type(4)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 4 to be named 'alert_channel'. Was {:?}",
                row.column_name(4)
            ));
        }

        if let Some("alert_role") = row.column_name(5) {
            if let Ok(ValueType::Integer) = row.column_type(5) {
                alert_role = Some(RoleId::new(
                    row.get::<i64>(5)?
                        .try_into()
                        .expect("Role Id should be positive"),
                ));
            } else if let Ok(ValueType::Null) = row.column_type(5) {
                alert_role = None;
            } else {
                return Err(anyhow!(
                    "Expected field 5 to be of type Integer or Null. Was {:?}",
                    row.column_type(5)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 5 to be named 'alert_role'. Was {:?}",
                row.column_name(5)
            ));
        }

        Ok(Self {
            id,
            guild_id,
            enabled,
            last_code,
            alert_channel,
            alert_role,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TursoCode {
    pub id: i64,
    pub code: String,
    pub valid: i64,
}

impl TursoCode {
    pub fn from_row(row: Row) -> Result<Self> {
        let id: i64;
        let code: String;
        let valid: i64;

        if let Some("id") = row.column_name(0) {
            if let Ok(ValueType::Integer) = row.column_type(0) {
                id = row.get(0)?;
            } else {
                return Err(anyhow!(
                    "Expected field 0 to be of type Integer. Was {:?}",
                    row.column_type(0)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 0 to be named 'id'. Was {:?}",
                row.column_name(0)
            ));
        }

        if let Some("code") = row.column_name(1) {
            if let Ok(ValueType::Text) = row.column_type(1) {
                code = row.get(1)?;
            } else {
                return Err(anyhow!(
                    "Expected field 1 to be of type Text. Was {:?}",
                    row.column_type(1)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 1 to be named 'code'. Was {:?}",
                row.column_name(1)
            ));
        }

        if let Some("valid") = row.column_name(2) {
            if let Ok(ValueType::Integer) = row.column_type(2) {
                valid = row.get(2)?;
            } else {
                return Err(anyhow!(
                    "Expected field 2 to be of type Integer. Was {:?}",
                    row.column_type(2)
                ));
            }
        } else {
            return Err(anyhow!(
                "Expected field 2 to be named 'valid'. Was {:?}",
                row.column_name(2)
            ));
        }

        Ok(Self { id, code, valid })
    }
}

pub struct TursoDb {
    client: Arc<Connection>,
}

impl TursoDb {
    async fn guilds(&self) -> Result<Vec<TursoGuild>> {
        let mut rows = self.client.query("SELECT * FROM guilds;", ()).await?;
        let mut guilds = Vec::new();
        while let Some(row) = rows.next()? {
            guilds.push(TursoGuild::from_row(row)?);
        }
        Ok(guilds)
    }

    pub async fn new(client: Arc<Connection>) -> Result<Self> {
        Ok(Self {
            client: client.clone(),
        })
    }

    async fn invalidate_codes(&self, new_codes: &Vec<String>) -> Result<()> {
        let code_str = new_codes
            .iter()
            .map(|code| format!("'{code}'"))
            .collect::<Vec<_>>()
            .join(",");
        let q = format!("UPDATE codes SET valid = 0 WHERE code NOT IN ({code_str})");
        self.client.execute(q.as_str(), ()).await?;
        Ok(())
    }

    pub async fn diff_guild_codes(
        &self,
        new_codes: &Vec<String>,
    ) -> Result<HashMap<GuildId, (TursoGuild, Vec<TursoCode>)>> {
        for code in new_codes {
            if let Err(err) = self
                .client
                .execute(
                    "INSERT INTO codes (id, code, valid) VALUES (NULL, ?1, 1);",
                    [code.as_str()],
                )
                .await
            {
                match err {
                    libsql::Error::SqliteFailure(err_code, _) => {
                        if err_code != 2067 {
                            return Err(err.into());
                        } else {
                            warn!(code, "Code already encountered, skipping");
                        }
                    }
                    _ => return Err(err.into()),
                }
            }
        }
        self.invalidate_codes(new_codes).await?;
        let mut new_codes = HashMap::new();
        for guild in self.guilds().await? {
            let mut rows = self.client.query("SELECT * FROM codes WHERE id > (SELECT last_code FROM guilds WHERE guild_id = ?1) AND valid = 1", [i64::from(guild.guild_id)]).await?;
            let mut codes = Vec::new();
            while let Some(row) = rows.next()? {
                codes.push(TursoCode::from_row(row)?);
            }
            let last_inserted = codes
                .iter()
                .max_by(|x, y| x.id.cmp(&y.id))
                .map_or(0, |code| code.id);
            new_codes.insert(guild.guild_id, (guild.clone(), codes));
            let res = self
                .client
                .execute(
                    "UPDATE guilds SET last_code = ?1 WHERE guild_id = ?2",
                    params![last_inserted, i64::from(guild.guild_id)],
                )
                .await?;
            if res != 1 {
                return Err(anyhow!(
                    "Could not update last_code. Affected rows: {}",
                    res
                ));
            }
        }

        Ok(new_codes)
    }

    pub async fn set_guild_state(&self, guild: GuildId, enabled: bool) -> Result<()> {
        let res = self
            .client
            .execute(
                "UPDATE guilds SET enabled = ?1 WHERE guild_id = ?2;",
                [enabled as i64, i64::from(guild)],
            )
            .await?;
        if res != 1 {
            return Err(anyhow!("Update did not succeed. Affected rows: {}", res));
        }
        Ok(())
    }

    pub async fn guild_alert_role(&self, guild: GuildId) -> Result<Option<RoleId>> {
        let guilds = self.guilds().await?;
        let guild_info = guilds.iter().find(|g| g.guild_id == guild);
        return if let Some(g) = guild_info {
            Ok(g.alert_role)
        } else {
            Ok(None)
        };
    }

    pub async fn set_guild_alert_role(&self, guild: GuildId, role: Option<RoleId>) -> Result<()> {
        let res = self
            .client
            .execute(
                "UPDATE guilds SET alert_role = ?1 WHERE guild_id = ?2",
                params![role.map(|id| i64::from(id)), i64::from(guild)],
            )
            .await?;
        if res != 1 {
            return Err(anyhow!("Update did not succeed. Affected rows: {}", res));
        }
        Ok(())
    }

    pub async fn guild_alert_channel(&self, guild: GuildId) -> Result<Option<ChannelId>> {
        let guilds = self.guilds().await?;
        let guild_info = guilds.iter().find(|g| g.guild_id == guild);
        return if let Some(g) = guild_info {
            Ok(g.alert_channel)
        } else {
            Ok(None)
        };
    }

    pub async fn set_guild_alert_channel(
        &self,
        guild: GuildId,
        channel: Option<ChannelId>,
    ) -> Result<()> {
        let res = self
            .client
            .execute(
                "UPDATE guilds SET alert_channel = ?1 WHERE guild_id = ?2",
                params![channel.map(|id| i64::from(id)), i64::from(guild)],
            )
            .await?;
        if res != 1 {
            return Err(anyhow!("Update did not succeed. Affected rows: {}", res));
        }
        Ok(())
    }

    pub async fn update_guilds(&self, guilds: &Vec<UnavailableGuild>) -> Result<()> {
        for guild in guilds {
            let _ = self.try_add_guild(guild.id).await?;
        }

        Ok(())
    }

    pub async fn try_add_guild(&self, guild: GuildId) -> Result<bool> {
        let guilds = self.guilds().await?;
        if guilds.iter().find(|info| info.guild_id == guild).is_none() {
            warn!(guild=?&guild, "New guild joined. Adding to config");
            let res = self
                .client
                .execute(
                    "INSERT INTO guilds (id, guild_id) VALUES (NULL, ?1);",
                    [i64::from(guild)],
                )
                .await?;
            if res != 1 {
                return Err(anyhow!("Insert did not succeed. Affected rows: {}", res));
            }
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn validate_info(&self, ctx: &Context) -> Result<Vec<(GuildId, InvalidInfo)>> {
        let mut invalid_guilds: Vec<(GuildId, InvalidInfo)> = vec![];
        for guild in self.guilds().await?.iter() {
            let g = Self::get_guild(&guild.guild_id, &ctx).await?;
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
                    guild.guild_id,
                    InvalidInfo::Both(guild.alert_channel, guild.alert_role.unwrap()),
                ))
            }
            if !channel_valid {
                invalid_guilds.push((guild.guild_id, InvalidInfo::Channel(guild.alert_channel)))
            }
            if !role_valid {
                invalid_guilds.push((guild.guild_id, InvalidInfo::Role(guild.alert_role.unwrap())))
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

#[derive(Debug)]
pub enum InvalidInfo {
    Channel(Option<ChannelId>),
    Role(RoleId),
    Both(Option<ChannelId>, RoleId),
}
