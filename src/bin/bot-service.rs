use std::net::SocketAddr;

use clap::{Arg, Command};
use mongodb::{bson::doc, Client, Database, options::ClientOptions};
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("bot-service")
        .override_help("Part of myblog-api listens for the incoming event message from LINE messaging")
        .version("3.0.0")
        .arg(
            Arg::new("listen-address")
                .default_value("[::1]:8084")
                .help("Specify the host/IP and port to which HTTP server binds for listening")
                .long("listen-address")
                .takes_value(true),
        )
        .arg(
            Arg::new("mongodb-uri")
                .help("Specify URI which can be used to create a MongoDB instance")
                .long("mongodb-uri")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("line-channel-id")
                .help("Specify LINE channel ID which enabled messaging API")
                .long("line-channel-id")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("line-channel-secret")
                .help("Specify LINE channel secret which enabled messaging API")
                .long("line-channel-secret")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let addr: SocketAddr = matches.value_of("listen-address").unwrap().parse().unwrap();
    let database = connect_mongodb(
        matches.value_of("mongodb-uri").unwrap(),
        &"beta_nomkhonwaan_com",
    ).await?;

    println!("bot-service listening on {}", addr);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(addr)
        .await;

    Ok(())
}


/// Perform a database connection to MongoDB.
async fn connect_mongodb(uri: &str, database: &str) -> Result<Database, mongodb::error::Error> {
    let client_options = ClientOptions::parse(uri).await?;
    let client = Client::with_options(client_options)?;

    match client
        .database(database)
        .run_command(doc! {"ping": 1}, None)
        .await
    {
        Ok(_) => Ok(client.database(database)),
        Err(e) => Err(e),
    }
}