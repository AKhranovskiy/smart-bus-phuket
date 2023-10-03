use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Event, Payload};
use tokio::signal;

mod domain;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let source = "https://smartbus-7lpin5zc7a-as.a.run.app";

    ClientBuilder::new(source)
        .namespace("/")
        .on_any(|event, payload, _client| {
            async move {
                match event {
                    Event::Connect => println!("Connected"),
                    Event::Close => println!("Disconnected"),
                    Event::Error => eprintln!("Error: {payload:?}"),
                    Event::Message => println!("Message: {payload:?}"),
                    Event::Custom(custom) => match custom.as_ref() {
                        "sub_gps" => match payload {
                            Payload::Binary(bin) => println!("{custom} bin: {bin:?}"),
                            Payload::String(value) => println!(
                                "{custom}: {:?}",
                                serde_json::from_str::<crate::domain::Location>(&value).unwrap()
                            ),
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
