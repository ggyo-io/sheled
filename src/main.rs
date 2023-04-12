#![deny(warnings)]
mod auth;
mod chess;
mod model;
mod ws;

use warp::fs::dir;
use warp::hyper::Uri;
use warp::Filter;

use auth::api::{login, signup};
use auth::jwt::{current_key, MasterTokenSecret};
use auth::{jwt, UserCtx};
use chess::hub::Handle;
use model::db::init_db;
use ws::user_connected;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Verbose logging
    std::env::set_var("RUST_LOG", "trace");
    std::env::set_var("RUST_BACKTRACE", "full");
    pretty_env_logger::init();

    // GET /auth React app - from filesystem
    let auth = warp::path("auth").and(dir("frontend-auth/build"));

    // Filter/State - Extract Db connection
    let db = init_db().await?;
    let jwt_secret = current_key(&db).await?; // Read currecnt JWT secret
    let db = warp::any().map(move || db.clone());

    // Filter/State - Extract JWT token secret
    let token_secret = MasterTokenSecret::default();
    *token_secret.write().await = jwt_secret; // Init from Db
    let token_secret = warp::any().map(move || token_secret.clone());

    // Filter - Accept only authenticated users
    // Extract the user context (utx) from JWT token cookie
    let with_utx = warp::cookie::<String>("token")
        .and(token_secret.clone())
        .and_then(
            |token: String, token_secret| async move { jwt::to_utx(&token, token_secret).await },
        );

    // Filter/State - Extract Hub handle
    let hub = Handle::new();
    let hub = warp::any().map(move || hub.clone());

    // /ws -> hub websocket interface
    let ws = warp::path("ws")
        .and(with_utx.clone())
        .and(warp::ws())
        .and(db.clone())
        .and(hub)
        .map(|utx, ws: warp::ws::Ws, db, hub: Handle| {
            // This will call our function if the handshake succeeds.
            println!("#### ws.on_upgrade utx {:?}", utx);
            ws.on_upgrade(move |socket| user_connected(socket, db, hub, utx))
        });

    // GET / -> Authenticated Websocket UI
    let index = with_utx
        .clone()
        .map(|utx: UserCtx| {
            println!("#### INDEX_HTML utx {:?}", utx);
        })
        .untuple_one()
        .and(dir("ui/dist"));

    // POST /login
    let login = warp::post()
        .and(warp::path("login"))
        .and(token_secret.clone())
        .and(db.clone())
        .and(warp::body::json())
        .and_then(|token_secret, db, user| async move { login(token_secret, db, user).await });

    // POST /signup
    let signup = warp::post()
        .and(warp::path("signup"))
        .and(token_secret)
        .and(db.clone())
        .and(warp::body::json())
        .and_then(|token_secret, db, user| async move { signup(token_secret, db, user).await });

    // The default route - Log in with your account to continue.
    let redirect = warp::any().map(|| warp::redirect::temporary(Uri::from_static("/auth")));

    // Compose all filters
    let routes = auth.or(login).or(signup).or(ws).or(index).or(redirect);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
