use alcoholic_jwt::{JWKS, token_kid, Validation};
use serde::{Deserialize, Serialize};
use tonic::{Request, Status};

pub mod user;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
}

/// The gRPC interceptor for validating and extracting user info from the Bearer token (if exists).
pub fn interceptor(jwks: &JWKS) -> impl Fn(Request<()>) -> Result<Request<()>, Status> {
    |r: Request<()>| -> Result<Request<()>, Status> {
        Ok(r)
    }
}

// pub fn interceptor(authority: &str) -> impl Fn(Request<()>) -> Result<Request<()>, Status> {
//     let jwks = fetch_jwks(&format!("{}{}", authority, ".well-known/jwks.json"))
//         .expect("failed to fetch jwks");
//     let validations = vec![Validation::Issuer(authority.to_string()), Validation::SubjectPresent];
//     // Ok(validate_token)
//     // Ok(|r: Request<()>| -> Result<Request<()>, Box<dyn std::error::Error>>{
//     //     Ok(r)
//     // })
//     |r: Request<()>| {
//         Ok(r)
//     }
// }

// fn validate_token(r: Request<()>) -> Result<Request<()>, Status> {
//     // Ok(r)
// }
// pub fn new_interceptor(authority: &str) -> Result<(Request<()> -> Result<Request<() >, 
// Status>), < Box<dyn std::error::Error> {
// Ok((r: Request < () >) -> Result < Request < () >, Box < dyn std::error::Error > > {
// Ok(r)
// })
// }
// pub fn interceptor(r: Request<()>) -> Result<Request<()>, Status> {
//     if let Some(token) = r.metadata().get("authorization") {
//         let jwks = fetch_jwks(&format!("{}{}", authority.as_str(), ".well-known/jwks.json"))
//             .expect("failed to fetch jwks");
//         let validations = vec![Validation::Issuer(authority), Validation::SubjectPresent];
//         let kid = match token_kid(&token.t) {
//             Ok(res) => res.expect("failed to decode kid"),
//             Err(_) => return Err(ServiceError::JWKSFetchError),
//         };
//     }
// 
//     Ok(r)
// }
