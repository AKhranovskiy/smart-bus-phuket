use std::collections::HashMap;

use chrono::NaiveTime;
use itertools::Itertools;
use rangemap::RangeMap;

use crate::domain::{Ride, Schedule};

pub struct RideService {
    rides: HashMap<String, RangeMap<NaiveTime, crate::domain::Ride>>,
}

impl RideService {
    pub fn new(schedule: Vec<Schedule>) -> Self {
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

        Self { rides }
    }
    pub fn get(&self, pos: &str, time: NaiveTime) -> Option<&Ride> {
        self.rides.get(pos).and_then(|r| r.get(&time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::{parse_list, Schedule, TEST_SCHEDULE};

    #[test]
    fn rides() {
        let schedule = parse_list::<_, Schedule>(TEST_SCHEDULE).unwrap();

        let sut = RideService::new(schedule);

        let bus6 = sut.get("Bus6", NaiveTime::from_hms_opt(14, 0, 0).unwrap());
        assert!(bus6.is_some());
        assert_eq!("Bus6", bus6.unwrap().name);

        let bus3 = sut.get("Bus3", NaiveTime::from_hms_opt(14, 0, 0).unwrap());
        assert!(bus3.is_none());
    }
}
