use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use std::fs;

pub type Db = Pool<Postgres>;

const PG_HOST: &str = "localhost";
const PG_DB: &str = "postgres";
const PG_USER: &str = "postgres";
const PG_PASS: &str = "postgres";

//const PG_APP_DB: &str = "sheled";
//const PG_APP_USER: &str = "sheled";
//const PG_APP_PASS: &str = "pwd_to_change";
const PG_APP_MAX_CON: u32 = 5;

const SQL_DIR: &str = "sql/";
//const SQL_RECREATE: &str = "sql/00-dbrecreate.sql";

async fn new_db_pool(host: &str, db: &str, user: &str, pass: &str, max_con: u32) -> Result<Db, sqlx::Error> {
    let con_string = format!("postgres://{}:{}@{}/{}", user, pass, host, db);
    PgPoolOptions::new()
    .max_connections(max_con)
    .connect(&con_string)
    .await
}

pub async fn init_db() -> Result<Db, sqlx::Error> {
    /* 
    {
        let root_db = new_db_pool(PG_HOST, PG_DB, PG_USER, PG_PASS, 1).await?;
        pexec(&root_db, SQL_RECREATE).await?;
    }
    */

    let app_db = new_db_pool(PG_HOST, PG_DB, PG_USER, PG_PASS, 1).await?;
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .into_iter()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    for path in paths {
        if let Some(path) = path.to_str() {
            if path.ends_with(".sql") /* &&  path != SQL_RECREATE */ {
                pexec(&app_db, &path).await?
            }
        }
    }

    new_db_pool(PG_HOST, PG_DB, PG_USER, PG_PASS, PG_APP_MAX_CON).await
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    let content = fs::read_to_string(file).map_err(|ex| {
        println!("Error reading {}, {}", file, ex);
        ex
    })?;

    let sqls: Vec<&str> = content.split(";").collect();

    for sql in sqls {
        match sqlx::query(&sql).execute(db).await {
            Ok(_) => (),
            Err(ex) => {
                println!("Error executing sql {}, {}", sql, ex);
                return Err(ex);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{init_db, Db};
    use sqlx::Row;

    async fn table_exists(db: &Db, name: &str) -> bool {
        let query = format!("SELECT EXISTS (
            SELECT FROM
                pg_tables
            WHERE
                schemaname = 'public' AND
                tablename  = '{name}'
            );");

        let result = sqlx::query(&query)
                .fetch_all(db)
                .await.unwrap();
        assert_eq!(1, result.len());
        assert_eq!(1, result[0].len());
        let result = result[0].get::<bool, usize>(0);
        println!("---> '{name}' table exists {result}");
        result
    }

    #[tokio::test]
    async fn model_db_init_db() -> Result<(), Box<dyn std::error::Error>> {
        let db = init_db().await?;

        assert!(table_exists(&db, "games").await);
        assert!(table_exists(&db, "users").await);
        assert!(!table_exists(&db, "lusers").await);

        Ok(())
    }
}