use std::collections::HashMap;

use crate::domain::Bus;

type CarLicense = String;

#[derive(Debug)]
pub struct BusService {
    buses: HashMap<CarLicense, Bus>,
}

impl BusService {
    pub fn new(buses: Vec<Bus>) -> Self {
        Self {
            buses: buses
                .into_iter()
                .map(|bus| (bus.licence_plate_no.clone(), bus))
                .collect(),
        }
    }

    pub fn operate_position(&self, car_license: &str) -> Option<&str> {
        self.buses
            .get(car_license)
            .map(|b| b.operate_position.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{parse_list, TEST_BUSES};

    use super::*;

    #[test]
    fn operate_position() {
        let buses = parse_list(TEST_BUSES).unwrap();
        let sut = BusService::new(buses);

        assert_eq!(sut.operate_position("10-1152"), Some("Bus7"));
    }
}
