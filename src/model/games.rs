use super::db::Db;
use crate::model;
use sqlb::HasFields;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Game {
    pub id: i64,
    pub pgn: String,
}

#[derive(sqlb::Fields, Default, Debug, Clone)]
pub struct GamePatch {
    pub pgn: Option<String>,
}

pub struct GameMac;

const TABLE: &str = "games";
const COLUMNS: & [&str] = &["id", "pgn"];

impl GameMac {
    pub async fn create(db: &Db, data: GamePatch) -> Result<Game, model::Error> {

        let sb = sqlb::insert()
            .table(TABLE)
            .data(data.fields())
            .returning(COLUMNS);
        let game = sb.fetch_one(db).await?;
        Ok(game)
    }

/*
    pub async fn get(db: &Db, id: i64) -> Result<Game, model::Error> {
        let sb = sqlb::select()
            .table(TABLE)
            .and_where_eq("id", id)
            .columns(COLUMNS);
        let game = sb.fetch_one(db).await?;
        Ok(game)
    }


    pub async fn update(db: &Db, id: i64, data:GamePatch) -> Result<Game, model::Error> {
        let sb = sqlb::update()
            .table(TABLE)
            .data(data.fields())
            .and_where_eq("id", id)
            .returning(COLUMNS);
        let game = sb.fetch_one(db).await?;
        Ok(game)
    }
    */

    pub async fn list(db: &Db) -> Result<Vec<Game>, model::Error> {

        let sb = sqlb::select().table(TABLE).columns(COLUMNS);
        let games = sb.fetch_all(db).await?;

        Ok(games)
    }

}

#[cfg(test)]
mod tests {
    use crate::model::db::init_db;
    use super::{GameMac, GamePatch};

/*

cargo watch -q -c -w src -x 'test model_game_ -- --nocapture --test-threads=1'

 */
    #[tokio::test]
    async fn model_game_create() -> Result<(), Box<dyn std::error::Error>> {
        let db = init_db().await?;

        let new_game = GamePatch {
            pgn: Some("1. f4".to_string()),
        };

        let result = GameMac::create(&db, new_game.clone()).await?;
        println!("\n--> result {:?}", result);

        assert_eq!(result.pgn, new_game.pgn.unwrap());
  
        Ok(())
    }    
    
   /* 
    #[tokio::test]
    async fn model_game_list() -> Result<(), Box<dyn std::error::Error>> {
        let db = init_db().await?;

        let result = GameMac::list(&db).await?;
        println!("\n--> result {:?}", result);

        assert_eq!(2, result.len(), "number of dev seed games");
        assert_eq!(100, result[0].id);
        assert_eq!("1. e4 d6", result[0].pgn);
        assert_eq!("1. d4 d5", result[1].pgn);
        Ok(())
    }
    */

}
