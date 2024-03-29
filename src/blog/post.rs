use std::str::FromStr;
use std::time::SystemTime;

use mongodb::{bson::doc, bson::oid::ObjectId, bson::Document, Collection};
use myblog_proto_rust::myblog::proto::{
    auth::User,
    blog::{Post, PostStatus, Taxonomy},
    discussion::Comment,
    storage::File,
};
use prost_types::Timestamp;
use tokio_stream::StreamExt;
use tonic;

use crate::encoding::bson::Unmarshaler;

/// A post repository definition.
#[tonic::async_trait]
pub trait PostRepository: Send + Sync + 'static {
    async fn find_by_id(&self, id: &str) -> Result<Option<Post>, Box<dyn std::error::Error>>;
    async fn find_all(&self, q: &PostQuery) -> Result<Vec<Post>, Box<dyn std::error::Error>>;
    // async fn find_post_comments(&self, id: &str, q: &PostQuery) -> Result<Vec<Comment>, Box<dyn std::error::Error>>;
    // async fn find_post_attachments(&self, id: &str) -> Result<Vec<File>, Box<dyn std::error::Error>>;
}

/// A post query builder.
#[derive(Default)]
pub struct PostQuery {
    /* Filters */
    status: Option<PostStatus>,
    category: Option<Taxonomy>,
    tag: Option<Taxonomy>,

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

    pub fn with_category(mut self, category: Option<Taxonomy>) -> Self {
        self.category = category;
        self
    }

    pub fn with_tag(mut self, tag: Option<Taxonomy>) -> Self {
        self.tag = tag;
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
    async fn find_by_id(&self, id: &str) -> Result<Option<Post>, Box<dyn std::error::Error>> {
        let filter = doc! {"_id": ObjectId::from_str(id)?};

        if let Some(document) = self.collection.find_one(filter, None).await? {
            return Ok(Some(Post::unmarshal_bson(&document)?));
        }

        Ok(None)
    }

    async fn find_all(&self, q: &PostQuery) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let mut pipeline: Vec<Document> = vec![];

        if let Some(status) = q.status {
            pipeline.push(doc! {"$match": {"status": status as i32}});

            // Will sort by `published_at` descending if status is `Published`
            if status == PostStatus::Published {
                pipeline.push(doc! {"$sort": {"publishedAt": -1}})
            } else {
                pipeline.push(doc! {"$sort": {"createdAt": -1}})
            }
        }

        if let Some(category) = &q.category {
            pipeline
                .push(doc! {"$match": {"categories": ObjectId::from_str(category.id.as_str())?}});
        }
        if let Some(tag) = &q.tag {
            pipeline
                .push(doc! {"$match": {"tags": ObjectId::from_str(tag.id.as_str())?}});
        }

        pipeline.append(&mut vec![
            doc! {"$lookup": {"from": "users", "localField": "author", "foreignField": "_id", "as": "author"}},
            doc! {"$unwind": {"path": "$author"}},
            doc! {"$lookup": {"from": "taxonomies", "localField": "categories", "foreignField": "_id", "as": "categories"}},
            doc! {"$lookup": {"from": "taxonomies", "localField": "tags", "foreignField": "_id", "as": "tags"}},
            doc! {"$lookup": {"from": "files", "localField": "featuredImage", "foreignField": "_id", "as": "featuredImage"}},
            doc! {"$unwind": {"path": "$featuredImage", "preserveNullAndEmptyArrays": true}},
            doc! {"$skip": q.offset as i64},
            doc! {"$limit": q.limit as i64},
        ]);

        let mut cursor = self.collection.aggregate(pipeline, None).await?;
        let mut result: Vec<Post> = vec![];

        while let Some(document) = cursor.try_next().await? {
            result.push(Post::unmarshal_bson(&document)?);
        }

        Ok(result)
    }

