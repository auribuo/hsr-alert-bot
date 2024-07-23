CREATE TABLE IF NOT EXISTS guilds (
    id integer primary key autoincrement,
    guild_id text unique not null,
    enabled integer not null default 1,
    last_code int not null default 0,
    alert_channel text null default null,
    alert_role text null default null
);
