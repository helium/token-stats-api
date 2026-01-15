use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use spl_token::state::Mint;

use crate::api::{empty_string_as_none, TokenType};

pub type SharedRpcClient = Arc<RpcClient>;

pub fn router(rpc_client: SharedRpcClient) -> Router {
    Router::new()
        .route("/api/stats/supply/{token}", get(get_supply))
        .with_state(rpc_client)
}

pub enum SupplyError {
    RpcError(String),
    ParseError(String),
}

impl IntoResponse for SupplyError {
    fn into_response(self) -> Response {
        match &self {
            SupplyError::RpcError(e) => log::error!("Solana RPC error: {}", e),
            SupplyError::ParseError(e) => log::error!("Mint data parse error: {}", e),
        }
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

async fn circulating_supply(token: &TokenType, client: &RpcClient) -> Result<f64, SupplyError> {
    let account = client
        .get_account(&token.mint())
        .await
        .map_err(|e| SupplyError::RpcError(e.to_string()))?;

    let mint = Mint::unpack_from_slice(&account.data)
        .map_err(|e| SupplyError::ParseError(e.to_string()))?;

    let float_supply = mint.supply as f64 / 10f64.powi(mint.decimals as i32);
    Ok(float_supply)
}

#[derive(Deserialize)]
pub struct SupplyParams {
    #[serde(default, deserialize_with = "empty_string_as_none", rename = "type")]
    supply_type: Option<SupplyType>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum SupplyType {
    Max,
    Circulating,
    Total,
}

async fn get_supply(
    State(rpc_client): State<SharedRpcClient>,
    Path(token): Path<TokenType>,
    Query(params): Query<SupplyParams>,
) -> Result<String, SupplyError> {
    match params.supply_type {
        None => Ok("".to_string()),
        Some(supply_type) => match supply_type {
            SupplyType::Max => Ok(token.max_supply().to_string()),
            SupplyType::Circulating | SupplyType::Total => {
                let supply = circulating_supply(&token, &rpc_client).await?;
                Ok(supply.to_string())
            }
        },
    }
}
