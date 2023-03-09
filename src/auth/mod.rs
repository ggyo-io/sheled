use serde::{Deserialize, Serialize};

pub mod md5;
pub mod jwt;
pub mod api;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCtx {
    id: i64,
    pub email: String,
    name: String,
    exp: usize,
}

#[cfg(test)]
mod tests {
    use super::UserCtx;
    use super::jwt;

    #[tokio::test]
    async fn auth_from_jwt() {

        let jwt = jwt::from_utx(&UserCtx {
            id: 17,
            email: String::from("someone@out.there"),
            name: String::from("Some One"),
            exp: std::u64::MAX as usize, // set exp claim to maximum value of usize
        }, jwt::MasterTokenSecret::default()).await;
        println!("{:?}", jwt);
    }

    #[tokio::test]
    async fn auth_to_utx() {


        let secret = jwt::MasterTokenSecret::default();
        let jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MTcsImVtYWlsIjoic29tZW9uZUBvdXQudGhlcmUiLCJuYW1lIjoiU29tZSBPbmUiLCJleHAiOjE4NDQ2NzQ0MDczNzA5NTUxNjE1fQ.PcJ07s5W97BnX1jJxtuoex0rgQzvqU4ENLGP290ihxE";

        let claim = jwt::to_utx(jwt, secret).await.unwrap();
        println!("{:?}", claim);
        assert_eq!(claim.email, String::from("someone@out.there"));
        assert_eq!(claim.name, String::from("Some One"));
        assert_eq!(claim.id, 17);
    }
}