use std::sync::Arc;

use domain::fetch;
use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Event, Payload};
use services::{BusService, RideService, RouteService};
use tokio::signal;

mod domain;
mod lookup;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let source = "https://smartbus-7lpin5zc7a-as.a.run.app";

    let bus_service = Arc::new(BusService::new(fetch(domain::BUS_ENDPOINT)?));
    let ride_service = Arc::new(RideService::new(fetch(domain::SCHEDULE_ENDPOINT)?));
    let route_service = Arc::new(RouteService::new(fetch(domain::STOP_ENDPOINT)?));

    ClientBuilder::new(source)
        .namespace("/")
        .on_any(move |event, payload, _client| {
            let bus_service = bus_service.clone();
            let ride_service = ride_service.clone();
            let route_service = route_service.clone();
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
                                    location.date_time, ride.name, ride.start, ride.stop, prev.1, prev.0.name, next.1, next.0.name,
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
