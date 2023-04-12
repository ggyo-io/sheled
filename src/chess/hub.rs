// Recipe Keynote | Actors with Tokio – a lesson in ownership - Alice Ryhl
use super::*;
use crate::model::IdType;
use crate::ws::*;
use tokio::{io, sync::mpsc};

#[derive(Debug)]
pub enum Message {
    GameRequest {
        msg: GamePreference,
        respond_to: mpsc::Sender<WsMessage>, // handle to user's Ws Tx
        uid: IdType,                         // user Db Id
    },
    WsDisconnect {
        uid: IdType,
    },
}

pub struct Hub {
    receiver: mpsc::Receiver<Message>,
}

impl Hub {
    async fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::GameRequest {
                msg,
                respond_to,
                uid: _uid,
            } => {
                self.handle_game_preference(msg, &respond_to).await;
            }
            Message::WsDisconnect { uid: _uid } => {}
        }
    }

    async fn handle_game_preference(
        &mut self,
        msg: GamePreference,
        respond_to: &mpsc::Sender<WsMessage>,
    ) {
        let resp = WsMessage::GameResponse(WsColor::default());
        println!("HUB request {:?} resp {:?}", msg, resp);
        let _ = respond_to.send(resp).await;
    }

    async fn run(mut self) -> io::Result<()> {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
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
