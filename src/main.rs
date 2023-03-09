#![deny(warnings)]
mod auth;
mod model;
mod ws;

use model::db::init_db;
use auth::api::{login, signup};
use auth::jwt::{MasterTokenSecret, current_key};
use auth::{jwt, UserCtx};
use warp::fs::dir;

use warp::Filter;
use warp::hyper::Uri;
use ws::user_connected;
use ws::WebsocketUsers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Verbose logging
    std::env::set_var("RUST_LOG", "trace");
    std::env::set_var("RUST_BACKTRACE", "full");
    pretty_env_logger::init();
 
    // GET /auth React app - from filesystem
    let auth =
        warp::get()
        .and(warp::path("auth"))
        .and(dir("frontend-auth/build"));

    // Filter/State - Extract Db connection
    let db = init_db().await?;
    let jwt_secret = current_key(&db).await?; // Read currecnt JWT secret
    let db = warp::any()
        .map(move || db.clone());

    // Filter/State - Extract JWT token secret
    let token_secret = MasterTokenSecret::default();
    *token_secret.write().await = jwt_secret; // Init from Db
    let token_secret =
        warp::any()
        .map(move || token_secret.clone());

    // Filter - Accept only authenticated users
    // Extract the user context (utx) from JWT token cookie
    let with_utx =
        warp::cookie::<String>("token")
        .and(token_secret.clone())
        .and_then(|token: String, token_secret| 
            async move { jwt::to_utx(&token, token_secret).await });


    // Filter/State - Extract Websocket users hash map
    let ws_users = WebsocketUsers::default();
    let ws_users =
        warp::any()
        .map(move || ws_users.clone());

    // GET /chat -> Warp websocket demo
    let chat =
        warp::path("chat")
        .and(with_utx.clone())
        .and(warp::ws())
        .and(db.clone())
        .and(ws_users)
        .map(|utx, ws: warp::ws::Ws, db, users| {
            // This will call our function if the handshake succeeds.
            println!("#### ws.on_upgrade utx {:?}", utx);
            ws.on_upgrade(move |socket| user_connected(socket, db, users))
        });

    // GET / -> index html - from memory Websocket UI
    let index = warp::path::end()
        .and(with_utx.clone())
        .map(|utx: UserCtx | {
            println!("#### INDEX_HTML utx {:?}", utx);
            let html = index_html(&utx.email);
            warp::reply::html(html)
        });

    // POST /login
    let login =
        warp::post()
        .and(warp::path("login"))
        .and(token_secret.clone())
        .and(db.clone())
        .and(warp::body::json())
        .and_then(|token_secret, db, user| async move {
            login(token_secret, db, user).await
        });

    // POST /signup
    let signup =
        warp::post()
        .and(warp::path("signup"))
        .and(token_secret)
        .and(db.clone())
        .and(warp::body::json())
        .and_then(|token_secret, db, user| async move {
            signup(token_secret, db, user).await
        });

    // The default route - Log in with your account to continue.
    let redirect =
        warp::any()
        .map(|| {
            warp::redirect::temporary(Uri::from_static("/auth"))
        });

    // Compose all filters
    let routes =
        auth.or(login).or(signup).or(index).or(chat).or(redirect);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

fn index_html(email: &str) -> String {
    let title = format!("<h1>Hello {}</h1>", email);
    format!("{}{}{}", INDEX_HTML_PRE, title, INDEX_HTML_POST)
}

static INDEX_HTML_PRE: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <link rel="icon" href="/auth/favicon.ico"/>
        <title> Warp websocket demo </title>
    </head>
    <body>
"#;

static INDEX_HTML_POST: &str = r#"<!DOCTYPE html>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);

        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }

        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };

        ws.onmessage = function(msg) {
            message(msg.data);
        };

        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };

        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';

            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
