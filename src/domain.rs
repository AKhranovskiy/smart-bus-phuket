use std::io::Read;

use serde::Deserialize;
use serde_json::Value;

mod buses;
mod coordinates;
mod location;
mod routes;
mod stops;
mod terminal;

pub use coordinates::Coordinates;
pub use location::Location;
pub use stops::Stop;

use terminal::Terminal;

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

#[allow(dead_code)]
pub fn fetch<T>(endpoint: &str) -> anyhow::Result<Vec<T>>
where
    T: for<'a> TryFrom<&'a Value, Error = anyhow::Error>,
{
    parse_list(ureq::get(endpoint).call()?.into_reader())
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
