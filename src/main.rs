use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use dotenv::dotenv;
use solana_client::nonblocking::rpc_client::RpcClient;
use tokio::net::TcpListener;

mod api;

use crate::api::address::get_address;
use crate::api::legacy;
use crate::api::supply;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let solana_rpc = env::var("SOLANA_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let rpc_client = Arc::new(RpcClient::new(solana_rpc));

    let app = Router::new()
        .merge(supply::router(rpc_client))
        .merge(legacy::router())
        .route("/api/tools/address", get(get_address))
        .fallback(legacy::fallback);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Unable to bind to address");

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
