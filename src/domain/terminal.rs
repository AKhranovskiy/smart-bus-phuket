use std::str::FromStr;

use anyhow::bail;
use serde::Deserialize;
#[derive(Debug, Copy, Clone, Deserialize)]
pub enum Terminal {
    Airport,
    Rawai,
    Kata,
    Patong,
}

impl FromStr for Terminal {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Airport" => Ok(Self::Airport),
            "Rawai" => Ok(Self::Rawai),
            "Kata" => Ok(Self::Kata),
            "Patong" => Ok(Self::Patong),
            _ => bail!("unknown terminal stop: {}", s),
        }
    }
}
