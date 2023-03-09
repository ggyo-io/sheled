CREATE TABLE IF NOT EXISTS  games (
    id  BIGSERIAL PRIMARY KEY,
    ctime TIMESTAMP without time zone default (now() at time zone('utc')),
    pgn VARCHAR
);

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    ctime TIMESTAMP without time zone default (now() at time zone('utc')),
    name VARCHAR NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    hash VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS  keys (
    id  BIGSERIAL PRIMARY KEY,
    ctime TIMESTAMP without time zone default (now() at time zone('utc')),
    key VARCHAR
);