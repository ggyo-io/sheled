use crate::model::*;
use sea_orm::entity::prelude::*;
use sea_orm::*;

pub type Db = DatabaseConnection; // use sea orm

const PG_HOST: &str = "localhost";
const PG_DB: &str = "postgres";
const PG_USER: &str = "postgres";
const PG_PASS: &str = "postgres";

const PG_APP_DB: &str = "sheled";
const PG_APP_USER: &str = "sheled";
const PG_APP_PASS: &str = "pwd_to_change";
const PG_APP_MAX_CON: u32 = 5;

async fn create_database_role(db: &Db) -> Result<(), Error> {
    let pool = db.get_postgres_connection_pool();
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
            "CREATE DATABASE {} WITH OWNER = {} ENCODING 'UTF-8'",
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

fn db_url(host: &str, db: &str, user: &str, pass: &str) -> String {
    format!("postgres://{}:{}@{}/{}", user, pass, host, db)
}

async fn new_db_connection(
    host: &str,
    db: &str,
    user: &str,
    pass: &str,
    max_con: u32,
) -> Result<DbConn, DbErr> {
    let url = db_url(host, db, user, pass);
    let mut opts = ConnectOptions::new(url);
    opts.max_connections(max_con);

    Database::connect(opts).await
}

pub async fn init_tables(db: &DbConn) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    let tables = [
        builder.build(
            schema
                .create_table_from_entity(keys::Entity)
                .if_not_exists(),
        ),
        builder.build(
            schema
                .create_table_from_entity(games::Entity)
                .if_not_exists(),
        ),
        builder.build(
            schema
                .create_table_from_entity(users::Entity)
                .if_not_exists(),
        ),
    ];
    for t in tables {
        db.execute(t).await.unwrap();
    }
    Ok(())
}

pub async fn init_db() -> Result<DbConn, Error> {
    {
        let root_db = new_db_connection(PG_HOST, PG_DB, PG_USER, PG_PASS, 1).await?;
        create_database_role(&root_db).await?;
    }

    let db = new_db_connection(PG_HOST, PG_DB, PG_USER, PG_PASS, PG_APP_MAX_CON).await?;

    init_tables(&db).await?;

    Ok(db)
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
        let pool = db.get_postgres_connection_pool();
        let result: bool = sqlx::query_scalar(&query).fetch_one(pool).await.unwrap();
        println!("---> '{name}' table exists {result}");
        result
    }

    #[tokio::test]
    async fn model_db_init_db() -> Result<(), Box<dyn std::error::Error>> {
        let db = init_db().await?;

        assert!(table_exists(&db, "keys").await);
        assert!(table_exists(&db, "games").await);
        assert!(table_exists(&db, "users").await);
        assert!(!table_exists(&db, "lusers").await);

        Ok(())
    }

    #[tokio::test]
    async fn model_db_create_db_role() -> Result<(), Box<dyn std::error::Error>> {
        use crate::model::db::*;

        let root_db = new_db_connection(PG_HOST, PG_DB, PG_USER, PG_PASS, 1).await?;
        create_database_role(&root_db).await?;

        let app_db =
            new_db_connection(PG_HOST, PG_APP_DB, PG_APP_USER, PG_APP_PASS, PG_APP_MAX_CON).await?;
        let pool = app_db.get_postgres_connection_pool();

        let two: i32 = sqlx::query_scalar("SELECT 1 + 1;").fetch_one(pool).await?;

        println!("Result of SELECT 1 + 1: {:?}", two);
        assert_eq!(two, 2);

        Ok(())
    }
}
