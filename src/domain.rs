use std::io::Read;

use serde::Deserialize;
use serde_json::Value;

mod buses;
mod routes;
mod stops;
mod terminal;

use terminal::Terminal;

fn parse_list<R: Read, T>(input: R) -> anyhow::Result<Vec<T>>
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
fn fetch<T>(endpoint: &str) -> anyhow::Result<Vec<T>>
where
    T: for<'a> TryFrom<&'a Value, Error = anyhow::Error>,
{
    parse_list(ureq::get(endpoint).call()?.into_reader())
}
