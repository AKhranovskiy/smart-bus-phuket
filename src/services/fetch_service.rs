use std::sync::{
    atomic::{AtomicU64, Ordering},
    RwLock, RwLockReadGuard,
};

use chrono::{NaiveDateTime, Utc};

use crate::{
    config::Config,
    domain::{fetch, Bus, Schedule, Stop},
};

pub struct FetchService {
    config: Config,
    inner: RwLock<Inner>,
    version: AtomicU64,
}

#[derive(Debug, Default, Clone)]
struct Inner {
    buses: Vec<Bus>,
    schedule: Vec<Schedule>,
    stops: Vec<Stop>,
    last_updated: NaiveDateTime,
}

impl Inner {
    fn fetch(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            buses: fetch(&config.buses_url)?,
            schedule: fetch(&config.schedule_url)?,
            stops: fetch(&config.stops_url)?,
            last_updated: Utc::now().naive_local(),
        })
    }

    #[cfg(test)]
    fn for_tests() -> Self {
        use crate::domain::{parse_list, TEST_BUSES, TEST_SCHEDULE, TEST_STOPS};
        Self {
            buses: parse_list(TEST_BUSES).unwrap(),
            schedule: parse_list(TEST_SCHEDULE).unwrap(),
            stops: parse_list(TEST_STOPS).unwrap(),
            last_updated: Utc::now().naive_local() + chrono::TimeDelta::try_days(365).unwrap(),
        }
    }
}

impl FetchService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            inner: RwLock::default(),
            version: AtomicU64::new(1),
        }
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self {
            config: Config::default(),
            inner: RwLock::new(Inner::for_tests()),
            version: AtomicU64::new(1),
        }
    }

    pub fn buses(&self) -> Vec<Bus> {
        self.inner().buses.clone()
    }
    pub fn schedule(&self) -> Vec<Schedule> {
        self.inner().schedule.clone()
    }
    pub fn stops(&self) -> Vec<Stop> {
        self.inner().stops.clone()
    }
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    fn inner(&self) -> RwLockReadGuard<Inner> {
        self.fetch_if_outdated();
        self.inner.read().unwrap()
    }

    fn fetch_if_outdated(&self) {
        if (Utc::now().naive_local() - self.inner.read().unwrap().last_updated)
            < self.config.update_interval
        {
            return;
        }

        {
            let mut inner_guard = self.inner.write().unwrap();
            // Write lock check
            if (Utc::now().naive_local() - inner_guard.last_updated) <= self.config.update_interval
            {
                return;
            }
            // Postpone other attempts by 1 minute
            inner_guard.last_updated = Utc::now().naive_local() - self.config.update_interval
                + chrono::TimeDelta::try_minutes(1).unwrap();
        }

        println!("Fetching new data");
        match Inner::fetch(&self.config) {
            Ok(inner) => {
                *self.inner.write().unwrap() = inner;
                self.version.fetch_add(1, Ordering::AcqRel);
                println!("Update completed, version {}", self.version());
            }
            Err(err) => {
                self.inner.write().unwrap().last_updated = Utc::now().naive_local()
                    - self.config.update_interval
                    + chrono::TimeDelta::try_minutes(1).unwrap();
                eprintln!("Failed to fetch {err:#}, retry in 1 minute");
            }
        }
    }
}
