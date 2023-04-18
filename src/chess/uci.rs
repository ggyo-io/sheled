use shakmaty::{uci::*, *};
use std::str::FromStr;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseUci(#[from] ParseUciError),

    #[error(transparent)]
    IllegalUci(#[from] IllegalUciError),
}

pub trait UciMove {
    fn make_move(&mut self, new_move: &str) -> Result<(), Error>;
}

impl UciMove for Chess {
    fn make_move(&mut self, new_move: &str) -> Result<(), Error> {
        let new_move = Uci::from_str(new_move)?.to_move(self)?;

        // illegal moves are filtered out by
        // Uci::to_move(m)
        assert!(self.is_legal(&new_move));

        self.play_unchecked(&new_move);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chess_uci_shakmaty() -> Result<(), Box<dyn std::error::Error>> {
        let mut game = shakmaty::Chess::default();

        assert!(game.make_move("e2e4").is_ok());
        assert!(game.make_move("e7e5").is_ok());
        assert!(game.make_move("d2d4").is_ok());
        assert!(game.make_move("d7d5").is_ok());

        Ok(())
    }

    #[test]
    fn chess_uci_shakmaty_error() -> Result<(), Box<dyn std::error::Error>> {
        let mut game = shakmaty::Chess::default();

        assert!(game.make_move("e2e4").is_ok());

        let res = game.make_move("e2e5");
        assert!(res.is_err());
        assert!(match res {
            Err(Error::IllegalUci(_)) => true,
            _ => false,
        });
        println!("res {:?}", res);

        let res = game.make_move("wtf?");
        assert!(res.is_err());
        assert!(match res {
            Err(Error::ParseUci(_)) => true,
            _ => false,
        });
        println!("res {:?}", res);

        let res = game.make_move("e7d4");
        assert!(res.is_err());
        assert!(match res {
            Err(Error::IllegalUci(_)) => true,
            _ => false,
        });
        println!("res {:?}", res);

        assert!(game.make_move("e7e5").is_ok());
        Ok(())
    }
}
