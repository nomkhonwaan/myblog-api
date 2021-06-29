use myblog_proto_rust::myblog::proto::blog::blog_service_server::{BlogService, BlogServiceServer};
use myblog_proto_rust::myblog::proto::blog::{
    ListCategoriesResponse, ListPublishedPostsRequest, ListPublishedPostsResponse,
    ListTaxonomyPublishedPostsRequest, ListTaxonomyPublishedPostsResponse, PostStatus,
    TaxonomyType,
};
use tonic::{Request, Response, Status};

use crate::blog::taxonomy::{TaxonomyQuery, TaxonomyRepository};

use super::post::{PostQuery, PostRepository};

/// An implementation of the BlogServiceServer which provides gRPC handler functions.
pub struct MyBlogServiceServer {
    post_repository: Box<dyn PostRepository>,
    taxonomy_repository: Box<dyn TaxonomyRepository>,
}

impl MyBlogServiceServer {
    pub fn builder() -> MyBlogServiceServerBuilder {
        MyBlogServiceServerBuilder::default()
    }
}

#[tonic::async_trait]
impl BlogService for MyBlogServiceServer {
    async fn list_categories(
        &self,
        _: Request<()>,
    ) -> Result<Response<ListCategoriesResponse>, Status> {
        match self
            .taxonomy_repository
            .find_all(TaxonomyQuery::builder().with_type(TaxonomyType::Category))
            .await
        {
            Ok(categories) => Ok(Response::new(ListCategoriesResponse { categories })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_published_posts(
        &self,
        request: Request<ListPublishedPostsRequest>,
    ) -> Result<Response<ListPublishedPostsResponse>, Status> {
        let r = request.into_inner();
        let q = PostQuery::builder()
            .with_status(PostStatus::Published)
            .with_offset(r.offset)
            .with_limit(r.limit);

        match self.post_repository.find_all(q).await {
            Ok(posts) => Ok(Response::new(ListPublishedPostsResponse { posts })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_taxonomy_published_posts(
        &self,
        request: Request<ListTaxonomyPublishedPostsRequest>,
    ) -> Result<Response<ListTaxonomyPublishedPostsResponse>, Status> {
        let r = request.into_inner();
        let q = PostQuery::builder()
            .with_status(PostStatus::Published)
            .with_category(r.taxonomy)
            .with_offset(r.offset)
            .with_limit(r.limit);

        match self.post_repository.find_all(q).await {
            Ok(posts) => Ok(Response::new(ListTaxonomyPublishedPostsResponse { posts })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

#[derive(Default)]
pub struct MyBlogServiceServerBuilder {
    /* gRPC */
    interceptor: Option<F>,

    /* Repositories */
    post_repository: Option<Box<dyn PostRepository>>,
    taxonomy_repository: Option<Box<dyn TaxonomyRepository>>,
}

impl MyBlogServiceServerBuilder {
    pub fn with_interceptor(mut self, interceptor: F) -> Self
    where
        F: FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>,
    {
        self.interceptor = Some(interceptor.into());
        self
    }

    pub fn with_post_repository(mut self, repository: Box<dyn PostRepository>) -> Self {
        self.post_repository = Some(repository);
        self
    }

    pub fn with_taxonomy_repository(mut self, repository: Box<dyn TaxonomyRepository>) -> Self {
        self.taxonomy_repository = Some(repository);
        self
    }

    //     pub fn build(self) -> BlogServiceServer<MyBlogServiceServer> {
    //         let inner = MyBlogServiceServer {
    //             post_repository: self.post_repository.unwrap(),
    //             taxonomy_repository: self.taxonomy_repository.unwrap(),
    //         };
    //
    //         let a = BlogServiceServer::with_interceptor(inner, self.interceptor);
    // )
    //         BlogServiceServer::with_interceptor(innsert, self.interceptor.unwrap());
    //         BlogServiceServer::new(inner)
    //
    //         // match self.interceptor {
    //         //     Some(interceptor) => BlogServiceServer::with_interceptor(inner, interceptor),
    //         //     _ => BlogServiceServer::new(inner),
    //         }
    //     }
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
