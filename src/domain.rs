use std::io::Read;

use serde::Deserialize;
use serde_json::Value;

mod buses;
mod coordinates;
mod location;
mod ride;
mod route_direction;
mod schedule;
mod stops;
mod terminal;

pub use buses::Bus;
#[allow(unused_imports)]
pub use coordinates::{Coordinates, Latitude, Longitude};
pub use location::Location;
pub use ride::Ride;
pub use route_direction::RouteDirection;
pub use schedule::Schedule;
pub use stops::Stop;
pub use terminal::Terminal;

pub const BUS_ENDPOINT: &str = "https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/Bus!A1:Q100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak";
pub const SCHEDULE_ENDPOINT: &str = "https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/BusOperate!A1:Q100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak";
pub const STOP_ENDPOINT: &str = "https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/BusStop!A1:100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak";

#[cfg(test)]
macro_rules! test_data {
    ($input:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/",
            $input,
            ".json"
        ))
    };
}

#[cfg(test)]
pub const TEST_BUSES: &[u8] = test_data!("buses");

#[cfg(test)]
pub const TEST_SCHEDULE: &[u8] = test_data!("schedule");

#[cfg(test)]
pub const TEST_STOPS: &[u8] = test_data!("stops");

#[cfg(test)]
#[macro_export]
macro_rules! test_parse {
    ($t:ty, $input:ident, $expected:expr) => {
        #[test]
        fn test_parse() {
            let list = $crate::domain::parse_list::<_, $t>($input).expect("Parsed list");
            assert_eq!($expected, list.len());
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! test_fetch {
    ($t:ty, $endpoint:ident, $expected:expr) => {
        #[test]
        #[ignore]
        fn test_fetch() {
            let list = $crate::domain::fetch::<$t>($endpoint).expect("Fetched list");
            assert_eq!($expected, list.len());
        }
    };
}

pub fn parse_list<R: Read, T>(input: R) -> anyhow::Result<Vec<T>>
where
    T: for<'a> TryFrom<&'a Value, Error = anyhow::Error>,
{
    #[derive(Debug, Deserialize)]
    struct Input {
        #[serde(rename = "range")]
        #[serde(skip)]
        _range: String,
        #[serde(rename = "majorDimension")]
        #[serde(skip)]
        _major_dimension: String,
        values: Vec<Value>,
    }

    Ok(serde_json::from_reader::<_, Input>(input)?
        .values
        .iter()
        .skip(1) // Skip "header" row
        .map(TryInto::try_into)
        .filter_map(Result::ok)
        .collect())
}

fn fetch<T>(endpoint: &str) -> anyhow::Result<Vec<T>>
where
    T: for<'a> TryFrom<&'a Value, Error = anyhow::Error>,
{
    parse_list(ureq::get(endpoint).call()?.into_reader())
}

pub fn fetch_buses() -> anyhow::Result<Vec<Bus>> {
    fetch(BUS_ENDPOINT)
}

pub fn fetch_shedule() -> anyhow::Result<Vec<Schedule>> {
    fetch(SCHEDULE_ENDPOINT)
}

pub fn fetch_stops() -> anyhow::Result<Vec<Stop>> {
    fetch(STOP_ENDPOINT)
}
