use std::convert::TryFrom;

use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use serde::{Deserialize, Serialize};
use tonic::metadata::MetadataValue;
use tonic::{Request, Status};

pub mod user;

pub const JWT_CLAIMS: &str = "claims";

/// User context that deserializes from the JSON Web Token string.
#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    permissions: Vec<String>,
}

/// The gRPC interceptor for validating and extracting user info from the Bearer token (if exists).
pub fn interceptor(
    authority: String,
    audience: String,
    jwks: JWKS,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> {
    move |mut r: Request<()>| -> Result<Request<()>, Status> {
        if let Some(metadata) = r.metadata().get("Authorization") {
            let validations = vec![
                Validation::Issuer(authority.clone()),
                Validation::Audience(audience.clone()),
                Validation::SubjectPresent,
                Validation::NotExpired,
            ];

            if let Some(result) = metadata
                .to_str()
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .get(1)
                .map(|token| {
                    let kid = token_kid(*token)
                        .expect("Failed to decode token headers")
                        .expect("No 'kid' claim present in token");
                    let jwk = jwks.find(&kid).expect("Specified key not found in set");

                    validate(*token, jwk, validations)
                })
            {
                if let Ok(valid_jwt) = result {
                    // let claims: Claims = serde_json::from_value(valid_jwt.claims).unwrap();
                    //
                    // let metadta_value: MetadataValue<Claims> =
                    //     MetadataValue::try_from(claims).unwrap();

                    r.extensions_mut()
                        .insert(serde_json::from_value::<Claims>(valid_jwt.claims).unwrap());
                    // r.metadata_mut().insert(
                    //     JWT_SUB,
                    //     MetadataValue::from_str(
                    //         valid_jwt.claims.get("sub").unwrap().as_str().unwrap(),
                    //     )
                    //     .unwrap(),
                    // );

                    // r.metadata_mut().insert(
                    //     JWT_CLAIMS,
                    //     MetadataValue::try_from(Claims::default()).unwrap(),
                    // );
                    // r.metadata_mut().insert("")
                    // r.metadata_mut().insert(
                    //     "claims",
                    //     MetadataValue(valid_jwt.claims.get("sub").unwrap().as_str()),
                    //     // Claims {
                    //     //     sub: valid_jwt.claims.get("sub").unwrap().as_str().to_string(),
                    //     //     ..Default::default()
                    //     // },
                    // );

                    return Ok(r);
                }
            }

            return Err(Status::unauthenticated("unauthorized"));
        }

        Ok(r)
    }
}
