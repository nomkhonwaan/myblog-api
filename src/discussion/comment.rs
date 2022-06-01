use std::str::FromStr;
use std::time::SystemTime;

use mongodb::{bson::DateTime, bson::doc, bson::Document, bson::oid::ObjectId, Collection};
use myblog_proto_rust::myblog::proto::auth::User;
use myblog_proto_rust::myblog::proto::discussion::{Comment, CommentStatus};
use prost_types::Timestamp;

use crate::encoding::bson::{Marshaler, Unmarshaler};

// A comment repository definition.
#[tonic::async_trait]
pub trait CommentRepository: Send + Sync + 'static {
    async fn create(&self, c: &mut Comment) -> Result<(), Box<dyn std::error::Error>>;
    async fn find_all(&self, q: &CommentQuery) -> Result<Vec<Comment>, Box<dyn std::error::Error>>;
}

/// A comment query builder.
#[derive(Default)]
pub struct CommentQuery {
    status: Option<CommentStatus>,
    offset: u32,
    limit: u32,
}

impl CommentQuery {
    pub fn builder() -> Self {
        CommentQuery {
            offset: 0,
            limit: 5,
            ..Default::default()
        }
    }

    pub fn with_status(mut self, status: CommentStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }
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

    async fn find_all(&self, q: &CommentQuery) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        // Comment always sort by `created_at` time ascending
        let mut pipeline: Vec<Document> = vec![
            doc! {"$sort": {"createdAt": 1}},
        ];

        if let Some(status) = q.status {
            pipeline.push(doc! {"$match": {"status": status as i32}});
        }

        pipeline.append(&mut vec![
            doc! {"$lookup": {"from": "users", "localField": "author", "foreignField": "_id", "as": "author"}},
            doc! {"$unwind": {"path": "$author"}},
            doc! {"$offset": q.offset as i64},
            doc! {"$limit": q.limit as i64},
        ]);

        let mut cursor = self.collection.aggregate(pipeline, None).await?;
        let mut result: Vec<Comment> = vec![];

        while let Some(document) = cursor.try_next().await? {
            result.push(Comment::unmarshal_bson(&document)?);
        }

        Ok(result)
    }
}

impl Marshaler for Comment {
    fn marshal_bson(&self) -> Result<Document, mongodb::bson::oid::Error> {
        let mut document = doc! {
            "_id": ObjectId::from_str(self.id.as_str())?,
            "status": self.status,
            "text": self.text.as_str(),
            "author": self.author.as_ref().unwrap().id.as_str(),
            "children": self.children
                .iter()
                .map(|c| ObjectId::from_str(c.id.as_str()))
                .collect::<Result<Vec<ObjectId>, _>>()?,
            "createdAt": DateTime::from_millis(self.created_at.as_ref().unwrap().seconds * 1000),
        };

        // The parent id will be present when the comment has replied to another comment.
        if self.parent_id.is_some() {
            document.insert(
                "parent_id",
                ObjectId::from_str(self.parent_id.as_str())?,
            )
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
            parent_id: Some(document.get_object_id("parent_id")?.to_hex()),
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
