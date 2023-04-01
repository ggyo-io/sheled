use serde::{Deserialize, Serialize};
use crate::auth::md5::hash_password;
use crate::model::db::Db;
use crate::model::users::{UserMac, UserPatch};
use super::{UserCtx, jwt};
use super::jwt::MasterTokenSecret;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthReply {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSignup {
    name: String,
    email: String,
    password: String,
}

fn token_reply(token: &str) -> Result<impl warp::Reply, warp::Rejection> {
    let response = UserAuthReply {
        token: token.to_owned(),
    };
    let reply_body = warp::reply::json(&response);
    let with_token = warp::reply::with_header(
        reply_body,
        "Set-Cookie",
        format!("token={}; HttpOnly", token),
    );

    Ok(with_token)
}

pub async fn signup(token_secret: MasterTokenSecret, db: Db, user: UserSignup) -> Result<impl warp::Reply, warp::Rejection> {
    println!("-<>-<>-<>- user_signup ${:?}", user);
    let hash = hash_password(&user.password);
    let new_user = UserPatch {
        name: Some(user.name),
        email: Some(user.email),
        hash: Some(hash),
    };

    let result = UserMac::create(&db, new_user.clone()).await?;
    println!("\n--> result {:?}", result);

    let claim = UserCtx {
        id: result.id,
        email: result.email,
        name: result.name,
        exp: std::u64::MAX as usize, // set exp claim to maximum value of usize
    };
    let token = jwt::from_utx(&claim, token_secret).await;
    println!("\n--> token {:?}", token);

    token_reply(&token)
}

pub async fn login(token_secret: MasterTokenSecret, db: Db, user: UserLogin) -> Result<impl warp::Reply, warp::Rejection> {
    println!("-<>-<>-<>- user_login ${:?}", user);
    let unauthorized_token = "unauthorized";

    let result = UserMac::get_by_email(&db, &user.email).await;
    if result.is_err() {
        return token_reply(unauthorized_token);
    }
    let result = result.unwrap();
    println!("\n--> result {:?}", result);

    let hash = hash_password(&user.password);
    if hash != result.hash {
        return token_reply(unauthorized_token);
    }

    let claim = UserCtx {
        id: result.id,
        email: result.email,
        name: result.name,
        exp: std::u64::MAX as usize, // set exp claim to maximum value of usize
    };
    let token = jwt::from_utx(&claim, token_secret).await;
    println!("\n--> token {:?}", token);

    token_reply(&token)
}
