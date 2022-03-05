use std::net::SocketAddr;

use clap::{Arg, Command};
use mongodb::{bson::doc, Client, Database, options::ClientOptions};
use myblog_proto_rust::myblog::proto::auth::auth_service_server::AuthServiceServer;
use tonic::transport::Server;

use myblog_api::auth::{
    service::MyAuthService,
    user::MongoUserRepository,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("auth-service")
        .override_help("Part of myblog-api provide all authentication and authorization APIs")
        .version("3.0.0")
        .arg(
            Arg::new("listen-address")
                .default_value("[::1]:8081")
                .help("Specify the host/IP and port to which gRPC server binds for listening")
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
        // .arg(
        //     Arg::new("authority")
        //         .help("Specify the address of the token-issuing authentication server")
        //         .long("authority")
        //         .takes_value(true)
        //         .required(true),
        // )
        // .arg(
        //     Arg::new("audience")
        //         .help("Specify the resource server that should accept the token")
        //         .long("audience")
        //         .takes_value(true)
        //         .required(true),
        // )
        .get_matches();

    let addr: SocketAddr = matches.value_of("listen-address").unwrap().parse().unwrap();
    let database = connect_mongodb(
        matches.value_of("mongodb-uri").unwrap(),
        &"beta_nomkhonwaan_com",
    ).await?;

    println!("auth-service listening on {}", addr);
    Server::builder()
        .add_service(AuthServiceServer::new(
            MyAuthService::builder()
                .with_user_repository(Box::from(MongoUserRepository::new(
                    database.collection("users"),
                )))
                .build(),
        ))
        .serve(addr)
        .await?;

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
