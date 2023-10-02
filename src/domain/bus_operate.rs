#![allow(dead_code)]

use std::{borrow::Cow, convert::TryInto, io::Read, str::FromStr};

use anyhow::{anyhow, bail, ensure, Result};
use chrono::NaiveTime;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

const ENDPOINT: &str = "https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/BusOperate!A1:Q100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak";

pub fn fetch_bus_operations() -> Result<Vec<BusOperation>> {
    parse_bus_operations(ureq::get(ENDPOINT).call()?.into_reader())
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusOperation {
    position: String,
    start: TerminalStop,
    departure: NaiveTime,
    color_changed: NaiveTime,
    arrival: NaiveTime,
    destination: TerminalStop,
    #[serde(deserialize_with = "Direction::deserialize")]
    direction: Direction,
    icon: String,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum TerminalStop {
    Airport,
    Rawai,
    Kata,
    Patong,
}

impl FromStr for TerminalStop {
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

#[derive(Debug, Copy, Clone)]
pub struct Direction(TerminalStop);

impl FromStr for Direction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.split_ascii_whitespace()
            .nth(1)
            .ok_or_else(|| anyhow!("expected Stop name"))
            .and_then(str::parse)
            .map(Self)
    }
}

impl Direction {
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Cow::<&str>::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

fn parse_bus_operations<R: Read>(input: R) -> Result<Vec<BusOperation>> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Input {
        #[serde(skip)]
        range: String,
        #[serde(skip)]
        major_dimension: String,
        values: Vec<Value>,
    }

    serde_json::from_reader::<_, Input>(input)?
        .values
        .iter()
        .skip(1) // Skip "header" row
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()
}

impl TryFrom<&Value> for BusOperation {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let Some(array) = value.as_array() else {
            bail!("expected array");
        };

        ensure!(array.len() == 8, "expected 8 items, got {}", array.len());

        let get_str = |index: usize| {
            array[index]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("expected str at position {index}"))
                .map(ToString::to_string)
        };

        Ok(Self {
            position: get_str(0)?,
            start: get_str(1)?.parse()?,
            departure: *SmartBusTime::from_str(&get_str(2)?)?.as_ref(),
            color_changed: *SmartBusTime::from_str(&get_str(3)?)?.as_ref(),
            arrival: *SmartBusTime::from_str(&get_str(4)?)?.as_ref(),
            destination: get_str(5)?.parse()?,
            direction: get_str(6)?.parse()?,
            icon: get_str(7)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct SmartBusTime(NaiveTime);

impl FromStr for SmartBusTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.split_ascii_whitespace()
            .nth(0)
            .ok_or_else(|| anyhow!("missing input"))
            .and_then(|s| NaiveTime::parse_from_str(s, "%T").map_err(Into::into))
            .map(Self)
    }
}

impl AsRef<NaiveTime> for SmartBusTime {
    fn as_ref(&self) -> &NaiveTime {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("bus_operate.json");

    #[test]
    fn test_parse_bus_operations() {
        let ops = parse_bus_operations(INPUT).expect("Parsed bus operations");
        assert_eq!(34, ops.len());
    }

    #[test]
    #[ignore]
    fn test_fetch_bus_operations() {
        let ops = fetch_bus_operations().expect("Fetched bus operations");
        assert_eq!(34, ops.len());
    }
}
