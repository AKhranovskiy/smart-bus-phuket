use std::{
    collections::BTreeMap,
    iter::once,
    ops::Bound::{Included, Unbounded},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};

use itertools::Itertools;

use crate::domain::{Coordinates, Latitude, RouteDirection, Stop, Terminal};

use super::FetchService;

pub struct RouteService {
    fetch_service: Arc<FetchService>,
    current_version: AtomicU64,
    inner: RwLock<Inner>,
}

#[derive(Debug, Clone, Default)]
struct Inner {
    north: BTreeMap<Latitude, Stop>,
    south: BTreeMap<Latitude, Stop>,
}

impl RouteService {
    pub fn new(fetch_service: Arc<FetchService>) -> Self {
        Self {
            fetch_service,
            current_version: AtomicU64::default(),
            inner: RwLock::default(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn locate(&self, dir: RouteDirection, pos: Coordinates) -> Option<(Stop, Stop)> {
        self.update_if_neeeded();

        let stops = match dir {
            RouteDirection::North => self.inner.read().unwrap().north.clone(),
            RouteDirection::South => self.inner.read().unwrap().south.clone(),
        };

        let mut previous_it = stops.range((Unbounded, Included(pos.latitude)));
        let mut next_it = stops.range((Included(pos.latitude), Unbounded));

        let (mut previous, mut next) = {
            let (previous, next) = (previous_it.next_back(), next_it.next());
            if previous == next {
                (previous_it.next_back(), previous)
            } else if previous.is_none() {
                (next, next_it.next())
            } else {
                (previous, next)
            }
        };

        if dir == RouteDirection::South {
            std::mem::swap(&mut previous, &mut next);
        }

        previous.map(|s| s.1.clone()).zip(next.map(|s| s.1.clone()))
    }

    fn update_if_neeeded(&self) {
        if self.current_version.load(Ordering::Acquire) == self.fetch_service.version() {
            return;
        }

        let stops = self.fetch_service.stops();

        let build = |terminal: Terminal| {
            stops
                .iter()
                .filter(|s| s.route_direction == terminal)
                .cloned()
                .sorted_by_key(|s| s.order)
                .chain(once(terminal.stop(&stops)))
                .map(|s| (s.coordinates.latitude, s))
                .collect()
        };
        let inner = Inner {
            north: build(Terminal::Airport),
            south: build(Terminal::Rawai),
        };

        if self.current_version.load(Ordering::Acquire) == self.fetch_service.version() {
            return;
        }

        *self.inner.write().unwrap() = inner;
        self.current_version
            .store(self.fetch_service.version(), Ordering::Relaxed);

        println!(
            "Routes updated, version {}",
            self.current_version.load(Ordering::Acquire)
        );
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::domain::Longitude;

    use super::*;

    const AIRPORT: Coordinates = Coordinates::new(Longitude(98.306_55), Latitude(8.108_46));
    const NEAR_AIRPORT: Coordinates = Coordinates::new(Longitude(98.306_55), Latitude(8.102_46));
    const THAWI_WONG: Coordinates = Coordinates::new(Longitude(98.296_32), Latitude(7.896_155));
    const RAT_UTHIT: Coordinates = Coordinates::new(Longitude(98.300_77), Latitude(7.903_634));
    const RAWAI: Coordinates = Coordinates::new(Longitude(98.321_785), Latitude(7.772_087));
    const NEAR_RAWAI: Coordinates = Coordinates::new(Longitude(98.321_785), Latitude(7.782_087));

    fn sut() -> RouteService {
        RouteService::new(Arc::new(FetchService::for_tests()))
    }

    #[test]
    #[ignore]
    fn print_routes() {
        let sut = sut();
        sut.update_if_neeeded();

        println!(
            "NORTH: {}",
            sut.inner
                .read()
                .unwrap()
                .north
                .values()
                .map(|s| s.name.as_str())
                .join(" > ")
        );
        println!(
            "SOUTH: {}",
            sut.inner
                .read()
                .unwrap()
                .south
                .values()
                .rev()
                .map(|s| s.name.as_str())
                .join(" > ")
        );
    }

    #[rstest]
    #[case::south_airport(
        RouteDirection::South,
        AIRPORT,
        "Phuket Airport",
        "Thalang Public Health Office"
    )]
    #[case::south_near_airport(
        RouteDirection::South,
        NEAR_AIRPORT,
        "Phuket Airport",
        "Thalang Public Health Office"
    )]
    #[case::south_rat_uthit(
        RouteDirection::South,
        RAT_UTHIT,
        "Diamond Cliff Resort & Spa",
        "Indigo Patong"
    )]
    #[case::south_near_rawai(RouteDirection::South, NEAR_RAWAI, "Sai Yuan", "Rawai Beach")]
    #[case::south_rawai(RouteDirection::South, RAWAI, "Sai Yuan", "Rawai Beach")]
    #[case::north_rawai(RouteDirection::North, RAWAI, "Rawai Beach", "Sai Yuan")]
    #[case::north_near_rawai(RouteDirection::North, NEAR_RAWAI, "Rawai Beach", "Sai Yuan")]
    #[case::north_thawi_wong(
        RouteDirection::North,
        THAWI_WONG,
        "Bangla Patong",
        "Four Point Patong"
    )]
    #[case::north_near_airport(
        RouteDirection::North,
        NEAR_AIRPORT,
        "Thalang Public Health Office",
        "Phuket Airport"
    )]
    #[case::north_airport(
        RouteDirection::North,
        AIRPORT,
        "Thalang Public Health Office",
        "Phuket Airport"
    )]
    fn locate(
        #[case] direction: RouteDirection,
        #[case] pos: Coordinates,
        #[case] previous_stop_name: &str,
        #[case] next_stop_name: &str,
    ) {
        let sut = sut();

        let Some((prev, next)) = sut.locate(direction, pos) else {
            panic!("Failed to locate the stop")
        };

        println!(
            "{} -{} <=> +{} {}",
            prev.name,
            prev.coordinates.distance_to(pos),
            next.coordinates.distance_to(pos),
            next.name
        );

        assert_eq!(prev.name, previous_stop_name);
        assert_eq!(next.name, next_stop_name);
    }
}
