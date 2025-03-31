use std::str::FromStr;

use axum::extract::Query;
use helium_crypto::WriteTo;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

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

fn maybe_convert_to_helium(address: String) -> Option<String> {
    match Pubkey::from_str(address.as_str()) {
        Err(_) => None,
        Ok(pubkey) => {
            use helium_crypto::ReadFrom;
            let mut input = std::io::Cursor::new(pubkey.as_ref());
            match helium_crypto::ed25519::PublicKey::read_from(&mut input) {
                Err(_) => None,
                Ok(helium_pubkey) => {
                    let mut data = vec![0u8; ed25519_compact::PublicKey::BYTES + 2];
                    data[0] = 0x0;
                    data[1] = 0x1;
                    helium_pubkey
                        .write_to(&mut std::io::Cursor::new(&mut data[2..]))
                        .unwrap();
                    let encoded = bs58::encode(&data).with_check().into_string();
                    Some(encoded.to_string())
                }
            }
        }
    }
}

fn maybe_convert_to_solana(address: String) -> Option<String> {
    match bs58::decode(address).with_check(None).into_vec() {
        Err(_) => None,
        Ok(decoded) => match decoded[2..(ed25519_compact::PublicKey::BYTES + 2)].try_into() {
            Err(_) => None,
            Ok(data) => Some(Pubkey::new_from_array(data).to_string()),
        },
    }
}
