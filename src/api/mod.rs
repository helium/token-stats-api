pub mod address;
pub mod legacy;
pub mod supply;

use std::str::FromStr;

use serde::{de::IntoDeserializer, Deserialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Hnt,
    Iot,
    Mobile,
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

fn maybe_convert_to_helium(address: String) -> Option<String> {
    Pubkey::from_str(&address).ok()
        .and_then(|pk| helium_crypto::PublicKey::try_from(pk).ok())
        .map(|pk| pk.to_string())
}

fn maybe_convert_to_solana(address: String) -> Option<String> {
    helium_crypto::PublicKey::from_str(&address).ok()
        .and_then(|pk| Pubkey::try_from(pk).ok())
        .map(|pk| pk.to_string())
}
