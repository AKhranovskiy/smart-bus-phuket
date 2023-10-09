use std::{cmp::Ordering, fmt::Display};

use chrono::NaiveTime;

use super::{route_direction::RouteDirection, schedule::Schedule, Terminal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ride {
    pub name: String,
    pub start: Terminal,
    pub stop: Terminal,
    pub loading: NaiveTime,
    pub departure: NaiveTime,
    pub arrival: NaiveTime,
}

impl Ord for Ride {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.name.cmp(&other.name) == Ordering::Equal {
            return self.departure.cmp(&other.departure);
        }
        self.name.cmp(&other.name)
    }
}
impl PartialOrd for Ride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ride {
    pub fn direction(&self) -> RouteDirection {
        RouteDirection::from((self.start, self.stop))
    }
}

impl From<Schedule> for Ride {
    fn from(s: Schedule) -> Self {
        Self {
            name: s.position,
            start: s.start,
            stop: s.destination,
            loading: s.color_changed,
            departure: s.departure,
            arrival: s.arrival,
        }
    }
}

impl Display for Ride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}: {} / {} -> {} / {}",
            self.name, self.departure, self.start, self.arrival, self.stop,
        ))
    }
}
