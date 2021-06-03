use std::time::SystemTime;

use mongodb::{bson::doc, bson::Document, Collection, Cursor, options::FindOptions};
use prost_types::Timestamp;
use tokio::stream::StreamExt;
use tonic;

use super::myblog::api::proto::blog::{Post, PostStatus, Taxonomy};
use super::myblog::api::proto::storage::{File};

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
        let mut post = Post::default();

        post.id = document.get_object_id("_id")?.to_hex();

        if let Ok(title) = document.get_str("title") {
            post.title = title.to_owned();
        }
        if let Ok(slug) = document.get_str("slug") {
            post.slug = slug.to_owned();
        }
        if let Ok(status) = document.get_i32("status") {
            post.status = status.to_owned();
        }
        if let Ok(markdown) = document.get_str("markdown") {
            post.markdown = markdown.to_owned();
        }
        if let Ok(html) = document.get_str("html") {
            post.html = html.to_owned();
        }
        if let Ok(published_at) = document.get_datetime("published_at") {
            post.published_at = Some(Timestamp::from(SystemTime::from(published_at.to_owned())));
        }
        if let Ok(author_id) = document.get_object_id("author_id") {
            post.author_id = author_id.to_hex();
        }
        if let Ok(categories) = document.get_array("categories") {
            for category in categories.into_iter() {
                let mut taxonomy = Taxonomy::default();
                if let Some(id) = category.as_object_id() { taxonomy.id = id.to_hex(); }
                post.categories.push(taxonomy);
            }
        }
        if let Ok(tags) = document.get_array("tags") {
            for tag in tags.into_iter() {
                let mut taxonomy = Taxonomy::default();
                if let Some(id) = tag.as_object_id() { taxonomy.id = id.to_hex(); }
                post.tags.push(taxonomy);
            }
        }
        if let Ok(featured_image) = document.get_object_id("featured_image") {
            let mut file = File::default();
            file.id = featured_image.to_hex();
            post.featured_image = Some(file);
        }
        if let Ok(attachments) = document.get_array("attachments") {
            for attachment in attachments.into_iter() {
                let mut file = File::default();
                if let Some(id) = attachment.as_object_id() { file.id = id.to_hex(); }
                post.attachments.push(file);
            }
        }
        if let Ok(created_at) = document.get_datetime("created_at") {
            post.created_at = Some(Timestamp::from(SystemTime::from(created_at.to_owned())));
        }
        if let Ok(updated_at) = document.get_datetime("updated_at") {
            post.updated_at = Some(Timestamp::from(SystemTime::from(updated_at.to_owned())));
        }

        Ok(post)
    }
}

impl Taxonomy {
    pub fn unmarshal_bson(document: &Document) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Taxonomy {
            id: document.get_object_id("_id")?.to_hex(),
            name: document.get_str("name")?.to_owned(),
            slug: document.get_str("slug")?.to_owned(),
            term_group: document.get_str("term_group")?.to_owned(),
        })
    }
}