#![allow(dead_code)]

use std::convert::TryInto;

use anyhow::{bail, ensure, Result};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use serde_json::Value;

const ENDPOINT: &str = "https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/Bus!A1:Q100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak";

#[derive(Debug, Clone)]
pub struct Bus {
    no: u8,
    licence_plate_no: String,
    bus_id: String,
    _icon: String,
    service_status: ServiceStatus,
    direction: Direction,
    operate_position: String,
    _a: String,
    _b: String,
    _c: String,
    _d: String,
    _e: String,
    _f: String,
    _concat: String,
    _run: String,
    date: NaiveDate,
    time: NaiveTime,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Suspend,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum Direction {
    #[serde(rename = "0")]
    A,
    #[serde(rename = "1")]
    B,
}

fn parse_buses(input: &[u8]) -> Result<Vec<Bus>> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Input {
        range: String,
        major_dimension: String,
        values: Vec<Value>,
    }

    serde_json::from_slice::<Input>(input)?
        .values
        .iter()
        .skip(1) // Skip "header" row
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()
}

impl TryFrom<&Value> for Bus {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let Some(array) = value.as_array() else {
            bail!("expected array");
        };

        ensure!(array.len() == 17, "expected 17 items, got {}", array.len());

        let get_str = |index: usize| {
            let s = array[index]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("expected str at position {index}"))
                .map(ToString::to_string);
            s
        };

        dbg!(&array);
        Ok(Self {
            no: get_str(0)?.parse()?,
            licence_plate_no: get_str(1)?,
            bus_id: get_str(2)?,
            _icon: get_str(3)?,
            service_status: serde_json::from_value(array[4].clone())?,
            direction: serde_json::from_value(array[5].clone())?,
            operate_position: get_str(6)?,
            _a: get_str(7)?,
            _b: get_str(8)?,
            _c: get_str(9)?,
            _d: get_str(10)?,
            _e: get_str(11)?, // 10
            _f: get_str(12)?,
            _concat: get_str(13)?,
            _run: get_str(14)?,
            date: NaiveDate::parse_from_str(&get_str(15)?, "%d/%m/%Y")?,
            time: NaiveTime::parse_from_str(&get_str(16)?, "%r")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("bus.json");

    #[test]
    fn test_parse_buses() {
        let buses = parse_buses(INPUT).expect("Parsed buses");

        dbg!(&buses);
        assert_eq!(11, buses.len());
    }
}
