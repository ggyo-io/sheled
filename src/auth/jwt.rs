use super::UserCtx;
use crate::model::db::Db;
use crate::model::keys::KeyMac;
use crate::model::Error as ModelError;
use core::convert;
use core::fmt;
use core::ops;
use hex::FromHexError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use std::sync::Arc;
use thiserror::Error as ThisError;
use tokio::sync::RwLock;
use warp::Rejection;

/// A JWT token secret.
#[derive(Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct TokenSecret(pub [u8; 32]);

impl From<Vec<u8>> for TokenSecret {
    fn from(bytes: Vec<u8>) -> Self {
        let mut secret = [0; 32];
        secret.copy_from_slice(&bytes[..32]);
        TokenSecret(secret)
    }
}

impl convert::From<TokenSecret> for [u8; 32] {
    #[inline]
    fn from(token_secret: TokenSecret) -> Self {
        token_secret.0
    }
}

impl fmt::Debug for TokenSecret {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(self, formatter)
    }
}

impl ops::Deref for TokenSecret {
    type Target = [u8; 32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for TokenSecret {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

macro_rules! implement {
    ($kind:ident, $format:expr) => {
        impl fmt::$kind for TokenSecret {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                for value in &self.0 {
                    write!(formatter, $format, value)?;
                }
                Ok(())
            }
        }
    };
}

implement!(LowerHex, "{:02x}");
implement!(UpperHex, "{:02X}");

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
    //    #[error("Invalid token {0} error {1}")]
    //    InvalidToken(String, jsonwebtoken::errors::Error),
    //
    #[error(transparent)]
    HexDecodeError(#[from] FromHexError),

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

pub fn random_key() -> String {
    let k = rand::thread_rng().gen::<[u8; 32]>();
    let k = TokenSecret(k);
    format!("{:x}", &k)
}

pub async fn current_key(db: &Db) -> Result<TokenSecret, Error> {
    match KeyMac::get_last(db).await {
        Ok(Some(k)) => {
            let token = hex::decode(&k.key)?.into();
            println!("\n--> found old key {:?}", k);
            Ok(token)
        }
        _ => {
            let new_key = random_key();
            let result = KeyMac::create(db, &new_key).await?;
            println!("\n--> create new key {} {:?}", new_key, result);
            let token = hex::decode(new_key)?.into();

            Ok(token)
        }
    }
}
