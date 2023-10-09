use std::{fmt::Display, str::FromStr};

use anyhow::bail;
use serde::Deserialize;

use super::Stop;
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
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

impl Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Airport => f.write_str("Airport"),
            Self::Rawai => f.write_str("Rawai"),
            Self::Kata => f.write_str("Kata"),
            Self::Patong => f.write_str("Patong"),
        }
    }
}

impl Terminal {
    pub const fn stop_name(self) -> &'static str {
        match self {
            Self::Airport => "Phuket Airport",
            Self::Rawai => "Rawai Beach",
            Self::Kata => "Kata Palm",
            Self::Patong => "Bangla Patong",
        }
    }

    pub fn stop(self, stops: &[Stop]) -> Stop {
        stops
            .iter()
            .find(|s| s.name == self.stop_name())
            .unwrap()
            .clone()
    }
}
