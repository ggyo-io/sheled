use hex::FromHexError;
use thiserror::Error as ThisError;
use warp::reject::Reject;

pub mod db;
pub mod games;
pub mod keys;
pub mod users;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    DB(#[from] sea_orm::DbErr),

    #[error(transparent)]
    HexDecode(#[from] FromHexError),
}

// error[E0277]: the trait bound `model::Error: warp::reject::Reject` is not satisfied
impl Reject for Error {}
