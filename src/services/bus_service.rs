use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};

use crate::domain::Bus;

use super::FetchService;

type CarLicense = String;

pub struct BusService {
    #[allow(dead_code)]
    fetch_service: Arc<FetchService>,
    current_version: AtomicU64,
    buses: RwLock<HashMap<CarLicense, Bus>>,
}

impl BusService {
    pub fn new(fetch_service: Arc<FetchService>) -> Self {
        Self {
            buses: RwLock::default(),
            current_version: AtomicU64::default(),
            fetch_service,
        }
    }

    pub fn operate_position(&self, car_license: &str) -> Option<String> {
        self.update_if_neeeded();
        self.buses
            .read()
            .unwrap()
            .get(car_license)
            .map(|b| b.operate_position.clone())
    }

    pub fn number_of_buses(&self) -> usize {
        self.buses.read().unwrap().len()
    }

    fn update_if_neeeded(&self) {
        if self.current_version.load(Ordering::Acquire) == self.fetch_service.version() {
            return;
        }

        let buses = self
            .fetch_service
            .buses()
            .into_iter()
            .map(|bus| (bus.licence_plate_no.clone(), bus))
            .collect();

        if self.current_version.load(Ordering::Acquire) == self.fetch_service.version() {
            return;
        }

        *self.buses.write().unwrap() = buses;
        self.current_version
            .store(self.fetch_service.version(), Ordering::Relaxed);

        println!(
            "Buses updated, version {}",
            self.current_version.load(Ordering::Acquire)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operate_position() {
        let sut = BusService::new(Arc::new(FetchService::for_tests()));
        assert_eq!(sut.operate_position("10-1152").as_deref(), Some("Bus7"));
    }
}
