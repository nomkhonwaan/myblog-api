use std::time::SystemTime;

use mongodb::{bson::doc, bson::Document, bson::oid::ObjectId, Collection, Cursor, options::FindOptions};
use myblog_proto_rust::myblog::proto::blog::{Post, PostStatus, Taxonomy};
use myblog_proto_rust::myblog::proto::storage::File;
use prost_types::Timestamp;
use tokio::stream::StreamExt;
use tonic;

use super::Unmarshal;

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

    pub fn with_status(mut self, status: PostStatus) -> Self {
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
        let mut find_options = FindOptions::builder()
            .sort(doc! {"created_at": 1})
            .skip(q.offset as i64)
            .limit(q.limit as i64)
            .build();

        if let Some(status) = q.status {
            filter.insert("status", status as i32);

            // Will sort by `published_at` descending if status is `Published`
            if status == PostStatus::Published {
                find_options.sort = Some(doc! { "published_at": -1 });
            }
        }

        let mut cursor: Cursor = self.collection.find(filter, find_options).await?;
        let mut result: Vec<Post> = Vec::new();

        while let Some(document) = cursor.next().await {
            result.push(Post::unmarshal_bson(&document?)?);
        }

        Ok(result)
    }
}

/// An implementation of Post for un-marshaling data into struct.
impl Unmarshal for Post {
    fn unmarshal_bson(document: &Document) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized {
        Ok(Post {
            id: document.get_object_id("_id")?.to_hex(),
            title: document.get_str("title")?.to_owned(),
            slug: document.get_str("slug")?.to_owned(),
            status: document.get_i32("status")?.to_owned(),
            markdown: document.get_str("markdown")?.to_owned(),
            html: document.get_str("html")?.to_owned(),
            published_at: Some(document.get_datetime("published_at")
                .and_then(|published_at| Ok(Timestamp::from(SystemTime::from(published_at.to_owned()))))?),
            author_id: document.get_object_id("author_id")?.to_hex(),
            categories: document.get_array("categories")
                .and_then(|categories| {
                    Ok(categories
                        .into_iter()
                        .map(|id| {
                            Taxonomy {
                                id: id.as_object_id().or(Some(&ObjectId::new())).unwrap().to_hex(),
                                ..Default::default()
                            }
                        })
                        .collect())
                })?,
            tags: document.get_array("tags")
                .and_then(|tags| {
                    Ok(tags
                        .into_iter()
                        .map(|id| {
                            Taxonomy {
                                id: id.as_object_id().or(Some(&ObjectId::new())).unwrap().to_hex(),
                                ..Default::default()
                            }
                        })
                        .collect())
                })?,
            featured_image: Some(document.get_object_id("featured_image")
                .and_then(|featured_image| {
                    Ok(File {
                        id: featured_image.to_hex(),
                        ..Default::default()
                    })
                })?),
            attachments: document.get_array("attachments")
                .and_then(|attachments| {
                    Ok(attachments
                        .into_iter()
                        .map(|id| {
                            File {
                                id: id.as_object_id().or(Some(&ObjectId::new())).unwrap().to_hex(),
                                ..Default::default()
                            }
                        })
                        .collect())
                })?,
            created_at: Some(document.get_datetime("created_at")
                .and_then(|created_at| Ok(Timestamp::from(SystemTime::from(created_at.to_owned()))))?),
            updated_at: Some(document.get_datetime("updated_at")
                .and_then(|updated_at| Ok(Timestamp::from(SystemTime::from(updated_at.to_owned()))))?),
        })
    }
}

#[cfg(test)]
mod tests {
    use myblog_proto_rust::myblog::proto::blog::PostStatus;

    use crate::blog::post::PostQuery;

    #[test]
    fn init_post_query() {
        // Given

        // When
        let q: PostQuery = PostQuery::builder();

        // Then
        assert_eq!(0, q.offset);
        assert_eq!(5, q.limit);
    }

    #[test]
    fn post_query_with_status() {
        // Given

        // When
        let q: PostQuery = PostQuery::builder().with_status(PostStatus::Published);

        // When
        assert_eq!(PostStatus::Published, q.status.unwrap());
    }

    #[test]
    fn post_query_with_offset() {
        // Given 

        // When
        let q: PostQuery = PostQuery::builder().with_offset(19);

        // Then
        assert_eq!(19, q.offset);
    }

    #[test]
    fn post_query_with_limit() {
        // Given

        // When
        let q: PostQuery = PostQuery::builder().with_limit(6);

        // Then
        assert_eq!(6, q.limit);
    }
}