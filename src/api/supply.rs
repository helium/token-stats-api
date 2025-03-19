use axum::extract::{Path, Query};
use serde::{de::IntoDeserializer, Deserialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;

use crate::api::TokenType;
use crate::SOLANA_RPC;

impl TokenType {
    fn max_supply(&self) -> f64 {
        match self {
            TokenType::Hnt => 223_000_000f64,
            TokenType::Iot => 200_000_000_000f64,
            TokenType::Mobile => 230_000_000_000f64,
        }
    }

    fn maybe_circulating_supply(&self) -> Option<f64> {
        let client = RpcClient::new(SOLANA_RPC.to_string());
        let account = client.get_account(&self.mint());

        match account {
            Err(_) => return None,
            Ok(a) => {
                let mint = Mint::unpack_from_slice(&a.data).unwrap();
                let float_supply = mint.supply as f64 / 10f64.powi(mint.decimals as i32);
                return Some(float_supply);
            }
        }
    }

    fn mint(&self) -> Pubkey {
        match self {
            TokenType::Hnt => pubkey!("hntyVP6YFm1Hg25TN9WGLqM12b8TQmcknKrdu1oxWux"),
            TokenType::Iot => pubkey!("mb1eu7TzEc71KxDpsmsKoucSSuuoGLv1drys1oP2jh6"),
            TokenType::Mobile => pubkey!("iotEVVZLEywoTn1QdwNPddxPWszn3zFhEot3MfL9fns"),
        }
    }
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

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_ref().map(String::as_str);
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer())
            .map(Some)
            .or_else(|_: <D as serde::Deserializer<'de>>::Error| Ok(None)),
    }
}

pub async fn get_supply(
    Path(token): Path<TokenType>,
    Query(params): Query<SupplyParams>,
) -> String {
    match params.supply_type {
        None => "".to_string(),
        Some(supply_type) => match supply_type {
            SupplyType::Max => token.max_supply().to_string(),
            SupplyType::Circulating | SupplyType::Total => {
                token.maybe_circulating_supply().unwrap_or(0f64).to_string()
            }
        },
    }
}
