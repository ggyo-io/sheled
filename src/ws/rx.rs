use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use warp::ws::WebSocket;

use super::*;
use crate::model::IdType;

pub struct WsConnRx {
    receiver: SplitStream<WebSocket>, // Rx WebSocket
    hub: Handle,                      // to Hub
    ws_handle_tx: tx::WsHandleTx,     // Respond to from Hub, user Ws Tx
    uid: IdType,                      // User DB Id
}

impl WsConnRx {
    pub fn new(
        receiver: SplitStream<WebSocket>,
        hub: Handle,
        ws_handle_tx: tx::WsHandleTx,
        uid: IdType,
    ) -> Self {
        WsConnRx {
            receiver,
            hub,
            ws_handle_tx,
            uid,
        }
    }

    // forward messages from WebSocket to Hub
    async fn handle_message(&self, msg: WsMessage) {
        match msg {
            WsMessage::GameRequest(msg) => {
                let uid = self.uid;
                let respond_to = self.ws_handle_tx.sender.clone();
                let msg = HubMessage::GameRequest {
                    msg,
                    respond_to,
                    uid,
                };
                self.hub.send(msg).await.unwrap();
            }
            WsMessage::Move(uci) => {
                let uid = self.uid;
                let msg = HubMessage::Move { uci, uid };
                self.hub.send(msg).await.unwrap();
            }
            _ => eprintln!("WsConnRx::handle_message() unexpected msg: {:?}", msg),
        }
    }

    async fn handle_disconnect(&self) {
        let uid = self.uid;

        self.hub
            .send(HubMessage::WsDisconnect { uid })
            .await
            .unwrap();
    }

    pub async fn run(mut self) {
        while let Some(result) = self.receiver.next().await {
            if let Err(e) = result {
                eprintln!("websocket error: {}", e);
                break;
            }

            let msg = result.unwrap();
            let msg_str = match msg.to_str() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("expected text websocket message: {:?}", msg);
                    continue;
                }
            };
            println!("WsConnRX: {}", msg_str);

            let msg = match serde_json::from_str::<WsMessage>(msg_str) {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("json error: {}", e);
                    continue;
                }
            };

            self.handle_message(msg).await;
        }

        self.handle_disconnect().await;
    }
}
