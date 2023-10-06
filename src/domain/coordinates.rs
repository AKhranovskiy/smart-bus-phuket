use std::fmt::Display;

use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

macro_rules! wrap_f32_string {
    ($name:ident) => {
        #[serde_as]
        #[derive(Debug, Clone, Copy, Deserialize)]
        #[serde(transparent)]
        pub struct $name(#[serde_as(as = "DisplayFromStr")] pub f32);
    };
}

macro_rules! wrap_f32 {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Deserialize)]
        pub struct $name(pub f32);
    };
}

macro_rules! impl_eq {
    ($name:ident) => {
        impl From<f32> for $name {
            fn from(value: f32) -> Self {
                Self(value)
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                (self.0 - other.0).abs() < f32::EPSILON
            }
        }

        impl Eq for $name {}

        impl PartialEq<f32> for $name {
            fn eq(&self, other: &f32) -> bool {
                (self.0 - other).abs() < f32::EPSILON
            }
        }

        impl AsRef<f32> for $name {
            fn as_ref(&self) -> &f32 {
                &self.0
            }
        }
    };
}

wrap_f32_string!(Longitude);
impl_eq!(Longitude);

wrap_f32_string!(Latitude);
impl_eq!(Latitude);

wrap_f32!(Heading);
impl_eq!(Heading);

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct Coordinates {
    #[serde(rename = "lng")]
    pub longitude: Longitude,
    #[serde(rename = "lat")]
    pub latitude: Latitude,
}

impl Coordinates {
    pub const fn new(longitude: Longitude, latitude: Latitude) -> Self {
        Self {
            longitude,
            latitude,
        }
    }

    pub fn distance_to(self, other: Self) -> f64 {
        geoutils::Location::from(self)
            .haversine_distance_to(&geoutils::Location::from(other))
            .meters()
    }
}

impl Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.6},{:.6}",
            self.longitude.0, self.latitude.0
        ))
    }
}

impl From<Coordinates> for geoutils::Location {
    fn from(value: Coordinates) -> Self {
        Self::new(value.latitude.0, value.longitude.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_de_str() {
        assert_eq!(
            Longitude(7.882_165),
            serde_json::from_str::<Longitude>(r#""7.882165""#).unwrap()
        );
    }

    #[test]
    fn test_de() {
        assert_eq!(
            Heading(53.2),
            serde_json::from_str::<Heading>("53.2").unwrap()
        );
    }
    #[test]
    fn test_display() {
        assert_eq!(
            "98.359085,7.882165",
            Coordinates {
                longitude: Longitude(98.359_085),
                latitude: Latitude(7.882_165)
            }
            .to_string()
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_harvesine_distance() {
        let a: geoutils::Location =
            Coordinates::new(Longitude(98.36212), Latitude(7.892_785)).into();
        let b: geoutils::Location = Coordinates::new(98.32612.into(), 7.77470.into()).into();

        assert_eq!(13716.33, a.haversine_distance_to(&b).meters());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_vyncenty_distance() {
        let a: geoutils::Location =
            Coordinates::new(Longitude(98.36212), Latitude(7.892_785)).into();
        let b: geoutils::Location = Coordinates::new(98.32612.into(), 7.77470.into()).into();

        assert_eq!(13649.882, a.distance_to(&b).unwrap().meters());
    }
}
