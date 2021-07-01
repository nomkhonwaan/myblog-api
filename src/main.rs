use alcoholic_jwt::JWKS;
use clap::{App, Arg};
use mongodb::{bson::doc, Client, Database, options::ClientOptions};
use myblog_proto_rust::myblog::proto::blog::blog_service_server::BlogServiceServer;
use tokio::fs;
use tonic::transport::{Identity, Server, ServerTlsConfig};

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
                .about("Specify the host/IP and port to which gRPC server binds for listening")
                .long("listen-address")
                .takes_value(true),
        )
        .arg(
            Arg::new("tls-certificate")
                .about("Provide public and private key pairs for enabling TLS")
                .long("tls-certificate")
                .takes_value(true)
                .requires("tls-certificate-key")
        )
        .arg(
            Arg::new("tls-certificate-key")
                .about("Provide public and private key paris for enabling TLS")
                .long("tls-certificate-key")
                .takes_value(true)
                .requires("tls-certificate")
        )
        .arg(
            Arg::new("mongodb-uri")
                .about("Specify URI which can be used to create a MongoDB instance")
                .long("mongodb-uri")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("authority")
                .about("Specify the address of the token-issuing authentication server")
                .long("authority")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("audience")
                .about("Specify the resource server that should accept the token")
                .long("audience")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let addr = matches.value_of("listen-address").unwrap().parse().unwrap();

    // TODO: need to get a database name from the connection string instead
    let database = connect_mongodb(matches.value_of("mongodb-uri").unwrap(), &"beta_nomkhonwaan_com").await?;

    let authority = matches.value_of("authority").unwrap();
    let audience = matches.value_of("audience").unwrap();
    // Fetch the JSON Web Key Sets for using on the token validation
    let jwks = fetch_jwks(&format!("{}{}", authority, ".well-known/jwks.json")).await?;

    let mut builder = Server::builder();

    if let (Some(cert), Some(key)) = (
        matches.value_of("tls-certificate"),
        matches.value_of("tls-certificate-key"),
    ) {
        let identity = Identity::from_pem(fs::read(cert).await?, fs::read(key).await?);
        builder = builder.tls_config(ServerTlsConfig::new().identity(identity))?;
    }

    println!("server listening on {}", addr);

    builder.add_service(BlogServiceServer::with_interceptor(
        MyBlogServiceServer::builder()
            .with_post_repository(Box::from(MongoPostRepository::new(database.collection("posts"))))
            .with_taxonomy_repository(Box::from(MongoTaxonomyRepository::new(database.collection("taxonomies"))))
            .build(),
        auth::intercept(authority.to_string(), audience.to_string(), jwks),
    )).serve(addr).await?;

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