use futures_util::StreamExt;
use warp::ws::{Message, WebSocket};

use crate::auth::UserCtx;
use crate::chess::hub::{Handle, Message as HubMessage};
use crate::chess::GamePreference;
use crate::model::db::Db;
use serde::{Deserialize, Serialize};

pub mod rx;
pub mod tx;

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum WsColor {
    #[default]
    White,
    Black,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WsMessage {
    GameRequest(GamePreference),
    GameResponse(WsColor),
    Move(String),
}

pub async fn user_connected(ws: WebSocket, _db: Db, hub: Handle, utx: UserCtx) {
    eprintln!("new ws user: {} {} {}", utx.id, &utx.name, &utx.email);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, user_ws_rx) = ws.split();
    let tx_con = tx::WsHandleTx::new(user_ws_tx);
    let rx_con = rx::WsConnRx::new(user_ws_rx, hub, tx_con, utx.id);
    rx_con.run().await;
}

#[cfg(test)]
mod tests {

    use futures_channel::mpsc::UnboundedSender;
    use futures_util::{stream::SplitStream, StreamExt};
    use http::{header::COOKIE, header::SET_COOKIE, Request};
    use rand::{distributions::Alphanumeric, Rng};
    use tokio::net::TcpStream;
    use tokio_tungstenite::{
        connect_async, tungstenite::client::IntoClientRequest, tungstenite::protocol::Message,
        MaybeTlsStream, WebSocketStream,
    };

    use super::*;
    use crate::auth::api::UserSignup;

    #[tokio::test]
    async fn ws_messages_json() -> Result<(), Box<dyn std::error::Error>> {
        let messages = [
            WsMessage::GameRequest(GamePreference::default()),
            WsMessage::GameResponse(WsColor::default()),
        ];
        for msg in messages {
            let msg = serde_json::to_string(&msg).unwrap();
            println!("json string {}", msg);

            let msg: WsMessage = serde_json::from_str(&msg)?;
            println!("struct {:?}", msg);
        }
        Ok(())
    }

    #[tokio::test]
    async fn ws_game_request_response() -> Result<(), Box<dyn std::error::Error>> {
        let mut jhs = vec![];
        for _ in 0..2 {
            let jh = tokio::spawn(async move {
                let res = game_request_response().await;
                println!("game_request_response res {:?}", res);
                res.is_ok()
            });
            jhs.push(jh);
        }
        for jh in jhs {
            match jh.await {
                Ok(r) => assert!(r),
                Err(e) => panic!("error {:?}", e),
            }
        }

        Ok(())
    }

    async fn game_request_response() -> Result<(), Box<dyn std::error::Error>> {
        // Connect to WS endpoint
        let (tx_sender, mut ws_read) = ws_client().await;

        // Send message
        let req = WsMessage::GameRequest(GamePreference::default());
        tx_sender
            .unbounded_send(Message::Text(serde_json::to_string(&req)?))
            .unwrap();
        println!("send {:?}", req);

        // Read, parse verify response
        let resp = ws_read.next().await;
        println!("ws_read {:?}", resp);
        let resp = resp.unwrap()?;
        let resp = match resp {
            Message::Text(resp) => serde_json::from_str::<WsMessage>(&resp)?,
            _ => panic!("Expected text reply, got {:?}", resp),
        };
        println!("read {:?}", resp);

        Ok(())
    }

    fn rand_string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    async fn cookie_header_value() -> String {
        let signup_url = "http://localhost:3030/signup";
        let u = UserSignup {
            name: rand_string(8),
            email: rand_string(8),
            password: rand_string(8),
        };

        // Send POST request to signup endpoint
        let client = reqwest::Client::new();
        let response = client.post(signup_url).json(&u).send().await.unwrap();
        let header_value = response.headers().get(SET_COOKIE).unwrap();
        let token = header_value
            .to_str()
            .unwrap()
            .split("; ")
            .next()
            .unwrap()
            .into();

        println!("{:?}", token);

        token
    }

    async fn authenticated_ws_request() -> Request<()> {
        // Create the WebSocket HTTP request
        let connect_addr = "ws://localhost:3030/ws";
        let url = url::Url::parse(connect_addr).unwrap();
        let mut request: Request<()> = url.into_client_request().unwrap();
        let headers = request.headers_mut();
        headers.insert(COOKIE, cookie_header_value().await.parse().unwrap());
        println!("ws request {:?}", request);
        request
    }

    async fn ws_client() -> (
        UnboundedSender<Message>,
        SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {
        let auth_ws_req = authenticated_ws_request().await;
        let (ws_stream, _) = connect_async(auth_ws_req).await.expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");
        let (ws_write, ws_read) = ws_stream.split();

        let (tx_sender, tx_recv) = futures_channel::mpsc::unbounded();
        let tx_to_ws = tx_recv.map(Ok).forward(ws_write);
        tokio::spawn(tx_to_ws);

        (tx_sender, ws_read)
    }
}
