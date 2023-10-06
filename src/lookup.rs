#[cfg(test)]
mod tests {
    use crate::domain::{parse_list, Coordinates, Stop};

    #[test]
    fn sort_stops_by_distance() {
        const STOPS: &[u8] = include_bytes!("domain/stops.json");
        let mut stops = parse_list::<_, Stop>(STOPS).unwrap();

        let point = Coordinates::new(98.321_788.into(), 7.772_087.into());
        stops.sort_unstable_by(|a, b| {
            a.coordinates
                .distance_to(point)
                .partial_cmp(&b.coordinates.distance_to(point))
                .unwrap()
        });

        for stop in &stops {
            println!("{} {}", stop.name, stop.coordinates.distance_to(point));
        }
    }
}
