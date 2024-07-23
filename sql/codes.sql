CREATE TABLE IF NOT EXISTS codes (
    id integer primary key autoincrement,
    code varchar(50) not null unique,
    valid integer not null
);
