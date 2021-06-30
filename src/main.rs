use alcoholic_jwt::JWKS;
use clap::{App, Arg};
use mongodb::{bson::doc, Client, Database, options::ClientOptions};
use myblog_proto_rust::myblog::proto::blog::blog_service_server::BlogServiceServer;
use tonic::transport::Server;

use crate::blog::{
    post::MongoPostRepository,
    service::MyBlogServiceServer,
    taxonomy::MongoTaxonomyRepository,
};

mod auth;
mod blog;
mod encoding;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("myblog")
        .about("Provide APIs for read/write on the blog content")
        .version("3.0.0")
        .arg(
            Arg::new("listen-address")
                .default_value("[::1]:8080")
                .about("The combination of IP address and listen port to serve the service")
                .long("listen-address")
                .takes_value(true),
        )
        .arg(
            Arg::new("mongodb-uri")
                .about("URI which can be used to create a MongoDB instance")
                .long("mongodb-uri")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("authority")
                .about("The address of the token-issuing authentication server")
                .long("authority")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("audience")
                .about("Refer to the resource server that should accept the token")
                .long("audience")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let addr = matches.value_of("listen-address").unwrap().parse().unwrap();

    // TODO: need to get a database name from the connection string instead
    let database = connect_mongodb(
        matches.value_of("mongodb-uri").unwrap(),
        &"beta_nomkhonwaan_com",
    )
        .await?;

    let authority = matches.value_of("authority").unwrap();
    let audience = matches.value_of("audience").unwrap();
    // Fetch the JSON Web Key Sets for using on the token validation
    let jwks = fetch_jwks(&format!("{}{}", authority, ".well-known/jwks.json")).await?;

    println!("server listening on {}", addr);

    Server::builder()
        .add_service(BlogServiceServer::with_interceptor(
            MyBlogServiceServer::builder()
                .with_post_repository(Box::from(MongoPostRepository::new(database.collection("posts"))))
                .with_taxonomy_repository(Box::from(MongoTaxonomyRepository::new(database.collection("taxonomies"))))
                .build(),
            auth::intercept(authority.to_string(), audience.to_string(), jwks),
        ))
        .serve(addr)
        .await?;

    Ok(())
}

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

async fn fetch_jwks(uri: &str) -> Result<JWKS, Box<dyn std::error::Error>> {
    let response = reqwest::get(uri).await?;
    let jwks = response.json::<JWKS>().await?;
    return Ok(jwks);
}