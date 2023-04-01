use super::db::Db;
use crate::model;
use sqlb::HasFields;
use entity::{Entity, EM};

#[derive(sqlx::FromRow, Debug, Clone, Entity)]
pub struct Key {
    pub id: i64,
    pub key: String,
}

#[derive(sqlb::Fields, Default, Debug, Clone)]
pub struct KeyPatch {
    pub key: Option<String>,
}

pub struct KeyMac;

impl KeyMac {
    pub async fn create(db: &Db, data: &KeyPatch) -> Result<Key, model::Error> {

        let sb = sqlb::insert()
            .table(Key::entity())
            .data(data.fields())
            .returning(Key::columns());
        let game = sb.fetch_one(db).await?;
        Ok(game)
    }


    pub async fn get_last(db: &Db) -> Result<Key, model::Error> {
        let sb = sqlb::select()
            .table(Key::entity())
            .order_by("!id")
            .columns(Key::columns());
        let key = sb.fetch_one(db).await?;
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::db::init_db;
    use super::{KeyMac, KeyPatch};
    use crate::auth::jwt::{random_key, current_key};
    use entity::EM;
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

        let key0 = KeyPatch {
            key: Some(key0.to_owned()),
        };
        let key1 = KeyPatch {
            key: Some(key1.to_owned()),
        };

        let result = KeyMac::create(&db, &key0).await?;
        println!("\n--> result {:?}", result);
        let result = KeyMac::create(&db, &key1).await?;
        println!("\n--> result {:?}", result);
        let last_key = KeyMac::get_last(&db).await?;
        println!("\n--> last_key {:?}", last_key);

        assert_eq!(last_key.key, key1.key.unwrap());
  
        Ok(())
    }    

    #[test]
    fn model_key_entity() {
        use crate::model::keys::Key;

        assert_eq!(Key::entity(), "keys");
        assert_eq!(Key::columns(), &["id", "key"]);
        assert_eq!(Key::tys(), &["i64", "String"]);
    }

}
