use std::net::SocketAddr;

use clap::{Arg, Command};
use mongodb::{bson::doc, Client, Database, options::ClientOptions};
use myblog_proto_rust::myblog::proto::blog::blog_service_server::BlogServiceServer;
use tonic::transport::Server;

use myblog_api::blog::{
    post::MongoPostRepository, service::MyBlogService, taxonomy::MongoTaxonomyRepository,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("blog-service")
        .override_help("Part of myblog-api provides all blogging APIs")
        .version("3.0.0")
        .arg(
            Arg::new("listen-address")
                .default_value("[::1]:8082")
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
        .get_matches();

    let addr: SocketAddr = matches.value_of("listen-address").unwrap().parse().unwrap();
    let database = connect_mongodb(
        matches.value_of("mongodb-uri").unwrap(),
        &"beta_nomkhonwaan_com",
    ).await?;

    println!("blog-service listening on {}", addr);
    Server::builder()
        .add_service(BlogServiceServer::new(
            MyBlogService::builder()
                .with_post_repository(Box::from(MongoPostRepository::new(
                    database.collection("posts"),
                )))
                .with_taxonomy_repository(Box::from(MongoTaxonomyRepository::new(
                    database.collection("taxonomies"),
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
