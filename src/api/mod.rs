pub mod supply;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Hnt,
    Iot,
    Mobile,
}
