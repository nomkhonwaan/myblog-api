use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use serde::{Deserialize, Serialize};
use tonic::{Request, Status};

pub mod service;
pub mod user;

/// User context that deserializes from the JSON Web Token string.
#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    permissions: Vec<String>,
}

/// The gRPC interceptor for validating and extracting user info from the Bearer token (if exists).
pub fn intercept(
    authority: String,
    audience: String,
    jwks: JWKS,
) -> impl FnMut(Request<()>) -> Result<Request<()>, Status> + Clone {
    move |mut r| -> Result<Request<()>, Status> {
        if let Some(metadata) = r.metadata().get("Authorization") {
            let validations = vec![
                Validation::Issuer(authority.clone()),
                Validation::Audience(audience.clone()),
                Validation::SubjectPresent,
                Validation::NotExpired,
            ];

            let mut token = metadata.to_str().unwrap_or_default();

            // This is intentional to not accept a lowercase "bearer"
            // https://datatracker.ietf.org/doc/html/rfc6750
            if token.contains("Bearer") {
                token = token[6..token.len()].trim();
            }

            let kid = token_kid(token)
                .expect("Failed to decode token headers")
                .expect("No 'kid' claim present in token");

            let jwk = jwks.find(&kid).expect("Specified key not found in set");

            if let Ok(valid_jwt) = validate(token, jwk, validations) {
                r.extensions_mut()
                    .insert(serde_json::from_value::<Claims>(valid_jwt.claims).unwrap());

                return Ok(r);
            }

            return Err(Status::unauthenticated("unauthorized"));
        }

        Ok(r)
    }
}
