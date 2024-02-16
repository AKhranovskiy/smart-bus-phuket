use chrono::NaiveDateTime;

use crate::{
    config::Config,
    domain::{fetch, Bus, Schedule, Stop},
};

pub struct FetchService {
    buses: Vec<Bus>,
    schedule: Vec<Schedule>,
    stops: Vec<Stop>,
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    last_update: NaiveDateTime,
}

impl FetchService {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            buses: fetch(&config.buses_url)?,
            schedule: fetch(&config.schedule_url)?,
            stops: fetch(&config.stops_url)?,
            config,
            last_update: NaiveDateTime::default(),
        })
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        use chrono::Utc;

        use crate::domain::{parse_list, TEST_BUSES, TEST_SCHEDULE, TEST_STOPS};

        Self {
            config: Config::default(),
            buses: parse_list(TEST_BUSES).unwrap(),
            schedule: parse_list(TEST_SCHEDULE).unwrap(),
            stops: parse_list(TEST_STOPS).unwrap(),
            last_update: Utc::now().naive_local(),
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
