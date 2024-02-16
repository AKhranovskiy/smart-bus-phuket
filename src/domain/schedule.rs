#![allow(dead_code)]

use std::str::FromStr;

use anyhow::{anyhow, bail, ensure, Result};
use chrono::NaiveTime;
use serde::Deserialize;
use serde_json::Value;

use super::Terminal;

#[derive(Debug, Clone, Deserialize)]
pub struct Schedule {
    pub position: String,
    pub start: Terminal,
    pub departure: NaiveTime,
    pub color_changed: NaiveTime,
    pub arrival: NaiveTime,
    pub destination: Terminal,
    pub direction: Terminal,
    pub icon: String,
}

#[derive(Debug, Copy, Clone)]
pub struct Direction(Terminal);

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

// impl Direction {
//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         Cow::<&str>::deserialize(deserializer)?
//             .parse()
//             .map_err(de::Error::custom)
//     }
// }

impl TryFrom<&Value> for Schedule {
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
            direction: Direction::from_str(&get_str(6)?)?.0,
            icon: get_str(7)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct SmartBusTime(NaiveTime);

impl FromStr for SmartBusTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.replace("12:00:00 AM", "23:59:59 PM")
            .split_ascii_whitespace()
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
    use crate::{domain::TEST_SCHEDULE, test_parse};

    use super::*;

    test_parse!(Schedule, TEST_SCHEDULE, 34);
}
