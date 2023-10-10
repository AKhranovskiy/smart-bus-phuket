use std::{
    collections::BTreeMap,
    iter::once,
    ops::Bound::{Excluded, Included, Unbounded},
};

use itertools::Itertools;

use crate::domain::{Coordinates, Latitude, RouteDirection, Stop, Terminal};

pub struct RouteService {
    north: BTreeMap<Latitude, Stop>,
    south: BTreeMap<Latitude, Stop>,
}

impl RouteService {
    pub fn new(stops: &[Stop]) -> Self {
        let build = |terminal: Terminal| {
            stops
                .iter()
                .filter(|s| s.route_direction == terminal)
                .cloned()
                .sorted_by_key(|s| s.order)
                .chain(once(terminal.stop(stops)))
                .map(|s| (s.coordinates.latitude, s))
                .collect()
        };

        Self {
            north: build(Terminal::Airport),
            south: build(Terminal::Rawai),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn locate(&self, dir: RouteDirection, pos: Coordinates) -> Option<(&Stop, &Stop)> {
        let stops = match dir {
            RouteDirection::North => &self.north,
            RouteDirection::South => &self.south,
        };

        let mut previous = stops
            .range((Unbounded, Excluded(pos.latitude)))
            .next_back()
            .map(|(_, s)| s);

        let mut next = stops
            .range((Included(pos.latitude), Unbounded))
            .next()
            .map(|(_, s)| s);

        if dir == RouteDirection::South {
            std::mem::swap(&mut previous, &mut next);
        }

        previous.zip(next)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::domain::{parse_list, TEST_STOPS};

    use super::*;

    #[test]
    fn routes() {
        let sut = RouteService::new(&parse_list(TEST_STOPS).unwrap());

        println!(
            "NORTH: {}",
            sut.north.values().map(|s| s.name.as_str()).join(" > ")
        );
        println!(
            "SOUTH: {}",
            sut.south
                .values()
                .rev()
                .map(|s| s.name.as_str())
                .join(" > ")
        );
    }

    #[test]
    fn locate_on_north_route() {
        let sut = RouteService::new(&parse_list(TEST_STOPS).unwrap());

        let thawi_wong = Coordinates::new(98.296_32.into(), 7.896_155.into());

        let Some((prev, next)) = sut.locate(RouteDirection::North, thawi_wong) else {
            unreachable!()
        };

        println!(
            "{} -{} <=> +{} {}",
            prev.name,
            prev.coordinates.distance_to(thawi_wong),
            next.coordinates.distance_to(thawi_wong),
            next.name
        );

        assert_eq!(prev.name, "Bangla Patong");
        assert_eq!(next.name, "Four Point Patong");
    }

    #[test]
    fn locate_on_south_route() {
        let sut = RouteService::new(&parse_list(TEST_STOPS).unwrap());

        let rat_uthit = Coordinates::new(98.300_77.into(), 7.903_634.into());

        let Some((prev, next)) = sut.locate(RouteDirection::South, rat_uthit) else {
            unreachable!()
        };

        println!(
            "{} -{} <=> +{} {}",
            prev.name,
            prev.coordinates.distance_to(rat_uthit),
            next.coordinates.distance_to(rat_uthit),
            next.name
        );

        assert_eq!(prev.name, "Diamond Cliff Resort & Spa");
        assert_eq!(next.name, "Indigo Patong");
    }

    #[test]
    fn locate_terminal_airport_north() {
        let sut = RouteService::new(&parse_list(TEST_STOPS).unwrap());

        let airport = Coordinates::new(98.306_55.into(), 8.108_46.into());

        let Some((prev, next)) = sut.locate(RouteDirection::North, airport) else {
            unreachable!()
        };

        println!(
            "{} -{} <=> +{} {}",
            prev.name,
            prev.coordinates.distance_to(airport),
            next.coordinates.distance_to(airport),
            next.name
        );

        assert_eq!(prev.name, "Thalang Public Health Office");
        assert_eq!(next.name, "Phuket Airport");
    }

    #[test]
    fn locate_terminal_airport_south() {
        let sut = RouteService::new(&parse_list(TEST_STOPS).unwrap());

        let airport = Coordinates::new(98.306_55.into(), 8.108_46.into());

        let Some((prev, next)) = sut.locate(RouteDirection::South, airport) else {
            unreachable!()
        };

        println!(
            "{} -{} <=> +{} {}",
            prev.name,
            prev.coordinates.distance_to(airport),
            next.coordinates.distance_to(airport),
            next.name
        );

        assert_eq!(prev.name, "Phuket Airport");
        assert_eq!(next.name, "Thalang Public Health Office");
    }
}
