pub mod address;
pub mod legacy;
pub mod staked;
pub mod supply;

use std::str::FromStr;

use serde::{de::IntoDeserializer, Deserialize};
use solana_sdk::{pubkey, pubkey::Pubkey};

// Token mints
const HNT_MINT: Pubkey = pubkey!("hntyVP6YFm1Hg25TN9WGLqM12b8TQmcknKrdu1oxWux");
const IOT_MINT: Pubkey = pubkey!("iotEVVZLEywoTn1QdwNPddxPWszn3zFhEot3MfL9fns");
const MOBILE_MINT: Pubkey = pubkey!("mb1eu7TzEc71KxDpsmsKoucSSuuoGLv1drys1oP2jh6");

// VSR registrar addresses
const HNT_REGISTRAR: Pubkey = pubkey!("BMnWRWZrWqb6JMKznaDqNxWaWAHoaTzVabM6Qwyh3WKz");
const IOT_REGISTRAR: Pubkey = pubkey!("7ZZopN1mx6ECcb3YCG8dbxeLpA44xq4gzA1ETEiaLoeL");
const MOBILE_REGISTRAR: Pubkey = pubkey!("C4DWaps9bLiqy4e81wJ7VTQ6QR7C4MWvwsei3ZjsaDuW");

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Hnt,
    Iot,
    Mobile,
}

impl TokenType {
    pub fn mint(&self) -> Pubkey {
        match self {
            TokenType::Hnt => HNT_MINT,
            TokenType::Iot => IOT_MINT,
            TokenType::Mobile => MOBILE_MINT,
        }
    }

    pub fn registrar(&self) -> Pubkey {
        match self {
            TokenType::Hnt => HNT_REGISTRAR,
            TokenType::Iot => IOT_REGISTRAR,
            TokenType::Mobile => MOBILE_REGISTRAR,
        }
    }

    pub fn decimals(&self) -> i32 {
        match self {
            TokenType::Hnt => 8,
            TokenType::Iot => 6,
            TokenType::Mobile => 6,
        }
    }

    pub fn max_supply(&self) -> f64 {
        match self {
            TokenType::Hnt => 223_000_000f64,
            TokenType::Iot => 200_000_000_000f64,
            TokenType::Mobile => 230_000_000_000f64,
        }
    }
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
    Pubkey::from_str(&address)
        .map(helium_crypto::PublicKey::from)
        .map(|pk| pk.to_string())
        .ok()
}

fn maybe_convert_to_solana(address: String) -> Option<String> {
    helium_crypto::PublicKey::from_str(&address)
        .and_then(Pubkey::try_from)
        .map(|pk| pk.to_string())
        .ok()
}
