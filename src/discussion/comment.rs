use std::str::FromStr;
use std::time::SystemTime;

use mongodb::{bson::doc, bson::Document, Collection};
use myblog_proto_rust::myblog::proto::auth::User;
use myblog_proto_rust::myblog::proto::discussion::Comment;
use prost_types::Timestamp;
use tokio_stream::StreamExt;

use crate::encoding::bson::{Marshaler, Unmarshaler};
use chrono::DateTime;

#[tonic::async_trait]
pub trait CommentRepository: Send + Sync + 'static {
    async fn create(&self, c: Comment);
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
    async fn create(&self, c: Comment) {
        // mongodb::bson::DateTime::from_millis() c.created_at.unwrap().seconds
        // self.collection.insert_one()
    }
}

impl Marshaler for Comment {
    fn marshal_bson(&self) -> Document {
        let mut document = doc! {
            "status": self.status,
            "text": self.text.to_string(),
            // "author": self.author.marshal_bson(),
            // "parent": self.parent.marshal_bson(),
            // "children": self.children.marshal_bson(),
            "created_at": mongodb::bson::DateTime
        };

        document
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
