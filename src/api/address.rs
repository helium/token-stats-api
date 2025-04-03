use axum::extract::Query;
use serde::Deserialize;

use crate::api::{maybe_convert_to_helium, maybe_convert_to_solana};

#[derive(Deserialize)]
pub struct AddressParams {
    address: String,
    #[serde(rename = "to")]
    network: NetworkType,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum NetworkType {
    Helium,
    Solana,
}

pub async fn get_address(Query(params): Query<AddressParams>) -> String {
    match params.network {
        NetworkType::Helium => maybe_convert_to_helium(params.address),
        NetworkType::Solana => maybe_convert_to_solana(params.address),
    }
    .unwrap_or("".to_string())
}
