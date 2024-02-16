use std::{
    collections::HashMap,
    env::args,
    sync::{Arc, Mutex},
};

use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Event, Payload};
use tokio::signal;

mod domain;
mod services;

use services::{BusService, ConfigService, FetchService, RideService, RouteService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ConfigService::new()?;

    if args().nth(1).as_deref() == Some("fetch") {
        return fetch_test_data(&config);
    }

    let fetch_service = Arc::new(FetchService::new(config.clone())?);
    let bus_service = Arc::new(BusService::new(fetch_service.clone()));
    let ride_service = Arc::new(RideService::new(fetch_service.clone()));
    let route_service = Arc::new(RouteService::new(fetch_service.clone()));

    let bus_lru = Arc::new(Mutex::new(HashMap::with_capacity(
        bus_service.number_of_buses(),
    )));

    ClientBuilder::new(config.app_socket())
        .namespace("/")
        .on_any(move |event, payload, _client| {
            let bus_service = bus_service.clone();
            let ride_service = ride_service.clone();
            let route_service = route_service.clone();
            let bus_lru = bus_lru.clone();
            async move {
                match event {
                    Event::Connect => println!("Connected"),
                    Event::Close => println!("Disconnected"),
                    Event::Error => eprintln!("Error: {payload:?}"),
                    Event::Message => println!("Message: {payload:?}"),
                    Event::Custom(custom) => match custom.as_ref() {
                        "sub_gps" => match payload {
                            Payload::Binary(bin) => println!("{custom} bin: {bin:?}"),
                            Payload::String(value) => {
                                let location =
                                    match serde_json::from_str::<crate::domain::Location>(&value) {
                                        Ok(value) => value,
                                        Err(err) => {
                                            eprintln!("ERROR Failed to parse, {err:#}\n{value}");
                                            return;
                                        }
                                    };

                                if let Some(last_date_time) = bus_lru.lock().unwrap().insert(location.car_license.clone(), location.date_time) {
                                    if last_date_time == location.date_time {
                                        // duplicating message, skip
                                        return
                                    }
                                }

                                let Some(bus) = bus_service.operate_position(&location.car_license)
                                else {
                                    eprintln!(
                                        "WARN Non-operating bus, license={}",
                                        location.car_license
                                    );
                                    return;
                                };

                                let Some(ride) = ride_service.get(bus, location.date_time.time())
                                else {
                                    eprintln!(
                                        "WARN Non-operating bus, position={}, license={}",
                                        bus, location.car_license
                                    );
                                    return;
                                };

                                if let Some((prev, next)) = route_service.locate(ride.direction(), location.coordinates) {
                                println!(
                                    "{}\t{}\t{} => {}, {}m from {} => {}m to {}, speed={}kmh, heading={}°, altitude={}m",
                                    location.date_time,
                                    ride.name, ride.start, ride.stop,
                                    prev.coordinates.distance_to(location.coordinates), prev.name,
                                    next.coordinates.distance_to(location.coordinates), next.name,
                                    location.speed, location.heading.0, location.altitude
                                );
                                } else {
                                eprintln!(
                                    "WARN {}\t{} => {}, can't match location {}",
                                    ride.name, ride.start, ride.stop,location.coordinates
                                );
                                }
                            }
                        },
                        _ => println!("{custom}: {payload:?}"),
                    },
                }
            }
            .boxed()
        })
        .connect()
        .await
        .expect("Connection failed");

    signal::ctrl_c().await?;

    Ok(())
}

fn fetch_test_data(config: &ConfigService) -> anyhow::Result<()> {
    println!("Fetching test data");

    std::io::copy(
        &mut ureq::get(config.buses_url()).call()?.into_reader(),
        &mut std::fs::File::create("data/buses.json")?,
    )?;
    println!("Buses OK");

    std::io::copy(
        &mut ureq::get(config.schedule_url()).call()?.into_reader(),
        &mut std::fs::File::create("data/schedule.json")?,
    )?;
    println!("Schedule OK");

    std::io::copy(
        &mut ureq::get(config.stops_url()).call()?.into_reader(),
        &mut std::fs::File::create("data/stops.json")?,
    )?;
    println!("Stops OK");

    Ok(())
}
