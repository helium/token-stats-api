use crate::api::maybe_convert_to_solana;

use axum::{extract::Path, http::Uri, response::Redirect};

pub async fn handle_legacy_accounts(Path(address): Path<String>) -> Redirect {
    handle_legacy_accounts_redirect(address).await
}

pub async fn handle_legacy_accounts_subpaths(Path(address): Path<(String, String)>) -> Redirect {
    handle_legacy_accounts_redirect(address.0).await
}

async fn handle_legacy_accounts_redirect(address: String) -> Redirect {
    match maybe_convert_to_solana(address) {
        None => Redirect::permanent("https://world.helium.com"),
        Some(solana_address) => Redirect::permanent(
            format!("https://world.helium.com/mobile/wallet/{}", solana_address).as_str(),
        ),
    }
}

pub async fn handle_legacy_hotspots(Path(address): Path<String>) -> Redirect {
    handle_legacy_hotspots_redirect(address).await
}

pub async fn handle_legacy_hotspots_subpaths(Path(address): Path<(String, String)>) -> Redirect {
    handle_legacy_hotspots_redirect(address.0).await
}

async fn handle_legacy_hotspots_redirect(address: String) -> Redirect {
    Redirect::permanent(
        format!("https://world.helium.com/iot/hotspots/gateway/{}", address).as_str(),
    )
}

pub async fn handle_unknown_legacy_routes(_uri: Uri) -> Redirect {
    Redirect::permanent("https://world.helium.com")
}