    // async fn find_post_comments(&self, id: &str, q: &PostQuery) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
    //     let pipeline = vec![
    //         doc! {"$match": {"_id": ObjectId::from_str(id)?}},
    //         doc! {"$lookup": {"from": "comments", "localField": "comments", "foreignField": "_id", "as": "comments"}},
    //         doc! {"$project": {"comments": 1}},
    //         doc! {"$skip": q.offset as i64},
    //         doc! {"$limit": q.limit as i64},
    //     ];
    //
    //     let mut cursor = self.collection.aggregate(pipeline, None).await?;
    //     let mut result: Vec<Comment> = vec![];
    //
    //     while let Some(document) = cursor.try_next().await? {
    //         result = document.get_array("comments")
    //             .and_then(|comments| {
    //                 comments
    //                     .into_iter()
    //                     .map(|comment| comment.as_document())
    //                     .filter_map(|comment| comment)
    //                     .map(|comment| Comment::unmarshal_bson(comment))
    //                     .collect::<Result<Vec<Comment>, _>>()
    //             })?;
    //     }
    //
    //     Ok(result)
    // }
    //
    // async fn find_post_attachments(&self, id: &str) -> Result<Vec<File>, Box<dyn std::error::Error>> {
    //     let pipeline = vec![
    //         doc! {"$match": {"_id": ObjectId::from_str(id)?}},
    //         doc! {"$lookup": {"from": "files", "localField": "attachments", "foreignField": "_id", "as": "attachments"}},
    //         doc! {"$project": {"attachments": 1}},
    //     ];
    //
    //     let mut cursor = self.collection.aggregate(pipeline, None).await?;
    //     let mut result: Vec<File> = vec![];
    //
    //     while let Some(document) = cursor.try_next().await? {
    //         result = document.get_array("attachments")
    //             .and_then(|files| {
    //                 files
    //                     .into_iter()
    //                     .map(|file| file.as_document())
    //                     .filter_map(|file| file)
    //                     .map(|file| File::unmarshal_bson(file))
    //                     .collect::<Result<Vec<File>, _>>()
    //             })?;
    //     }
    //
    //     Ok(result)
    // }
}

impl Unmarshaler for Post {
    fn unmarshal_bson(
        document: &Document,
    ) -> Result<Self, mongodb::bson::document::ValueAccessError>
        where
            Self: Sized,
    {
        Ok(Post {
            id: document.get_object_id("_id")?.to_hex(),
            title: document.get_str("title")?.to_owned(),
            slug: document.get_str("slug")?.to_owned(),
            status: document.get_i32("status")?.to_owned(),
            markdown: document.get_str("markdown")?.to_owned(),
            html: document.get_str("html")?.to_owned(),
            published_at: match document.get_datetime("publishedAt") {
                Ok(published_at) => {
                    Some(Timestamp::from(SystemTime::from(published_at.to_owned())))
                }
                _ => None,
            },
            author: Some(
                document
                    .get_document("author")
                    .and_then(|author| User::unmarshal_bson(author))?,
            ),
            categories: document.get_array("categories").and_then(|categories| {
                categories
                    .into_iter()
                    .map(|category| category.as_document())
                    .filter_map(|category| category)
                    .map(|category| Taxonomy::unmarshal_bson(category))
                    .collect::<Result<Vec<Taxonomy>, _>>()
            })?,
            tags: document.get_array("tags").and_then(|tags| {
                tags.into_iter()
                    .map(|tag| tag.as_document())
                    .filter_map(|tag| tag)
                    .map(|tag| Taxonomy::unmarshal_bson(tag))
                    .collect::<Result<Vec<Taxonomy>, _>>()
            })?,
            featured_image: match document.get_document("featuredImage") {
                Ok(featured_image) => Some(File::unmarshal_bson(featured_image)?),
                _ => None,
            },
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

#[cfg(test)]
mod tests {
    use myblog_proto_rust::myblog::proto::blog::{PostStatus, Taxonomy, TaxonomyType};

    use crate::blog::post::{MongoPostRepository, PostQuery};

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
        let q = PostQuery::builder().with_status(PostStatus::Published);

        // When
        assert_eq!(PostStatus::Published, q.status.unwrap());
    }

    #[test]
    fn post_query_with_category() {
        // Given
        let cat = Taxonomy {
            id: String::from("1"),
            name: String::from("Test"),
            slug: String::from("test-1"),
            r#type: TaxonomyType::Category as i32,
        };

        // When
        let q: PostQuery = PostQuery::builder().with_category(Some(cat));

        // Then
        assert_eq!("1", q.category.unwrap().id);
    }
    
    #[test]
    fn post_query_with_tag() {
        // Given
        let tag = Taxonomy{
            id: String::from("2"),
            name: String::from("Test"),
            slug: String::from("test-2"),
            r#type: TaxonomyType::Tag as i32,
        };
        
        // When
        let q = PostQuery::builder().with_tag(Some(tag));
        
        // Then
        assert_eq!("2", q.tag.unwrap().id);
    }

    #[test]
    fn post_query_with_offset() {
        // Given

        // When
        let q = PostQuery::builder().with_offset(19);

        // Then
        assert_eq!(19, q.offset);
    }

    #[test]
    fn post_query_with_limit() {
        // Given

        // When
        let q = PostQuery::builder().with_limit(6);

        // Then
        assert_eq!(6, q.limit);
    }
}
