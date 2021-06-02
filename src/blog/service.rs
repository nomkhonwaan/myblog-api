use mongodb::Database;
use tonic::{Request, Response, Status};

use super::myblog::proto::blog::{ListPublishedPostsRequest, ListPublishedPostsResponse, PostStatus};
use super::myblog::proto::blog::blog_service_server::{BlogService, BlogServiceServer};
use super::repository::{MongoPostRepository, PostQuery, PostRepository};

/// An implementation of the BlogServiceServer which provides gRPC handler functions.
pub struct MyBlogServiceServer {
    post_repository: Box<dyn PostRepository>,
}

#[tonic::async_trait]
impl BlogService for MyBlogServiceServer {
    async fn list_published_posts(
        &self,
        request: Request<ListPublishedPostsRequest>,
    ) -> Result<Response<ListPublishedPostsResponse>, Status> {
        let r = request.into_inner();
        let q: PostQuery = PostQuery::builder().with_status(PostStatus::Published).with_offset(r.offset).with_limit(r.limit);

        match self.post_repository.find_all(q).await {
            Ok(posts) => Ok(Response::new(ListPublishedPostsResponse { posts })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

#[derive(Default)]
struct MyBlogServiceServerBuilder {
    post_repository: Option<Box<dyn PostRepository>>,
}

impl MyBlogServiceServerBuilder {
    pub fn with_post_repository(mut self, post_repository: Box<dyn PostRepository>) -> Self {
        self.post_repository = Some(post_repository);
        self
    }

    pub fn build(self) -> MyBlogServiceServer {
        MyBlogServiceServer {
            post_repository: self.post_repository.unwrap(),
        }
    }
}

pub fn new(database: Database) -> BlogServiceServer<MyBlogServiceServer> {
    BlogServiceServer::new(
        MyBlogServiceServerBuilder::default()
            .with_post_repository(Box::from(MongoPostRepository::new(database.collection("posts"))))
            .build()
    )
}