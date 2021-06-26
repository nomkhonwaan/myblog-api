use std::time::SystemTime;

use mongodb::bson::Document;
use myblog_proto_rust::myblog::proto::auth::User;
use prost_types::Timestamp;

use crate::encoding::bson::Unmarshal;

impl Unmarshal for User {
    fn unmarshal_bson(document: &Document) -> Result<Self, mongodb::bson::document::ValueAccessError> where Self: Sized {
        Ok(
            User {
                id: document.get_object_id("_id")?.to_hex(),
                user: document.get_str("user")?.to_owned(),
                display_name: document.get_str("displayName")?.to_owned(),
                profile_picture: document.get_str("profilePicture")?.to_owned(),
                created_at: Some(document.get_datetime("createdAt")
                    .and_then(|created_at| Ok(Timestamp::from(SystemTime::from(created_at.to_owned()))))?),
                updated_at: match document.get_datetime("updatedAt") {
                    Ok(updated_at) => Some(Timestamp::from(SystemTime::from(updated_at.to_owned()))),
                    _ => None,
                },
            }
        )
    }
}