use super::UserCtx;
use crate::model::db::Db;
use crate::model::keys::KeyMac;
use crate::model::Error as ModelError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use std::sync::Arc;
use thiserror::Error as ThisError;
use tokio::sync::RwLock;
use warp::Rejection;

/// A JWT token secret.
#[derive(Default, Debug, Clone)]
pub struct TokenSecret(pub [u8; 32]);

impl From<Vec<u8>> for TokenSecret {
    fn from(bytes: Vec<u8>) -> Self {
        let mut secret = [0; 32];
        secret.copy_from_slice(&bytes[..32]);
        TokenSecret(secret)
    }
}

pub type MasterTokenSecret = Arc<RwLock<TokenSecret>>;

async fn parse_jwt(
    jwt: &str,
    secret: MasterTokenSecret,
) -> Result<UserCtx, jsonwebtoken::errors::Error> {
    let s = secret.read().await;
    println!("parse_jwt secret {:?} jwt {}", s, jwt);
    let decoding_key = DecodingKey::from_secret(&s.0);

    decode::<UserCtx>(jwt, &decoding_key, &Validation::new(Algorithm::HS256))
        .map(|data| data.claims)
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    FromModelError(#[from] ModelError),
}

pub async fn to_utx(token: &str, secret: MasterTokenSecret) -> Result<UserCtx, Rejection> {
    match parse_jwt(token, secret).await {
        Ok(utx) => {
            println!("--> with_utx token: {} utx {:?}", token, &utx);
            Ok(utx)
        }
        Err(_ex) => {
            println!("--> with_utx invalid token: {}", token);
            Err(warp::reject::not_found())
        }
    }
}

pub async fn from_utx(claim: &UserCtx, secret: MasterTokenSecret) -> String {
    let s = secret.read().await.to_owned();
    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(&s.0);
    encode(&header, &claim, &encoding_key).unwrap()
}

pub fn random_key() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}

pub async fn current_key(db: &Db) -> Result<TokenSecret, Error> {
    match KeyMac::get_last(db).await {
        Ok(Some(k)) => {
            let token: TokenSecret = k.key.into();
            println!("\n--> found old key {:?}", token);
            Ok(token)
        }
        _ => {
            let new_key = random_key();
            let result = KeyMac::create(db, &new_key).await?;
            println!("\n--> create new key {:?} {:?}", new_key, result);
            let token = TokenSecret(new_key);

            Ok(token)
        }
    }
}
