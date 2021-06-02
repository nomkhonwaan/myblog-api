use std::time::SystemTime;

use mongodb::{bson::doc, bson::Document, Collection, Cursor, options::FindOptions};
use prost_types::Timestamp;
use tokio::stream::StreamExt;
use tonic;

use super::myblog::proto::blog::{Post, PostStatus};

/// A post repository definition.
#[tonic::async_trait]
pub trait PostRepository: Send + Sync + 'static {
    async fn find_all(&self, q: PostQuery) -> Result<Vec<Post>, Box<dyn std::error::Error>>;
}

/// A post query builder.
#[derive(Default)]
pub struct PostQuery {
    /* Filters */
    status: Option<PostStatus>,

    /* Pagination Options */
    offset: u32,
    limit: u32,
}

impl PostQuery {
    pub fn builder() -> Self {
        PostQuery {
            offset: 0,
            limit: 5,
            ..Default::default()
        }
    }

    pub fn with_status(self, status: PostStatus) -> Self {
        PostQuery {
            status: Some(status),
            ..self
        }
    }

    pub fn with_offset(self, offset: u32) -> Self {
        PostQuery {
            offset,
            ..self
        }
    }

    pub fn with_limit(self, limit: u32) -> Self {
        PostQuery {
            limit,
            ..self
        }
    }
}

/// An implementation of the PostRepository specifies with MongoDB.
pub struct MongoPostRepository {
    collection: Collection<Document>,
}

impl MongoPostRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        MongoPostRepository { collection }
    }
}

#[tonic::async_trait]
impl PostRepository for MongoPostRepository {
    async fn find_all(&self, q: PostQuery) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let mut filter = doc! {};

        if let Some(post_status) = q.status {
            filter.insert("status", match post_status {
                PostStatus::Draft => "DRAFT",
                PostStatus::Published => "PUBLISHED",
            });
        }

        let find_options = FindOptions::builder().skip(q.offset as i64).limit(q.limit as i64).build();

        let mut cursor: Cursor = self.collection.find(Some(filter), find_options).await?;
        let mut result: Vec<Post> = Vec::new();

        while let Some(document) = cursor.next().await {
            result.push(Post::unmarshal_bson(&document?)?);
        }

        Ok(result)
    }
}

/// An implementation of Post struct for marshaling, un-marshaling.
impl Post {
    pub fn unmarshal_bson(document: &Document) -> Result<Self, Box<dyn std::error::Error>> {
        let mut post = Post {
            id: document.get_object_id("_id")?.to_hex(),
            title: document.get_str("title")?.to_owned(),
            slug: document.get_str("slug")?.to_owned(),
            status: match document.get_str("status")?.to_owned().as_str() {
                "PUBLISHED" => PostStatus::Published as i32,
                "DRAFT" => PostStatus::Draft as i32,
                // Default status for un-marshaling is "0" which means draft
                _ => 0i32,
            },
            markdown: document.get_str("markdown")?.to_owned(),
            html: document.get_str("html")?.to_owned(),
            published_at: None,
            author_id: document.get_str("authorId")?.to_owned(),
            created_at: None,
            updated_at: None,
        };

        // TODO: this will convert chrono::DateTime to std::time::SystemTime then prost_types::TimeStamp
        if let Ok(published_at) = document.get_datetime("publishedAt") {
            post.published_at = Some(Timestamp::from(SystemTime::from(published_at.to_owned())));
        }

        if let Ok(created_at) = document.get_datetime("createdAt") {
            post.created_at = Some(Timestamp::from(SystemTime::from(created_at.to_owned())));
        }

        if let Ok(updated_at) = document.get_datetime("updatedAt") {
            post.updated_at = Some(Timestamp::from(SystemTime::from(updated_at.to_owned())));
        }

        Ok(post)
    }
}

// impl Category {
//     pub fn unmarshal_bson(document: &Document) -> Result<Self, Box<dyn std::error::Error>> {
//         todo()
//     }
// }