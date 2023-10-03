use std::borrow::Cow;

use chrono::NaiveDateTime;
use serde::{de, Deserialize, Deserializer};

use crate::domain::coordinates::Heading;

use super::Coordinates;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Location {
    #[serde(rename = "deviceno")]
    pub device_number: String,
    #[serde(flatten)]
    pub coordinates: Coordinates,
    #[serde(rename = "state")]
    pub state: u32,
    #[serde(rename = "speed")]
    speed: u32,
    #[serde(rename = "direction")]
    heading: Heading,
    #[serde(rename = "altitude")]
    pub altitude: u32,
    #[serde(rename = "dateTime")]
    #[serde(deserialize_with = "deserialize_naive_dt")]
    pub date_time: NaiveDateTime,
    #[serde(rename = "vid")]
    pub vehicle_id: usize,
    #[serde(rename = "carlicense")]
    pub car_license: String,
    #[serde(rename = "groupName")]
    pub group_name: String,
}

fn deserialize_naive_dt<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    NaiveDateTime::parse_from_str(
        Cow::<&str>::deserialize(deserializer)?.as_ref(),
        "%Y-%m-%d %H:%M:%S",
    )
    .map_err(de::Error::custom)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn test_de() {
        const INPUT: &str = r#"{"deviceno":"008800AB63","lat":"7.882165","lng":"98.359084","state":1,"speed":52,"direction":53.2,"altitude":35,"dateTime":"2023-10-03 20:43:16","vid":251,"carlicense":"10-1155","groupName":"Phuket Smart Bus"}"#;
        let location: Location = serde_json::from_str(INPUT).expect("Parsed location");

        assert_eq!(
            location,
            Location {
                device_number: "008800AB63".to_string(),
                coordinates: Coordinates::new(98.359_084.into(), 7.882_165.into(),),
                state: 1,
                speed: 52,
                heading: 53.2.into(),
                altitude: 35,
                date_time: NaiveDate::from_ymd_opt(2023, 10, 3)
                    .unwrap()
                    .and_hms_opt(20, 43, 16)
                    .unwrap(),
                vehicle_id: 251,
                car_license: "10-1155".to_string(),
                group_name: "Phuket Smart Bus".to_string(),
            }
        );
    }
}
