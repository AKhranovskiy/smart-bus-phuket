#![allow(dead_code)]

use std::{convert::TryInto, io::Read, str::FromStr};

use anyhow::{anyhow, bail, ensure, Result};
use chrono::NaiveTime;
use serde::Deserialize;
use serde_json::Value;

const ENDPOINT: &str = "https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/BusStop!A1:100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak";

pub fn fetch_bus_stops() -> Result<Vec<BusStop>> {
    parse_bus_stops(ureq::get(ENDPOINT).call()?.into_reader())
}

#[derive(Debug, Clone)]
pub struct BusStop {
    order: usize,
    name_th: String,
    name: String,
    description: Option<String>,
    route_direction: Direction,
    longitude: f64,
    latitude: f64,
    schedule: Vec<NaiveTime>,
    icon: String,
    color: String,
    unique_id: Option<usize>,
    image: String,
    map_link: String,
    display: bool,
}

#[derive(Debug, Clone, Copy)]
struct BusDisplay(bool);

impl FromStr for BusDisplay {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self(true)),
            "off" => Ok(Self(false)),
            _ => bail!("unknown bus display: {}", s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Airport,
    Rawai,
}

impl FromStr for Direction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Airport" => Ok(Self::Airport),
            "Rawai" => Ok(Self::Rawai),
            _ => bail!("unknown direction: {}", s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RouteDirection(Direction);

impl FromStr for RouteDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.split(" --> ")
            .nth(1)
            .ok_or_else(|| anyhow!("missing route direction"))
            .and_then(Direction::from_str)
            .map(Self)
    }
}

#[derive(Debug, Clone)]
struct Schedule(Vec<NaiveTime>);

impl FromStr for Schedule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.replace("AM,", " ")
            .replace("PM,", " ")
            .replace("AM", "")
            .replace("PM", "")
            .replace(',', ":")
            .split_ascii_whitespace()
            .map(|time| NaiveTime::parse_from_str(&time[0..4], "%H:%M").map_err(Into::into))
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

#[derive(Debug, Clone)]
struct StopDescription(Option<String>);

impl FromStr for StopDescription {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.split(" Bus Stop ").nth(1).map(ToString::to_string)))
    }
}

fn parse_bus_stops<R: Read>(input: R) -> Result<Vec<BusStop>> {
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

impl TryFrom<&Value> for BusStop {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let Some(array) = value.as_array() else {
            bail!("expected array");
        };

        ensure!(array.len() == 14, "expected 14 items, got {}", array.len());

        let get_str = |index: usize| {
            array[index]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("expected str at position {index}"))
                .map(ToString::to_string)
        };

        Ok(Self {
            order: get_str(0)?.parse()?,
            name_th: get_str(1)?,
            name: get_str(2)?,
            description: StopDescription::from_str(&get_str(3)?)?.0,
            route_direction: RouteDirection::from_str(&get_str(4)?)?.0,
            longitude: get_str(5)?.parse()?,
            latitude: get_str(6)?.parse()?,
            schedule: Schedule::from_str(&get_str(7)?)?.0,
            icon: get_str(8)?,
            color: get_str(9)?,
            unique_id: get_str(10)?.parse().ok(),
            image: get_str(11)?,
            map_link: get_str(12)?,
            display: BusDisplay::from_str(&get_str(13)?)?.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("bus_stop.json");

    #[test]
    fn test_parse_bus_stops() {
        let stops = parse_bus_stops(INPUT).expect("Parsed bus stops");
        assert_eq!(52, stops.len());
    }

    #[test]
    #[ignore]
    fn test_fetch_bus_stops() {
        let stops = fetch_bus_stops().expect("Fetched bus stops");
        assert_eq!(52, stops.len());
    }
}
