use std::{
    collections::{BTreeMap, HashMap},
    iter::once,
    ops::Bound::{Excluded, Unbounded},
};

use itertools::Itertools;

use crate::domain::{Coordinates, Latitude, RouteDirection, Stop, Terminal};

pub type StopProximity<'a> = (&'a Stop, f64);

pub struct RouteService {
    routes: HashMap<RouteDirection, BTreeMap<Latitude, Stop>>,
}

impl RouteService {
    pub fn new(stops: Vec<Stop>) -> Self {
        let terminals = [
            (Terminal::Airport, Terminal::Airport.stop(&stops)),
            (Terminal::Rawai, Terminal::Rawai.stop(&stops)),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let routes = stops
            .into_iter()
            .into_group_map_by(|s| s.route_direction)
            .into_iter()
            .filter(|(terminal, _)| terminal == &Terminal::Airport || terminal == &Terminal::Rawai)
            .map(|(terminal, directed_stops)| {
                let terminal_stop = terminals.get(&terminal).cloned().unwrap();
                let stops = directed_stops
                    .into_iter()
                    .sorted_by_key(|s| s.order)
                    .chain(once(terminal_stop))
                    .map(|s| (s.coordinates.latitude, s))
                    .collect();

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
            let before = stops.range((Unbounded, Excluded(pos.latitude))).next_back();
            let after = stops.range((Excluded(pos.latitude), Unbounded)).next();

            if let (Some(before), Some(after)) = (before, after) {
                let dist_before = pos.distance_to(before.1.coordinates);
                let dist_after = pos.distance_to(after.1.coordinates);

                return match dir {
                    RouteDirection::South => Some(((after.1, dist_after), (before.1, dist_before))),
                    RouteDirection::North => Some(((before.1, dist_before), (after.1, dist_after))),
                };
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::{parse_list, Stop, TEST_STOPS};
    use itertools::Itertools;

    #[test]
    fn routes() {
        let stops = parse_list::<_, Stop>(TEST_STOPS).unwrap();

        let sut = RouteService::new(stops);

        for (direction, stops) in sut.routes {
            println!(
                "{direction}: {}",
                stops.values().map(|s| s.name.as_str()).join(" > ")
            );
        }
    }

    #[test]
    fn locate_on_north_route() {
        let sut = RouteService::new(parse_list::<_, Stop>(TEST_STOPS).unwrap());

        let thawi_wong = Coordinates::new(98.296_32.into(), 7.896_155.into());

        let Some((before, after)) = sut.locate(RouteDirection::North, thawi_wong) else {
            unreachable!()
        };

        println!(
            "{} -{} <=> +{} {}",
            before.0.name, before.1, after.1, after.0.name
        );

        assert_eq!(before.0.name, "Bangla Patong");
        assert_eq!(after.0.name, "Four Point Patong");
    }

    #[test]
    fn locate_on_south_route() {
        let sut = RouteService::new(parse_list::<_, Stop>(TEST_STOPS).unwrap());

        let rat_uthit = Coordinates::new(98.300_77.into(), 7.903_634.into());

        let Some((before, after)) = sut.locate(RouteDirection::South, rat_uthit) else {
            unreachable!()
        };

        println!(
            "{} -{} <=> +{} {}",
            before.0.name, before.1, after.1, after.0.name
        );

        assert_eq!(before.0.name, "Diamond Cliff Resort & Spa");
        assert_eq!(after.0.name, "Indigo Patong");
    }
}
