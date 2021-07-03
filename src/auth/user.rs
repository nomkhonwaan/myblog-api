use std::str::FromStr;
use std::time::SystemTime;

use mongodb::{bson::doc, bson::oid::ObjectId, bson::DateTime, bson::Document};
use myblog_proto_rust::myblog::proto::auth::User;
use prost_types::Timestamp;

use crate::encoding::bson::{Marshaler, Unmarshaler};

impl Marshaler for User {
    fn marshal_bson(&self) -> Result<Document, Box<dyn std::error::Error>> {
        let mut document = doc! {
            "_id": ObjectId::from_str(self.id.as_str())?,
            "user": self.user.as_str(),
            "displayName": self.display_name.as_str(),
            "profilePicture": self.profile_picture.as_str(),
        };

        if self.created_at.is_some() {
            document.insert(
                "createdAt",
                DateTime::from_millis(self.created_at.as_ref().unwrap().seconds * 1000),
            );
        }

        if self.updated_at.is_some() {
            document.insert(
                "updatedAt",
                DateTime::from_millis(self.updated_at.as_ref().unwrap().seconds * 1000),
            );
        }

        Ok(document)
    }
}

impl Unmarshaler for User {
    fn unmarshal_bson(
        document: &Document,
    ) -> Result<Self, mongodb::bson::document::ValueAccessError>
    where
        Self: Sized,
    {
        Ok(User {
            id: document.get_object_id("_id")?.to_hex(),
            user: document.get_str("user")?.to_owned(),
            display_name: document.get_str("displayName")?.to_owned(),
            profile_picture: document.get_str("profilePicture")?.to_owned(),
            created_at: Some(document.get_datetime("createdAt").and_then(|created_at| {
                Ok(Timestamp::from(SystemTime::from(created_at.to_owned())))
            })?),
            updated_at: match document.get_datetime("updatedAt") {
                Ok(updated_at) => Some(Timestamp::from(SystemTime::from(updated_at.to_owned()))),
                _ => None,
            },
        })
    }
}
