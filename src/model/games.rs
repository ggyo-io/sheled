use super::db::Db;
use crate::model;
use sea_orm::entity::prelude::*;
use sea_orm::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: model::IdType,
    pub pgn: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[allow(dead_code)]
pub struct GameMac;

#[allow(dead_code)]
impl GameMac {
    pub async fn create(db: &Db, data: &str) -> Result<model::IdType, model::Error> {
        let game = ActiveModel {
            pgn: Set(data.to_owned()),
            ..Default::default()
        };
        let res = Entity::insert(game).exec(db).await?;

        Ok(res.last_insert_id)
    }

    pub async fn get(db: &Db, id: model::IdType) -> Result<Option<Model>, model::Error> {
        Ok(Entity::find_by_id(id).one(db).await?)
    }

    pub async fn list(db: &Db) -> Result<Vec<Model>, model::Error> {
        Ok(Entity::find().all(db).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::GameMac;
    use crate::model::db::init_db;

    /*

    cargo watch -q -c -w src -x 'test model_game_ -- --nocapture --test-threads=1'

     */
    #[tokio::test]
    async fn model_game_create() -> Result<(), Box<dyn std::error::Error>> {
        let db = init_db().await?;

        let pgn = "1. f4";

        let id = GameMac::create(&db, pgn).await?;
        println!("\n--> id {:?}", id);
        let game = GameMac::get(&db, id).await.unwrap();
        assert!(game.is_some());
        assert_eq!(game.unwrap().pgn, pgn);

        Ok(())
    }
}
