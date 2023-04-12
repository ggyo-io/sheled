#![allow(dead_code)]
use std::str::FromStr;

use sacrifice::*;
use shakmaty::uci::*;
use uuid::Uuid;

struct MainlineGame {
    game: Game,
    current_node: Option<Uuid>,
}

impl Default for MainlineGame {
    fn default() -> Self {
        MainlineGame {
            game: Game::from_pgn(
                r#"[Event "Online game"]
[Site "http://ggyo.io"]
[Date "2023.04.17"]
[Round "17"]
[White "Shmulik"]
[Black "Srulik"]
[Result "*"]"#,
            ),
            current_node: None,
        }
    }
}

impl MainlineGame {
    fn current_node(&self) -> Uuid {
        match self.current_node {
            None => self.game.root(),
            Some(node) => node,
        }
    }

    fn make_move(&mut self, new_move: &str) -> Result<Uuid, Box<dyn std::error::Error>> {
        let current_node = self.current_node();
        let pos = self.game.board_at(current_node).expect("current position");
        let new_move = Uci::from_str(new_move)?.to_move(&pos)?;
        let new_node = self
            .game
            .add_node(current_node, new_move)
            .expect("legal move");
        self.current_node = Some(new_node);
        Ok(new_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn chess_mainline_game() -> Result<(), Box<dyn std::error::Error>> {
        let mut game = MainlineGame::default();
        assert!(game.make_move("e2e4").is_ok());
        assert!(game.make_move("e7e5").is_ok());
        assert!(game.make_move("d2d4").is_ok());
        assert!(game.make_move("d7d5").is_ok());

        println!(
            "mainline {} current node {} root {}",
            game.game.mainline(game.game.root()).unwrap(),
            game.current_node(),
            game.game.root()
        );
        println!("game {}", game.game);

        Ok(())
    }

    #[tokio::test]
    async fn chess_mainline_game_error() -> Result<(), Box<dyn std::error::Error>> {
        let mut game = MainlineGame::default();
        assert!(game.make_move("e2e4").is_ok());
        assert!(game.make_move("e2e4").is_err());
        println!("game {}", game.game);

        println!(
            "mainline {} current node {} root {}",
            game.game.mainline(game.game.root()).unwrap(),
            game.current_node(),
            game.game.root()
        );
        Ok(())
    }
}
