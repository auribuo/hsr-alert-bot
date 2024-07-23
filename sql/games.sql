CREATE TABLE IF NOT EXISTS games (
    id integer primary key autoincrement,
    name varchar(50) not null unique
);
