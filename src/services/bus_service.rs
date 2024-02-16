use std::{collections::HashMap, sync::Arc};

use crate::domain::Bus;

use super::FetchService;

type CarLicense = String;

pub struct BusService {
    #[allow(dead_code)]
    fetch_service: Arc<FetchService>,
    buses: HashMap<CarLicense, Bus>,
}

impl BusService {
    pub fn new(fetch_service: Arc<FetchService>) -> Self {
        let buses = fetch_service
            .buses()
            .into_iter()
            .map(|bus| (bus.licence_plate_no.clone(), bus))
            .collect();

        Self {
            fetch_service,
            buses,
        }
    }

    pub fn operate_position(&self, car_license: &str) -> Option<&str> {
        self.buses
            .get(car_license)
            .map(|b| b.operate_position.as_ref())
    }

    pub fn number_of_buses(&self) -> usize {
        self.buses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operate_position() {
        let sut = BusService::new(Arc::new(FetchService::for_tests()));
        assert_eq!(sut.operate_position("10-1152"), Some("Bus7"));
    }
}
