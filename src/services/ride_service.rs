use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};

use chrono::NaiveTime;
use itertools::Itertools;
use rangemap::RangeMap;

use crate::domain::Ride;

use super::FetchService;

pub struct RideService {
    rides: RwLock<HashMap<String, RangeMap<NaiveTime, crate::domain::Ride>>>,
    fetch_service: Arc<FetchService>,
    current_version: AtomicU64,
}

impl RideService {
    pub fn new(fetch_service: Arc<FetchService>) -> Self {
        Self {
            fetch_service,
            current_version: AtomicU64::default(),
            rides: RwLock::default(),
        }
    }

    pub fn get(&self, pos: &str, time: NaiveTime) -> Option<Ride> {
        self.update_if_neeeded();

        self.rides
            .read()
            .unwrap()
            .get(pos)
            .and_then(|r| r.get(&time))
            .cloned()
    }

    fn update_if_neeeded(&self) {
        if self.current_version.load(Ordering::Acquire) == self.fetch_service.version() {
            return;
        }

        let schedule = self.fetch_service.schedule();
        let mut rides = HashMap::new();

        for (position, schedules) in schedule
            .into_iter()
            .into_group_map_by(|s| s.position.clone())
        {
            let mut ranges = RangeMap::new();
            for ride in schedules
                .into_iter()
                .sorted_by_key(|s| s.departure)
                .map(Ride::from)
            {
                ranges.insert(ride.loading..ride.arrival, ride);
            }
            rides.insert(position, ranges);
        }

        if self.current_version.load(Ordering::Acquire) == self.fetch_service.version() {
            return;
        }

        *self.rides.write().unwrap() = rides;
        self.current_version
            .store(self.fetch_service.version(), Ordering::Relaxed);

        println!(
            "Rides updated, version {}",
            self.current_version.load(Ordering::Acquire)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rides() {
        let sut = RideService::new(Arc::new(FetchService::for_tests()));

        let bus6 = sut.get("Bus6", NaiveTime::from_hms_opt(14, 0, 0).unwrap());
        assert!(bus6.is_some());
        assert_eq!("Bus6", bus6.unwrap().name);

        let bus3 = sut.get("Bus3", NaiveTime::from_hms_opt(14, 0, 0).unwrap());
        assert!(bus3.is_none());
    }
}
