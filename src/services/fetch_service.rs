use crate::domain::{fetch_buses, fetch_shedule, fetch_stops, Bus, Schedule, Stop};

use super::ConfigService;

pub struct FetchService {
    #[allow(dead_code)]
    config: ConfigService,
    buses: Vec<Bus>,
    schedule: Vec<Schedule>,
    stops: Vec<Stop>,
}

impl FetchService {
    pub fn new(config: ConfigService) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            buses: fetch_buses()?,
            schedule: fetch_shedule()?,
            stops: fetch_stops()?,
        })
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        use crate::domain::{parse_list, TEST_BUSES, TEST_SCHEDULE, TEST_STOPS};

        Self {
            config: ConfigService::new().unwrap(),
            buses: parse_list(TEST_BUSES).unwrap(),
            schedule: parse_list(TEST_SCHEDULE).unwrap(),
            stops: parse_list(TEST_STOPS).unwrap(),
        }
    }

    pub fn buses(&self) -> Vec<Bus> {
        self.buses.clone()
    }
    pub fn schedule(&self) -> Vec<Schedule> {
        self.schedule.clone()
    }
    pub fn stops(&self) -> Vec<Stop> {
        self.stops.clone()
    }
}
