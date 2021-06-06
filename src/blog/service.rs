use mongodb::Database;
use myblog_proto_rust::myblog::proto::blog::{ListPublishedPostsRequest, ListPublishedPostsResponse, PostStatus};
use myblog_proto_rust::myblog::proto::blog::blog_service_server::{BlogService, BlogServiceServer};
use tonic::{Request, Response, Status};

use super::post::{MongoPostRepository, PostQuery, PostRepository};

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

// #[cfg(test)]
// mod tests {
//     use myblog_proto_rust::myblog::proto::blog::{ListPublishedPostsRequest, Post};
//     use myblog_proto_rust::myblog::proto::blog::blog_service_server::BlogService;
//     use tonic::Request;
//     use tonic::transport::channel::ResponseFuture;
// 
//     use crate::blog::post::{PostQuery, PostRepository};
//     use crate::blog::service::MyBlogServiceServer;
// 
//     #[derive(Default)]
//     struct MockPostRepository {
//         find_all_post_query: Option<PostQuery>,
//     }
// 
//     #[tonic::async_trait]
//     impl PostRepository for MockPostRepository {
//         async fn find_all(&mut self, q: PostQuery) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
//             self.find_all_post_query = Some(q);
//             Ok(Vec::new())
//         }
//     }
// 
//     #[test]
//     fn list_published_posts() {
//         // Given
//         let post_repository = MockPostRepository::default();
//         let myblog_service_server = MyBlogServiceServer { post_repository: Box::from(post_repository) };
//         let expected = PostQuery::builder();
//         
//         // When
//         let _result =
//             myblog_service_server.list_published_posts(Request::new(ListPublishedPostsRequest { offset: 0, limit: 5 }));
// 
//         // Then
//         // assert_eq!(expected, post_repository.find_all_post_query.unwrap());
//     }
// }