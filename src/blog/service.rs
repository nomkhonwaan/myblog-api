use myblog_proto_rust::myblog::proto::blog::{
    blog_service_server::BlogService,
    ListCategoriesResponse,
    ListCategoryPublishedPostsRequest,
    ListCategoryPublishedPostsResponse,
    ListPublishedPostsRequest,
    ListPublishedPostsResponse,
    ListTagPublishedPostsRequest,
    ListTagPublishedPostsResponse,
    PostStatus,
    TaxonomyType,
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

        match self.post_repository.find_all(&q).await {
            Ok(posts) => Ok(Response::new(ListPublishedPostsResponse { posts })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_category_published_posts(
        &self,
        request: Request<ListCategoryPublishedPostsRequest>,
    ) -> Result<Response<ListCategoryPublishedPostsResponse>, Status> {
        let r = request.into_inner();
        let q = PostQuery::builder()
            .with_status(PostStatus::Published)
            .with_category(r.category)
            .with_offset(r.offset)
            .with_limit(r.limit);

        match self.post_repository.find_all(&q).await {
            Ok(posts) => Ok(Response::new(ListCategoryPublishedPostsResponse { posts })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_tag_published_posts(
        &self, request: Request<ListTagPublishedPostsRequest>,
    ) -> Result<Response<ListTagPublishedPostsResponse>, Status> {
        let r = request.into_inner();
        let q = PostQuery::builder()
            .with_status(PostStatus::Published)
            .with_tag(r.tag)
            .with_offset(r.offset)
            .with_limit(r.limit);

        match self.post_repository.find_all(&q).await {
            Ok(posts) => Ok(Response::new(ListTagPublishedPostsResponse { posts })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // async fn list_post_comments(
    //     &self,
    //     request: Request<ListPostCommentsRequest>,
    // ) -> Result<Response<ListPostCommentsResponse>, Status> {
    //     let r = request.into_inner();
    //     let post = match r.post {
    //         Some(post) => Ok(post),
    //         _ => Err(Status::invalid_argument("Missing required 'post' field")),
    //     }?;
    //     let q = PostQuery::builder()
    //         .with_offset(r.offset)
    //         .with_limit(r.limit);
    //
    //     match self.post_repository.find_post_comments(post.id.as_str(), &q).await {
    //         Ok(comments) => Ok(Response::new(ListPostCommentsResponse { comments })),
    //         Err(e) => Err(Status::internal(e.to_string())),
    //     }
    // }

    // async fn list_post_attachments(
    //     &self,
    //     _request: Request<ListPostAttachmentsRequest>,
    // ) -> Result<Response<ListPostAttachmentsResponse>, Status> {
    //     Ok(Response::new(ListPostAttachmentsResponse {
    //         attachments: vec![],
    //     }))
    // }
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

#[cfg(test)]
mod tests {
    // use myblog_proto_rust::myblog::proto::blog::Post;
    // 
    // use crate::blog::post::PostRepository;
    // 
    // struct MockPostRepository {}
    // 
    // impl PostRepository for MockPostRepository {
    //     async fn find_by_id(&self, id: &str) -> Result<Option<Post>, Box<dyn std::error::Error>> {
    //         return Ok(None);
    //     }
    // }
    // 
    // #[test]
    // fn my_blog_service_list_categories() {
    //     // When
    // }
}