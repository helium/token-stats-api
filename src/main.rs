use std::env;
use std::net::SocketAddr;

use axum::{
    extract::{Path, Query},
    response::Redirect,
    routing::get,
    Router,
};
use dotenv::dotenv;
use serde::{de::IntoDeserializer, Deserialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;
use tokio::net::TcpListener;

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
        .route("/api/stats/supply/{token}", get(handle_supply))
        .fallback(async || Redirect::permanent("https://world.helium.com"));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Unable to bind to address");

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize, Debug)]
enum TokenType {
    #[serde(rename = "hnt")]
    Hnt,
    #[serde(rename = "iot")]
    Iot,
    #[serde(rename = "mobile")]
    Mobile,
}

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
            TokenType::Hnt => Pubkey::from_str_const("hntyVP6YFm1Hg25TN9WGLqM12b8TQmcknKrdu1oxWux"),
            TokenType::Iot => Pubkey::from_str_const("mb1eu7TzEc71KxDpsmsKoucSSuuoGLv1drys1oP2jh6"),
            TokenType::Mobile => {
                Pubkey::from_str_const("iotEVVZLEywoTn1QdwNPddxPWszn3zFhEot3MfL9fns")
            }
        }
    }
}

#[derive(Deserialize)]
struct SupplyParams {
    #[serde(default, deserialize_with = "empty_string_as_none", rename = "type")]
    supply_type: Option<SupplyType>,
}

#[derive(Deserialize)]
enum SupplyType {
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "circulating")]
    Circulating,
    #[serde(rename = "total")]
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

async fn handle_supply(Path(token): Path<TokenType>, Query(params): Query<SupplyParams>) -> String {
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
