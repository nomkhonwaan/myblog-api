use std::str::FromStr;
use std::time::SystemTime;

use mongodb::{bson::DateTime, bson::doc, bson::Document, bson::oid::ObjectId, Collection};
use myblog_proto_rust::myblog::proto::auth::User;
use myblog_proto_rust::myblog::proto::discussion::Comment;
use prost_types::Timestamp;

use crate::encoding::bson::{Marshaler, Unmarshaler};

// A comment repository definition.
#[tonic::async_trait]
pub trait CommentRepository: Send + Sync + 'static {
    async fn create(&self, c: &mut Comment) -> Result<(), Box<dyn std::error::Error>>;
}

/// An implementation of the CommentRepository specifies with MongoDB.
pub struct MongoCommentRepository {
    collection: Collection<Document>,
}

impl MongoCommentRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        MongoCommentRepository { collection }
    }
}

#[tonic::async_trait]
impl CommentRepository for MongoCommentRepository {
    async fn create(&self, c: &mut Comment) -> Result<(), Box<dyn std::error::Error>> {
        if c.id.is_empty() {
            c.id = ObjectId::new().to_hex();
        }

        self.collection.insert_one(&c.marshal_bson()?, None).await?;

        Ok(())
    }
}

impl Marshaler for Comment {
    fn marshal_bson(&self) -> Result<Document, mongodb::bson::oid::Error> {
        let mut document = doc! {
            "_id": ObjectId::from_str(self.id.as_str())?,
            "status": self.status,
            "text": self.status,
            "author": ObjectId::from_str(self.author.as_ref().unwrap().id.as_str())?,
            "children": self.children
                .iter()
                .map(|c| ObjectId::from_str(c.id.as_str()))
                .collect::<Result<Vec<ObjectId>, _>>()?,
            "createdAt": DateTime::from_millis(self.created_at.as_ref().unwrap().seconds * 1000),
        };

        if self.parent.is_some() {
            document.insert(
                "parent",
                ObjectId::from_str(self.parent.as_ref().unwrap().id.as_str())?,
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

impl Unmarshaler for Comment {
    fn unmarshal_bson(
        document: &Document,
    ) -> Result<Self, mongodb::bson::document::ValueAccessError>
        where
            Self: Sized,
    {
        Ok(Comment {
            id: document.get_object_id("_id")?.to_hex(),
            status: document.get_i32("status")?.to_owned(),
            text: document.get_str("text")?.to_owned(),
            author: Some(
                document
                    .get_document("author")
                    .and_then(|author| User::unmarshal_bson(author))?,
            ),
            parent: None,
            children: vec![],
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
