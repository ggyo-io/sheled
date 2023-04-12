use super::db::Db;
use crate::model;
use sea_orm::entity::prelude::*;
use sea_orm::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: model::IdType,
    pub key: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct KeyMac;

impl KeyMac {
    pub async fn create(db: &Db, data: &[u8]) -> Result<model::IdType, model::Error> {
        let key = ActiveModel {
            key: Set(data.to_owned()),
            ..Default::default()
        };
        let res = Entity::insert(key).exec(db).await?;

        Ok(res.last_insert_id)
    }

    pub async fn get_last(db: &Db) -> Result<Option<Model>, model::Error> {
        let k = Entity::find().order_by_desc(Column::Id).one(db).await?;
        Ok(k)
    }
}

#[cfg(test)]
mod tests {
    use super::KeyMac;
    use crate::auth::jwt::{current_key, random_key};
    use crate::model::db::init_db;
    /*
    cargo watch -q -c -w src -x 'test model_key_ -- --nocapture --test-threads=1'
     */

    #[tokio::test]
    async fn model_key_create() -> Result<(), Box<dyn std::error::Error>> {
        let db = init_db().await?;
        let result = current_key(&db).await?;
        println!("\n--> current token {:?}", result);

        let key0 = random_key();
        let key1 = random_key();

        let result = KeyMac::create(&db, &key0).await?;
        println!("\n--> result {:?}", result);
        let result = KeyMac::create(&db, &key1).await?;
        println!("\n--> result {:?}", result);
        let last_key = KeyMac::get_last(&db).await?;
        println!("\n--> last_key {:?}", last_key);

        assert_eq!(last_key.unwrap().key, key1);

        Ok(())
    }

    #[tokio::test]
    async fn model_key_model() -> Result<(), Box<dyn std::error::Error>> {
        use super::*;
        let schema = Schema::new(DbBackend::Postgres);
        let st = DbBackend::Postgres.build(schema.create_table_from_entity(Entity).if_not_exists());

        println!("sql {:?}", st);

        Ok(())
    }
}
