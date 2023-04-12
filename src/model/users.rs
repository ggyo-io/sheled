use super::db::Db;
use crate::model;
use sea_orm::entity::prelude::*;
use sea_orm::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: model::IdType,
    pub name: String,
    #[sea_orm(unique, indexed)]
    pub email: String,
    pub hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct UserMac;

impl UserMac {
    pub async fn create(
        db: &Db,
        name: &str,
        email: &str,
        hash: &str,
    ) -> Result<model::IdType, model::Error> {
        let user = ActiveModel {
            name: Set(name.to_owned()),
            email: Set(email.to_owned()),
            hash: Set(hash.to_owned()),
            ..Default::default()
        };
        let res = Entity::insert(user).exec(db).await?;

        Ok(res.last_insert_id)
    }

    pub async fn get_by_email(db: &Db, email: &str) -> Result<Option<Model>, model::Error> {
        let user = Entity::find()
            .filter(Column::Email.contains(email))
            .one(db)
            .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::UserMac;
    use crate::auth::md5::hash_password;
    use crate::model::db::init_db;
    use rand::{distributions::Alphanumeric, Rng};

    /*

    cargo watch -q -c -w src -x 'test model_user_ -- --nocapture --test-threads=1'

     */
    #[tokio::test]
    async fn model_user_create() -> Result<(), Box<dyn std::error::Error>> {
        let db = init_db().await?;
        let user = "Some User";
        let other_user = "Some Other User";
        let email: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let hash = hash_password("password");

        let result = UserMac::create(&db, user, &email, &hash).await?;
        println!("\n--> result {:?}", result);
        assert!(result > 0);

        // expected to fail due to duplicate email
        let other_hash = hash_password("other password");
        let errresult = UserMac::create(&db, other_user, &email, &other_hash).await;
        println!("\n--> errresult {:?}", errresult);
        assert!(errresult.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn model_user_model() -> Result<(), Box<dyn std::error::Error>> {
        use super::*;
        let schema = Schema::new(DbBackend::Postgres);
        let st = DbBackend::Postgres.build(schema.create_table_from_entity(Entity).if_not_exists());

        println!("sql {:?}", st);

        Ok(())
    }
}
