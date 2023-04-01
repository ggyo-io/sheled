use super::db::Db;
use crate::model;
use sqlb::HasFields;
use entity::Entity;

#[derive(sqlx::FromRow, Debug, Clone, Entity)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub hash: String,
}

fn table() -> &'static str {
    User::entity().entity
}

#[derive(sqlb::Fields, Default, Debug, Clone)]
pub struct UserPatch {
    pub name: Option<String>,
    pub email: Option<String>,
    pub hash: Option<String>,
}

pub struct UserMac;

impl UserMac {
    pub async fn create(db: &Db, data: UserPatch) -> Result<User, model::Error>{

        let sb = sqlb::insert()
            .table(table())
            .data(data.fields())
            .returning(&User::entity().columns);
        let user = sb.fetch_one(db).await?;
        Ok(user)
    }

/*
    pub async fn get(db: &Db, id: i64) -> Result<User, model::Error> {
        let sb = sqlb::select()
            .table(table())
            .and_where_eq("id", id)
            .columns(&User::entity().columns);
        let user = sb.fetch_one(db).await?;
        Ok(user)
    }
*/

    pub async fn get_by_email(db: &Db, email: &str) -> Result<User, model::Error> {
        let sb = sqlb::select()
            .table(table())
            .and_where_eq("email", email)
            .columns(&User::entity().columns);
        let user = sb.fetch_one(db).await?;
        Ok(user)
    }

/*
    pub async fn update(db: &Db, id: i64, data:UserPatch) -> Result<User, model::Error> {
        let sb = sqlb::update()
            .table(table())
            .data(data.fields())
            .and_where_eq("id", id)
            .returning(&User::entity().columns);
        let user = sb.fetch_one(db).await?;
        Ok(user)
    }


    pub async fn list(db: &Db) -> Result<Vec<User>, model::Error> {

        let sb = sqlb::select().table(table()).columns(&User::entity().columns);
        let users = sb.fetch_all(db).await?;

        Ok(users)
    }
*/
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, Rng};
    use crate::model::db::init_db;
    use super::{UserMac, UserPatch};
    use crate::auth::md5::hash_password;


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

        let new_user = UserPatch {
            name: Some(user.to_owned()),
            email: Some(email.to_owned()),
            hash: Some(hash_password("password")),
        };

        let result = UserMac::create(&db, new_user.clone()).await?;
        println!("\n--> result {:?}", result);

        assert_eq!(result.name, new_user.clone().name.unwrap());
        assert_eq!(result.email, new_user.clone().email.unwrap());

        // expected to fail due to duplicate email
        let new_err_user = UserPatch {
            name: Some(other_user.to_string()),
            email: Some(email.to_string()),
            hash: Some(hash_password("other password")),
        };
        let errresult = UserMac::create(&db, new_err_user.clone()).await;
        println!("\n--> errresult {:?}", errresult);

        /*
        let existing_user = UserMac::get(&db, result.id).await?;
        println!("\n--> existing_user {:?}", existing_user);
        assert_eq!(existing_user.name, new_user.name.unwrap());
        assert_eq!(existing_user.email, new_user.email.unwrap());
        */
        Ok(())
    }

}
