use alcoholic_jwt::JWKS;
use clap::{App, Arg};
use mongodb::{bson::doc, Client, Database, options::ClientOptions};
use tonic::transport::Server;

use crate::blog::post::MongoPostRepository;
use crate::blog::service::MyBlogServiceServer;
use crate::blog::taxonomy::MongoTaxonomyRepository;

mod auth;
mod blog;
mod encoding;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("myblog")
        .version("3.0.0")
        .arg(
            Arg::new("listen-address")
                .default_value("[::1]:8080")
                .required(true),
        )
        .arg(
            Arg::new("mongodb-uri")
                .env("MONGODB_URI")
                .required(true),
        )
        .arg(
            Arg::new("authority")
                .env("AUTHORITY")
                .required(true),
        )
        .get_matches();

    let addr = matches.value_of("listen-address").unwrap()
        .parse().unwrap();

    // TODO: need to get a database name from the connection string instead
    let database = connect_mongodb(matches.value_of("mongodb-uri").unwrap(), &"beta_nomkhonwaan_com").await?;

    // Fetch the JSON Web Key Sets for using on the token validation
    let jwks = fetch_jwks(&format!("{}/{}", matches.value_of("authority").unwrap(), ".well-known/jwks.json")).await?;

    println!("server listening on {}", addr);

    Server::builder()
        .add_service(MyBlogServiceServer::builder()
            .with_interceptor(auth::interceptor(&jwks))
            .with_post_repository(Box::from(MongoPostRepository::new(database.collection("posts"))))
            .with_taxonomy_repository(Box::from(MongoTaxonomyRepository::new(database.collection("taxonomies"))))
            .build()
        )
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
        .await {
        Ok(_) => Ok(client.database(database)),
        Err(e) => Err(e),
    }
}

async fn fetch_jwks(uri: &str) -> Result<JWKS, Box<dyn std::error::Error>> {
    let response = reqwest::get(uri).await?;
    let jwks = response.json::<JWKS>().await?;
    return Ok(jwks);
}