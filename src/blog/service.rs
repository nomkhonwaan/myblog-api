use myblog_proto_rust::myblog::proto::blog::{
    blog_service_server::BlogService, ListCategoriesResponse, ListPostAttachmentsRequest,
    ListPostAttachmentsResponse, ListPostCommentsRequest, ListPostCommentsResponse,
    ListPublishedPostsRequest, ListPublishedPostsResponse, ListTaxonomyPublishedPostsRequest,
    ListTaxonomyPublishedPostsResponse, PostStatus, TaxonomyType,
};
use tonic::{Request, Response, Status};

use crate::blog::{
    post::{PostQuery, PostRepository},
    taxonomy::{TaxonomyQuery, TaxonomyRepository},
};

pub struct MyBlogService {
    post_repository: Box<dyn PostRepository>,
    taxonomy_repository: Box<dyn TaxonomyRepository>,
}

impl MyBlogService {
    pub fn builder() -> MyBlogServiceBuilder {
        MyBlogServiceBuilder::default()
    }
}

#[tonic::async_trait]
impl BlogService for MyBlogService {
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

    async fn list_post_comments(
        &self,
        _request: Request<ListPostCommentsRequest>,
    ) -> Result<Response<ListPostCommentsResponse>, Status> {
        Ok(Response::new(ListPostCommentsResponse { comments: vec![] }))
    }

    async fn list_post_attachments(
        &self,
        _request: Request<ListPostAttachmentsRequest>,
    ) -> Result<Response<ListPostAttachmentsResponse>, Status> {
        Ok(Response::new(ListPostAttachmentsResponse {
            attachments: vec![],
        }))
    }
}

#[derive(Default)]
pub struct MyBlogServiceBuilder {
    /* Repositories */
    post_repository: Option<Box<dyn PostRepository>>,
    taxonomy_repository: Option<Box<dyn TaxonomyRepository>>,
}

impl MyBlogServiceBuilder {
    pub fn with_post_repository(mut self, repository: Box<dyn PostRepository>) -> Self {
        self.post_repository = Some(repository);
        self
    }

    pub fn with_taxonomy_repository(mut self, repository: Box<dyn TaxonomyRepository>) -> Self {
        self.taxonomy_repository = Some(repository);
        self
    }

    pub fn build(self) -> MyBlogService {
        MyBlogService {
            post_repository: self.post_repository.unwrap(),
            taxonomy_repository: self.taxonomy_repository.unwrap(),
        }
    }
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
