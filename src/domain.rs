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
pub use coordinates::Coordinates;
pub use location::Location;
pub use ride::Ride;
pub use route_direction::RouteDirection;
pub use schedule::Schedule;
pub use stops::Stop;
pub use terminal::Terminal;

pub use buses::ENDPOINT as BUS_ENDPOINT;
pub use schedule::ENDPOINT as SCHEDULE_ENDPOINT;
pub use stops::ENDPOINT as STOP_ENDPOINT;

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

    serde_json::from_reader::<_, Input>(input)?
        .values
        .iter()
        .skip(1) // Skip "header" row
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()
}

pub fn fetch<T>(endpoint: &str) -> anyhow::Result<Vec<T>>
where
    T: for<'a> TryFrom<&'a Value, Error = anyhow::Error>,
{
    parse_list(ureq::get(endpoint).call()?.into_reader())
}

#[cfg(test)]
#[macro_export]
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

#[cfg(test)]
pub const TEST_BUSES: &[u8] = test_data!("buses");

#[cfg(test)]
pub const TEST_SCHEDULE: &[u8] = test_data!("schedule");

#[cfg(test)]
pub const TEST_STOPS: &[u8] = test_data!("stops");
