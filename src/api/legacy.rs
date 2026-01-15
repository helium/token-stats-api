use axum::{extract::Path, http::Uri, response::Redirect, routing::get, Router};

use crate::api::maybe_convert_to_solana;

const WORLD_HELIUM: &str = "https://world.helium.com";

pub fn router() -> Router {
    Router::new()
        .route("/accounts/{account}", get(redirect_account))
        .route("/accounts/{account}/{*rest}", get(redirect_account_subpath))
        .route("/hotspots/{hotspot}", get(redirect_hotspot))
        .route("/hotspots/{hotspot}/{*rest}", get(redirect_hotspot_subpath))
}

pub async fn fallback(_uri: Uri) -> Redirect {
    Redirect::permanent(WORLD_HELIUM)
}

async fn redirect_account(Path(address): Path<String>) -> Redirect {
    match maybe_convert_to_solana(address) {
        None => Redirect::permanent(WORLD_HELIUM),
        Some(solana_address) => {
            Redirect::permanent(&format!("{}/mobile/wallet/{}", WORLD_HELIUM, solana_address))
        }
    }
}

async fn redirect_account_subpath(Path((address, _)): Path<(String, String)>) -> Redirect {
    redirect_account(Path(address)).await
}

async fn redirect_hotspot(Path(address): Path<String>) -> Redirect {
    Redirect::permanent(&format!("{}/iot/hotspots/gateway/{}", WORLD_HELIUM, address))
}

async fn redirect_hotspot_subpath(Path((address, _)): Path<(String, String)>) -> Redirect {
    redirect_hotspot(Path(address)).await
}
