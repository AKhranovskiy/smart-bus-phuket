#![allow(dead_code)]

use std::{cmp::Ordering, collections::HashMap, fmt::Display, iter::once};

use chrono::NaiveTime;
use itertools::Itertools;

use super::{schedule::Schedule, terminal::Terminal, Coordinates, Stop};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteDirection {
    North,
    South,
}

impl Display for RouteDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::North => f.write_str("North"),
            Self::South => f.write_str("South"),
        }
    }
}

impl From<(Terminal, Terminal)> for RouteDirection {
    fn from(ride: (Terminal, Terminal)) -> Self {
        match ride {
            (Terminal::Airport, _) | (_, Terminal::Rawai) => Self::South,
            (Terminal::Rawai, _) | (_, Terminal::Airport) => Self::North,
            _ => unreachable!("Unknown direction, {} => {}", ride.0, ride.1),
        }
    }
}

impl From<Terminal> for RouteDirection {
    fn from(terminal: Terminal) -> Self {
        match terminal {
            Terminal::Airport => Self::North,
            Terminal::Rawai => Self::South,
            _ => unreachable!("Can't choose direction for {}", terminal),
        }
    }
}

pub struct Routes {
    routes: HashMap<RouteDirection, Vec<Stop>>,
}

type StopProximity<'a> = (&'a Stop, f64);

impl Routes {
    pub fn new(stops: Vec<Stop>) -> Self {
        let terminals = [
            Terminal::Airport,
            Terminal::Rawai,
            Terminal::Kata,
            Terminal::Patong,
        ]
        .into_iter()
        .map(|t| (t, t.stop(&stops)))
        .collect::<HashMap<_, _>>();

        let routes = stops
            .into_iter()
            .into_group_map_by(|s| s.route_direction)
            .into_iter()
            .map(|(terminal, directed_stops)| {
                let stops = directed_stops
                    .into_iter()
                    .sorted_by_key(|s| s.order)
                    .chain(once(terminals.get(&terminal).cloned().unwrap()))
                    .collect::<Vec<_>>();

                (terminal.into(), stops)
            })
            .collect();

        Self { routes }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn locate(
        &self,
        dir: RouteDirection,
        pos: Coordinates,
    ) -> Option<(StopProximity, StopProximity)> {
        if let Some(stops) = self.routes.get(&dir) {
            let sign = |point: Coordinates| match dir {
                RouteDirection::North => f64::from(point.latitude.0 - pos.latitude.0).signum(),
                RouteDirection::South => f64::from(pos.latitude.0 - point.latitude.0).signum(),
            };

            let pair = stops.windows(2).find(|s| {
                let before = &s[0];
                let after = &s[1];
                sign(before.coordinates) + sign(after.coordinates) == 0.0
            });

            if let Some([before, after, ..]) = pair {
                let dist_from = pos.distance_to(before.coordinates);
                let dist_to = pos.distance_to(after.coordinates);
                return Some(((before, dist_from), (after, dist_to)));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Rides {
    rides: HashMap<String, rangemap::RangeMap<NaiveTime, Ride>>,
}

impl Rides {
    pub fn new(schedule: Vec<Schedule>) -> Self {
        let mut rides = HashMap::new();

        for (position, schedules) in schedule
            .into_iter()
            .into_group_map_by(|s| s.position.clone())
        {
            let mut ranges = rangemap::RangeMap::new();
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

impl Terminal {
    pub const fn stop_name(self) -> &'static str {
        match self {
            Self::Airport => "Phuket Airport",
            Self::Rawai => "Rawai Beach",
            Self::Kata => "Kata Palm",
            Self::Patong => "Bangla Patong",
        }
    }

    pub fn stop(self, stops: &[Stop]) -> Stop {
        stops
            .iter()
            .find(|s| s.name == self.stop_name())
            .unwrap()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::{parse_list, schedule::Schedule, Stop};
    use itertools::Itertools;

    #[test]
    fn routes() {
        let stops = parse_list::<_, Stop>(&include_bytes!("stops.json")[..]).unwrap();

        let sut = Routes::new(stops);

        for (direction, stops) in sut.routes {
            println!(
                "{direction}: {}",
                stops.iter().map(|s| s.name.as_str()).join(" > ")
            );
        }
    }

    #[test]
    fn rides() {
        let schedule = parse_list::<_, Schedule>(&include_bytes!("schedule.json")[..]).unwrap();

        let sut = Rides::new(schedule);

        let bus6 = sut.get("Bus6", NaiveTime::from_hms_opt(14, 0, 0).unwrap());
        assert!(bus6.is_some());
        assert_eq!("Bus6", bus6.unwrap().name);

        let bus3 = sut.get("Bus3", NaiveTime::from_hms_opt(14, 0, 0).unwrap());
        assert!(bus3.is_none());
    }

    #[test]
    fn locate() {
        let sut = Routes::new(parse_list::<_, Stop>(&include_bytes!("stops.json")[..]).unwrap());

        let thawi_wong = Coordinates::new(98.296_32.into(), 7.896_155.into());
        let Some((before, after)) = sut.locate(RouteDirection::North, thawi_wong) else {
            unreachable!()
        };
        println!(
            "{} -{} <=> +{} {}",
            before.0.name, before.1, after.1, after.0.name
        );

        let rat_uthit = Coordinates::new(98.300_77.into(), 7.903_634.into());
        let Some((before, after)) = sut.locate(RouteDirection::South, rat_uthit) else {
            unreachable!()
        };
        println!(
            "{} -{} <=> +{} {}",
            before.0.name, before.1, after.1, after.0.name
        );
    }
}
