 ![](../../frontend-auth/public/favicon.ico)
# Postgres server
Simple way to start the Db server required by backend server.

```sh
➜  docker run --rm -p 5432:5432 -e "POSTGRES_PASSWORD=postgres" --name pg postgres:15
```


# Misc DB
```sql
DROP DATABASE IF EXISTS sheled;
DROP USER IF EXISTS sheled;
```


```sql
DROP USER IF EXISTS sheled;
CREATE USER sheled PASSWORD 'pwd_to_change';
CREATE DATABASE IF NOT EXISTS sheled OWNER sheled ENCODING 'UTF-8';
```

```sql
INSERT INTO games ( id, pgn) VALUES ( 100, '1. e4 d6');
INSERT INTO games ( id, pgn) VALUES ( 101, '1. d4 d5');
```