use std::env;
use std::net::SocketAddr;

use axum::{response::Redirect, routing::get, Router};
use dotenv::dotenv;
use tokio::net::TcpListener;

mod api;

use crate::api::supply::get_supply;

lazy_static::lazy_static! {
    static ref SOLANA_RPC: String = env::var("SOLANA_RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let app = Router::new()
        .route("/api/stats/supply/{token}", get(get_supply))
        .fallback(async || Redirect::permanent("https://world.helium.com"));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Unable to bind to address");

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
