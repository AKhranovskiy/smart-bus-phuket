use std::{collections::HashMap, iter::once};

use itertools::Itertools;

use crate::domain::{Coordinates, RouteDirection, Stop, Terminal};

pub type StopProximity<'a> = (&'a Stop, f64);

pub struct RouteService {
    routes: HashMap<RouteDirection, Vec<Stop>>,
}

impl RouteService {
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
                stops.iter().map(|s| s.name.as_str()).join(" > ")
            );
        }
    }

    #[test]
    fn locate() {
        let sut = RouteService::new(parse_list::<_, Stop>(TEST_STOPS).unwrap());

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
