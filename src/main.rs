use std::env;
use std::net::SocketAddr;

use axum::{routing::get, Router};
use dotenv::dotenv;
use tokio::net::TcpListener;

mod api;

use crate::api::address::get_address;
use crate::api::legacy::{
    handle_legacy_accounts, handle_legacy_accounts_subpaths, handle_legacy_hotspots,
    handle_legacy_hotspots_subpaths, handle_unknown_legacy_routes,
};
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
        .route("/api/tools/address", get(get_address))
        // known legacy routes
        .route("/accounts/{account}", get(handle_legacy_accounts))
        .route(
            "/accounts/{account}/{*rest}",
            get(handle_legacy_accounts_subpaths),
        )
        .route("/hotspots/{hotspot}", get(handle_legacy_hotspots))
        .route(
            "/hotspots/{hotspot}/{*rest}",
            get(handle_legacy_hotspots_subpaths),
        )
        .fallback(handle_unknown_legacy_routes);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Unable to bind to address");

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
