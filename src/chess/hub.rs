#![allow(dead_code)]
// Recipe Keynote | Actors with Tokio – a lesson in ownership - Alice Ryhl
use std::collections::{HashMap, VecDeque};

use super::*;
use crate::chess::uci::UciMove;
use crate::model::IdType;
use crate::ws::*;
use shakmaty::Chess;
use tokio::{io, sync::mpsc};

#[derive(Debug)]
pub enum Message {
    GameRequest {
        msg: GamePreference,
        respond_to: mpsc::Sender<WsMessage>, // handle to user's Ws Tx
        uid: IdType,                         // user Db Id
    },
    Move {
        uci: String,
        uid: IdType, // user Db Id
    },
    WsDisconnect {
        uid: IdType,
    },
}

struct Player {
    uid: IdType,
    respond_to: mpsc::Sender<WsMessage>,
    color: WsColor,
    opponent: IdType,
}

type LiveGameId = (IdType, IdType);
struct LiveGame {
    game: Chess,
    tc: TimeControl,
    white: IdType,
    black: IdType,
}

struct GameRequest {
    msg: GamePreference,
    respond_to: mpsc::Sender<WsMessage>,
    uid: IdType,
}

type GameRequests = VecDeque<GameRequest>;
type Players = HashMap<IdType, Player>;
type LiveGames = HashMap<LiveGameId, LiveGame>;

struct HubState {
    requests: GameRequests,
    games: LiveGames,
    players: Players,
}

pub struct Hub {
    receiver: mpsc::Receiver<Message>,
}

impl Hub {
    async fn handle_message(&mut self, ctx: &mut HubState, msg: Message) {
        use Message::*;
        match msg {
            GameRequest {
                msg,
                respond_to,
                uid,
            } => {
                self.handle_game_preference(ctx, msg, respond_to, uid).await;
            }
            Move { uci, uid } => {
                self.handle_move(ctx, &uci, uid).await;
            }
            WsDisconnect { uid: _uid } => {
                todo!();
            }
        }
    }

    async fn handle_game_preference(
        &mut self,
        ctx: &mut HubState,
        msg: GamePreference,
        respond_to: mpsc::Sender<WsMessage>,
        uid: IdType,
    ) {
        let reqs = &mut ctx.requests;
        if reqs.is_empty() {
            println!("HUB request from {}: noone there", uid);
            reqs.push_back(GameRequest {
                msg,
                respond_to,
                uid,
            });

            return;
        }
        let opponent = reqs.remove(0).expect("non empty game requests");
        let my_player = Player {
            uid,
            respond_to: respond_to.clone(),
            color: WsColor::White,
            opponent: opponent.uid,
        };
        let opponent_player = Player {
            uid: opponent.uid,
            color: WsColor::Black,
            respond_to: opponent.respond_to.clone(),
            opponent: uid,
        };
        let live_game = LiveGame {
            game: Chess::default(),
            tc: msg.tc,
            white: uid,
            black: opponent.uid,
        };

        let game_id = (uid, opponent.uid);

        ctx.players.insert(uid, my_player);
        ctx.players.insert(uid, opponent_player);
        ctx.games.insert(game_id, live_game);

        let resp = WsMessage::GameResponse(WsColor::White);
        println!("HUB request {} resp to white {:?}", uid, resp);
        let _ = respond_to.send(resp).await;

        let resp = WsMessage::GameResponse(WsColor::Black);
        println!("HUB request {} resp to black {:?}", opponent.uid, resp);
        let _ = opponent.respond_to.send(resp).await;
    }

    async fn handle_move(&mut self, ctx: &mut HubState, uci: &str, uid: IdType) {
        let my_player = match ctx.players.get(&uid) {
            Some(player) => player,
            None => {
                println!("HUB move uci {}, no my player for uid {}", uci, uid);
                return;
            }
        };
        let opponent_player = match ctx.players.get(&my_player.opponent) {
            Some(player) => player,
            None => {
                println!(
                    "HUB move uci {}, no opponent player for uid {}",
                    uci, my_player.opponent
                );
                return;
            }
        };
        let game_id = match my_player.color {
            WsColor::White => (my_player.uid, opponent_player.uid),
            WsColor::Black => (opponent_player.uid, my_player.uid),
        };
        let live_game = match ctx.games.get_mut(&game_id) {
            Some(game) => game,
            None => {
                println!("HUB move uci {}, no live game game id {:?}", uci, game_id);
                return;
            }
        };
        let game = &mut live_game.game;
        match game.make_move(uci) {
            Ok(_) => println!("HUB move uci {}, success", uci),
            Err(e) => println!("HUB move uci {}, make move error {:?}", uci, e),
        }
    }

    async fn run(mut self) -> io::Result<()> {
        let mut ctx = HubState {
            requests: GameRequests::default(),
            players: Players::default(),
            games: LiveGames::default(),
        };
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(&mut ctx, msg).await;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Handle {
    pub sender: mpsc::Sender<Message>,
}

impl Handle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(256);
        tokio::spawn(Hub { receiver }.run());
        Handle { sender }
    }

    pub async fn send(&self, msg: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(msg).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test only
    async fn send_game_request(handle: Handle, msg: GamePreference, uid: IdType) {
        let (respond_to, mut receiver) = mpsc::channel::<WsMessage>(8);
        let msg = Message::GameRequest {
            msg,
            respond_to,
            uid,
        };

        let _ = handle.send(msg).await;
        let msg = receiver.recv().await.expect("Hub is dead");
        println!("reply {:?}", msg);
    }

    #[tokio::test]
    async fn chess_hub() -> Result<(), Box<dyn std::error::Error>> {
        let handle = Handle::new();
        let mut jhs = vec![];
        for i in 0..8 {
            let handle = handle.clone();
            let jh = tokio::spawn(async move {
                send_game_request(handle, GamePreference::default(), i).await;
            });
            jhs.push(jh);
        }
        for jh in jhs {
            let _ = jh.await;
        }

        Ok(())
    }
}
