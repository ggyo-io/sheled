use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::PathBuf;

pub type Db = Pool<Postgres>;

const PG_HOST: &str = "localhost";
const PG_DB: &str = "postgres";
const PG_USER: &str = "postgres";
const PG_PASS: &str = "postgres";

const PG_APP_DB: &str = "sheled";
const PG_APP_USER: &str = "sheled";
const PG_APP_PASS: &str = "pwd_to_change";
const PG_APP_MAX_CON: u32 = 5;

const SQL_DIR: &str = "sql/";

async fn create_database_role(pool: &Db) -> Result<(), sqlx::Error> {
    // start a transaction
    let mut transaction = pool.begin().await?;

    // create the owner role if it doesn't exist
    let role_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname = $1)")
            .bind(PG_APP_USER)
            .fetch_one(&mut transaction)
            .await?;

    if !role_exists {
        // create role with password
        sqlx::query(&format!(
            "CREATE ROLE {} LOGIN PASSWORD '{}'",
            PG_APP_USER, PG_APP_PASS
        ))
        .execute(&mut transaction)
        .await?;
        println!("Role {} created successfully", PG_APP_USER);
    } else {
        println!("Role {} already exists", PG_APP_USER);
    }

    // check if application database exists
    let database_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(PG_APP_DB)
            .fetch_one(&mut transaction)
            .await?;

    // commit the transaction
    transaction.commit().await?;

    if !database_exists {
        // create application database with the specified owner role
        // ignore create database error since not protected by transaction
        match sqlx::query(&format!(
            "CREATE DATABASE {} WITH OWNER = {}",
            PG_APP_DB, PG_APP_USER
        ))
        .execute(pool)
        .await
        {
            Ok(_) => println!(
                "Database {} created successfully with owner role {}",
                PG_APP_DB, PG_APP_USER
            ),
            Err(e) => println!(
                "Error creating database: {} database {} owner role {}",
                e, PG_APP_DB, PG_APP_USER
            ),
        }
    } else {
        println!("Database {} already exists", PG_APP_DB);
    }

    Ok(())
}

async fn new_db_pool(
    host: &str,
    db: &str,
    user: &str,
    pass: &str,
    max_con: u32,
) -> Result<Db, sqlx::Error> {
    let con_string = format!("postgres://{}:{}@{}/{}", user, pass, host, db);
    PgPoolOptions::new()
        .max_connections(max_con)
        .connect(&con_string)
        .await
}

pub async fn init_db() -> Result<Db, sqlx::Error> {
    {
        let root_db = new_db_pool(PG_HOST, PG_DB, PG_USER, PG_PASS, 1).await?;
        create_database_role(&root_db).await?;
    }

    let app_db = new_db_pool(PG_HOST, PG_APP_DB, PG_APP_USER, PG_APP_PASS, 1).await?;
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    for path in paths {
        if let Some(path) = path.to_str() {
            if path.ends_with(".sql") {
                pexec(&app_db, path).await?
            }
        }
    }

    new_db_pool(PG_HOST, PG_APP_DB, PG_APP_USER, PG_APP_PASS, PG_APP_MAX_CON).await
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    let content = fs::read_to_string(file).map_err(|ex| {
        println!("Error reading {}, {}", file, ex);
        ex
    })?;

    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        match sqlx::query(sql).execute(db).await {
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

    async fn table_exists(db: &Db, name: &str) -> bool {
        let query = format!(
            "SELECT EXISTS (
            SELECT FROM
                pg_tables
            WHERE
                schemaname = 'public' AND
                tablename  = '{name}'
            );"
        );
        let result: bool = sqlx::query_scalar(&query).fetch_one(db).await.unwrap();

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

    #[tokio::test]
    async fn model_db_create_db_role() -> Result<(), Box<dyn std::error::Error>> {
        use crate::model::db::*;

        let root_db = new_db_pool(PG_HOST, PG_DB, PG_USER, PG_PASS, 1).await?;
        create_database_role(&root_db).await?;

        let app_db =
            new_db_pool(PG_HOST, PG_APP_DB, PG_APP_USER, PG_APP_PASS, PG_APP_MAX_CON).await?;

        let two: i32 = sqlx::query_scalar("SELECT 1 + 1;")
            .fetch_one(&app_db)
            .await?;

        println!("Result of SELECT 1 + 1: {:?}", two);
        assert_eq!(two, 2);

        Ok(())
    }
}
