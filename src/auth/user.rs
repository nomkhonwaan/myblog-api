use std::str::FromStr;
use std::time::SystemTime;

use mongodb::{bson::DateTime, bson::doc, bson::Document, bson::oid::ObjectId, Collection};
use myblog_proto_rust::myblog::proto::auth::User;
use prost_types::Timestamp;

use crate::encoding::bson::{Marshaler, Unmarshaler};

/// A user repository definition.
#[tonic::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, u: &mut User) -> Result<(), Box<dyn std::error::Error>>;
    async fn find_by_user(&self, user: &str) -> Result<Option<User>, Box<dyn std::error::Error>>;
}

/// An implementation of the UserRepository specifies with MongoDB.
pub struct MongoUserRepository {
    collection: Collection<Document>,
}

impl MongoUserRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        MongoUserRepository { collection }
    }
}

#[tonic::async_trait]
impl UserRepository for MongoUserRepository {
    async fn create(&self, u: &mut User) -> Result<(), Box<dyn std::error::Error>> {
        if u.id.is_empty() {
            u.id = ObjectId::new().to_hex();
        }

        self.collection.insert_one(&u.marshal_bson()?, None).await?;

        Ok(())
    }

    async fn find_by_user(&self, user: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let filter = doc! {"user": user };

        if let Some(document) = self.collection.find_one(filter, None).await? {
            return Ok(Some(User::unmarshal_bson(&document)?));
        }

        Ok(None)
    }
}

impl Marshaler for User {
    fn marshal_bson(&self) -> Result<Document, mongodb::bson::oid::Error> {
        let mut document = doc! {
            "_id": ObjectId::from_str(self.id.as_str())?,
            "user": self.user.as_str(),
            "displayName": self.display_name.as_str(),
            "profilePicture": self.profile_picture.as_str(),
            "createdAt": DateTime::from_millis(self.created_at.as_ref().unwrap().seconds * 1000),
        };

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
