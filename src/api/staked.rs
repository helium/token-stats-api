use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey, pubkey::Pubkey};
use tokio::sync::{Mutex, RwLock};

use crate::api::TokenType;

// Helium Voter Stake Registry program
const VSR_PROGRAM_ID: Pubkey = pubkey!("hvsrNC3NKbcryqDs2DocYHZ9yPKEVzdSjQG6RVtK1s8");

// PositionV0 account layout offsets
const REGISTRAR_OFFSET: usize = 8; // After 8-byte discriminator
const AMOUNT_OFFSET: usize = 89; // After discriminator + registrar + mint + lockup

// Background refresh interval
const REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60); // 15 minutes

struct TokenCache {
    value: RwLock<Option<f64>>,
    fetch_lock: Mutex<()>,
}

impl TokenCache {
    fn new() -> Self {
        Self {
            value: RwLock::new(None),
            fetch_lock: Mutex::new(()),
        }
    }
}

pub struct StakeCache {
    hnt: TokenCache,
    iot: TokenCache,
    mobile: TokenCache,
}

impl StakeCache {
    pub fn new() -> Self {
        Self {
            hnt: TokenCache::new(),
            iot: TokenCache::new(),
            mobile: TokenCache::new(),
        }
    }

    fn get_cache(&self, token: &TokenType) -> &TokenCache {
        match token {
            TokenType::Hnt => &self.hnt,
            TokenType::Iot => &self.iot,
            TokenType::Mobile => &self.mobile,
        }
    }
}

pub struct AppState {
    pub rpc_client: Arc<RpcClient>,
    pub stake_cache: StakeCache,
}

pub enum StakedError {
    RpcError(String),
    ParseError(String),
}

impl IntoResponse for StakedError {
    fn into_response(self) -> Response {
        match &self {
            StakedError::RpcError(e) => log::error!("Staked supply RPC error: {}", e),
            StakedError::ParseError(e) => log::error!("Staked supply parse error: {}", e),
        }
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

const ALL_TOKENS: [TokenType; 3] = [TokenType::Hnt, TokenType::Iot, TokenType::Mobile];

pub fn router(rpc_client: Arc<RpcClient>) -> Router {
    let state = Arc::new(AppState {
        rpc_client,
        stake_cache: StakeCache::new(),
    });

    // Spawn background refresh task
    tokio::spawn(background_refresh(state.clone()));

    Router::new()
        .route("/api/stats/staked/{token}", get(get_staked))
        .with_state(state)
}

async fn background_refresh(state: Arc<AppState>) {
    // Initial fetch for all tokens
    log::info!("Starting initial staked supply cache population");
    for token in &ALL_TOKENS {
        if let Err(e) = refresh_token_cache(&state, token).await {
            log::error!("Failed to fetch initial staked supply for {:?}: {:?}", token, e);
        }
    }
    log::info!("Initial staked supply cache population complete");

    // Periodic refresh
    loop {
        tokio::time::sleep(REFRESH_INTERVAL).await;
        log::info!("Refreshing staked supply cache");
        for token in &ALL_TOKENS {
            if let Err(e) = refresh_token_cache(&state, token).await {
                log::error!("Failed to refresh staked supply for {:?}: {:?}", token, e);
            }
        }
    }
}

async fn refresh_token_cache(state: &AppState, token: &TokenType) -> Result<(), StakedError> {
    let cache = state.stake_cache.get_cache(token);

    // Acquire fetch lock to prevent parallel fetches
    let _lock = cache.fetch_lock.lock().await;

    // Fetch fresh data
    let value = fetch_staked_supply(&state.rpc_client, token).await?;

    // Update cache
    let mut cached = cache.value.write().await;
    *cached = Some(value);

    log::info!("Updated staked supply cache for {:?}: {}", token, value);
    Ok(())
}

async fn get_staked(
    State(state): State<Arc<AppState>>,
    Path(token): Path<TokenType>,
) -> Result<String, StakedError> {
    let cache = state.stake_cache.get_cache(&token);

    // Fast path: check if we have a cached value
    {
        let cached = cache.value.read().await;
        if let Some(value) = *cached {
            return Ok(value.to_string());
        }
    }

    // Slow path: need to fetch (blocks until fetch completes)
    // Acquire fetch lock - this ensures only one fetch happens
    let _lock = cache.fetch_lock.lock().await;

    // Double-check: another request might have populated the cache while we waited
    {
        let cached = cache.value.read().await;
        if let Some(value) = *cached {
            return Ok(value.to_string());
        }
    }

    // Still no cache, fetch now
    let value = fetch_staked_supply(&state.rpc_client, &token).await?;

    // Update cache
    let mut cached = cache.value.write().await;
    *cached = Some(value);

    Ok(value.to_string())
}

async fn fetch_staked_supply(client: &RpcClient, token: &TokenType) -> Result<f64, StakedError> {
    let registrar = token.registrar();

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
            REGISTRAR_OFFSET,
            registrar.to_bytes().to_vec(),
        ))]),
        account_config: RpcAccountInfoConfig {
            commitment: Some(CommitmentConfig::confirmed()),
            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
            data_slice: Some(solana_account_decoder::UiDataSliceConfig {
                offset: AMOUNT_OFFSET,
                length: 8,
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    let accounts = client
        .get_program_accounts_with_config(&VSR_PROGRAM_ID, config)
        .await
        .map_err(|e| StakedError::RpcError(e.to_string()))?;

    let mut total: u64 = 0;

    for (_pubkey, account) in accounts {
        if account.data.len() < 8 {
            continue; // Skip accounts with incomplete data slice
        }

        let amount_bytes: [u8; 8] = account.data[..8]
            .try_into()
            .map_err(|_| StakedError::ParseError("Failed to parse amount bytes".to_string()))?;

        let amount = u64::from_le_bytes(amount_bytes);
        total = total.saturating_add(amount);
    }

    let decimals = token.decimals();
    let float_total = total as f64 / 10f64.powi(decimals);

    Ok(float_total)
}

impl std::fmt::Debug for StakedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StakedError::RpcError(e) => write!(f, "RpcError({})", e),
            StakedError::ParseError(e) => write!(f, "ParseError({})", e),
        }
    }
}
