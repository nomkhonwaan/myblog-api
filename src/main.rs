use clap::{App, Arg};
use mongodb::{bson::doc, Client, Database, options::ClientOptions};
use tonic::transport::Server;

mod blog;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("myblog")
        .version("3.0.0")
        .arg(
            Arg::new("listen-address")
                .default_value("[::1]:9111")
                .required(true),
        )
        .arg(
            Arg::new("mongodb-uri")
                .env("MONGODB_URI")
                .required(true),
        )
        .get_matches();

    let addr = matches.value_of("listen-address").unwrap()
        .parse().unwrap();

    // TODO: need to get a database name from the connection string instead
    let database = connect_mongodb(matches.value_of("mongodb-uri").unwrap(), &"beta_nomkhonwaan_com").await?;

    println!("server listening on {}", addr);

    Server::builder()
        .add_service(blog::service::new(database))
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
