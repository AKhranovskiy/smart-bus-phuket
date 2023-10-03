use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

macro_rules! wrap_f32_string {
    ($name:ident) => {
        #[serde_as]
        #[derive(Debug, Clone, Copy, Deserialize)]
        #[serde(transparent)]
        pub struct $name(#[serde_as(as = "DisplayFromStr")] f32);
    };
}

macro_rules! wrap_f32 {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Deserialize)]
        pub struct $name(f32);
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
    };
}

wrap_f32_string!(Longitude);
impl_eq!(Longitude);

wrap_f32_string!(Latitude);
impl_eq!(Latitude);

wrap_f32!(Heading);
impl_eq!(Heading);

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
}
