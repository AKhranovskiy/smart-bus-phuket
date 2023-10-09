#![allow(dead_code)]

use std::str::FromStr;

use anyhow::{anyhow, bail, ensure, Result};
use chrono::NaiveTime;
use serde_json::Value;

use super::{Coordinates, Terminal};

#[derive(Debug, Clone)]
pub struct Stop {
    pub order: usize,
    pub name_th: String,
    pub name: String,
    pub description: Option<String>,
    pub route_direction: Terminal,
    pub coordinates: Coordinates,
    pub schedule: Vec<NaiveTime>,
    pub icon: String,
    pub color: String,
    pub unique_id: Option<usize>,
    pub image: String,
    pub map_link: String,
    pub display: bool,
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
struct RouteDirection(Terminal);

impl FromStr for RouteDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.split(" --> ")
            .nth(1)
            .ok_or_else(|| anyhow!("missing route direction"))
            .and_then(Terminal::from_str)
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

impl TryFrom<&Value> for Stop {
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
            name_th: get_str(1)?.trim().to_string(),
            name: get_str(2)?.trim().to_string(),
            description: StopDescription::from_str(&get_str(3)?)?.0,
            route_direction: RouteDirection::from_str(&get_str(4)?)?.0,
            coordinates: Coordinates::new(
                // JSON messes lat/lng.
                get_str(6)?.parse::<f32>()?.into(),
                get_str(5)?.parse::<f32>()?.into(),
            ),
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
    use crate::{
        domain::{STOP_ENDPOINT, TEST_STOPS},
        test_fetch, test_parse,
    };

    use super::*;

    test_parse!(Stop, TEST_STOPS, 52);
    test_fetch!(Stop, STOP_ENDPOINT, 52);
}
