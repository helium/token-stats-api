use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;

use bytes::Bytes;
use dotenv::dotenv;
use http_body_util::Full;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::{Request, Response};
use hyper::service::service_fn;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Mint;
use tokio::net::TcpListener;

lazy_static::lazy_static! {
    static ref HNT_MINT: Pubkey = Pubkey::from_str_const("hntyVP6YFm1Hg25TN9WGLqM12b8TQmcknKrdu1oxWux");
    static ref MOBILE_MINT: Pubkey = Pubkey::from_str_const("mb1eu7TzEc71KxDpsmsKoucSSuuoGLv1drys1oP2jh6");
    static ref IOT_MINT: Pubkey = Pubkey::from_str_const("iotEVVZLEywoTn1QdwNPddxPWszn3zFhEot3MfL9fns");

    static ref SOLANA_RPC: String = env::var("SOLANA_RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
}

const HNT_MAX_SUPPLY: f64 = 223_000_000f64;
const MOBILE_MAX_SUPPLY: f64 = 230_000_000_000f64;
const IOT_MAX_SUPPLY: f64 = 200_000_000_000f64;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    dotenv().ok();

    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handler))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handler(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let supply_type = maybe_get_type(req.uri().query());
    match req.uri().path() {
        "/api/stats/supply/iot" => return handle_supply(&MOBILE_MINT, supply_type),
        "/api/stats/supply/mobile" => return handle_supply(&IOT_MINT, supply_type),
        "/api/stats/supply/hnt" => return handle_supply(&HNT_MINT, supply_type),
        _ => {}
    }

    // If not a supply API call, fall back to a redirect.
    redirect_world()
}

fn handle_supply(mint: &Pubkey, supply_type: SupplyType) -> Result<Response<Full<Bytes>>, Infallible> {
    match supply_type {
        SupplyType::Unknown => empty_client_error(),
        SupplyType::Max => handle_max_supply(*mint),
        SupplyType::Circulating | SupplyType::Total => response(get_token_circulating_supply(mint).unwrap_or(0f64).to_string()),
    }
}

fn handle_max_supply(mint: Pubkey) -> Result<Response<Full<Bytes>>, Infallible> {
    let max_supply = match mint {
        m if m == *HNT_MINT => HNT_MAX_SUPPLY,
        m if m == *MOBILE_MINT => MOBILE_MAX_SUPPLY,
        m if m == *IOT_MINT => IOT_MAX_SUPPLY,
        _ => return empty_client_error(),
    };
    response(max_supply.to_string())
}

fn get_token_circulating_supply(mint_pubkey: &Pubkey) -> Option<f64> {
    let client = RpcClient::new(SOLANA_RPC.to_string());
    let account = client.get_account(&mint_pubkey);

    match account {
        Err(_) => return None,
        Ok(a) => {
            let mint = Mint::unpack_from_slice(&a.data).unwrap();
            let float_supply = mint.supply as f64 / 10f64.powi(mint.decimals as i32);
            return Some(float_supply);
        }
    }
}

enum SupplyType {
    Max,
    Circulating,
    Total,
    Unknown,
}

fn maybe_get_type(query: Option<&str>) -> SupplyType {
    match query {
        None => SupplyType::Unknown,
        Some(q) => {
            let query_pairs: Vec<&str> = q.split('&').collect();
            for pair in query_pairs {
                let mut key_value = pair.split('=');
                if let Some(key) = key_value.next() {
                    if key == "type" {
                        if let Some(value) = key_value.next() {
                            return match value {
                                "max" => SupplyType::Max,
                                "circulating" => SupplyType::Circulating,
                                "total" => SupplyType::Total,
                                _ => SupplyType::Unknown,
                            };
                        }
                    }
                }
            }
            SupplyType::Unknown
        }
    }
}

fn empty_client_error() -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(400)
        .body(Full::new(Bytes::new()))
        .unwrap())
}

fn response(body: String) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(200)
        .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

fn redirect_world() -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(308)
        .header("Location", "https://world.helium.com")
        .body(Full::new(Bytes::new()))
        .unwrap())
}
