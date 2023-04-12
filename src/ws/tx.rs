use futures_util::{stream::SplitSink, SinkExt, TryFutureExt};
use tokio::sync::mpsc;

use super::*;

#[derive(Clone)]
pub struct WsHandleTx {
    pub sender: mpsc::Sender<WsMessage>,
}

impl WsHandleTx {
    pub fn new(con: SplitSink<WebSocket, Message>) -> Self {
        let (sender, receiver) = mpsc::channel(256);
        tokio::spawn(WsConnTx { receiver, con }.run());
        WsHandleTx { sender }
    }
}

struct WsConnTx {
    receiver: mpsc::Receiver<WsMessage>, // from Hub
    con: SplitSink<WebSocket, Message>,  // Tx WebSocket
}

impl WsConnTx {
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            let msg = serde_json::to_string(&msg).unwrap();

            self.con
                .send(Message::text(msg))
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    }
}
