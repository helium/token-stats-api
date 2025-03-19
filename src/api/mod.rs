pub mod supply;

use serde::{de::IntoDeserializer, Deserialize};

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
