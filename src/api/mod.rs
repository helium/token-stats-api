pub mod address;
pub mod legacy;
pub mod supply;

use std::str::FromStr;

use helium_crypto::WriteTo;
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
