use thiserror::Error as ThisError;
use warp::reject::Reject;
use hex::FromHexError;

pub mod db;
pub mod games;
pub mod users;
pub mod keys;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    HexDecode(#[from] FromHexError),
}

// error[E0277]: the trait bound `model::Error: warp::reject::Reject` is not satisfied
impl Reject for Error {}