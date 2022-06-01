use myblog_proto_rust::myblog::proto::{
    auth::User,
    discussion::{
        CreateCommentRequest, CreateCommentResponse,
        discussion_service_server::DiscussionService,
    },
};
use myblog_proto_rust::myblog::proto::discussion::{CommentStatus, DeleteCommentRequest, GetCommentRequest, GetCommentResponse, ListPublishedCommentsRequest, ListPublishedCommentsResponse, UpdateCommentRequest, UpdateCommentResponse};
use tonic::{Request, Response, Status};

use crate::auth::Claims;
use crate::discussion::comment::{CommentQuery, CommentRepository};

pub struct MyDiscussionService {
    comment_repository: Box<dyn CommentRepository>,
}

impl MyDiscussionService {
    pub fn builder() -> MyDiscussionServiceBuilder { MyDiscussionServiceBuilder::default() }
}

#[tonic::async_trait]
impl DiscussionService for MyDiscussionService {
    async fn create_comment(
        &self,
        request: Request<CreateCommentRequest>,
    ) -> Result<Response<CreateCommentResponse>, Status> {
        // TODO: Will check on the list of the permissions are containing "write:comment" as well
        let sub = match request.extensions().get::<Claims>() {
            Some(claims) => Ok(claims.sub.clone()),
            _ => Err(Status::unauthenticated("Forbidden")),
        }?;
        let mut comment = match request.into_inner().comment {
            Some(comment) => Ok(comment),
            _ => Err(Status::invalid_argument("Missing required 'comment' field")),
        }?;

        let mut user = User::default();
        user.id = sub;
        comment.author = Some(user);

        match self.comment_repository.create(&mut comment).await {
            Ok(_) => Ok(Response::new(CreateCommentResponse { comment: Some(comment) })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_published_comments(
        &self,
        request: Request<ListPublishedCommentsRequest>,
    ) -> Result<Response<ListPublishedCommentsResponse>, Status> {
        let r = request.into_inner();
        let q = CommentQuery::builder()
            .with_status(CommentStatus::Published)
            .with_offset(r.offset)
            .with_limit(r.limit);

        match self.comment_repository.find_all(&q).await? {
            Ok(comments) => Ok(Response::new(ListPublishedCommentsResponse { comments })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_comment(&self, request: Request<GetCommentRequest>) -> Result<Response<GetCommentResponse>, Status> {
        todo!()
    }

    async fn update_comment(&self, request: Request<UpdateCommentRequest>) -> Result<Response<UpdateCommentResponse>, Status> {
        todo!()
    }

    async fn delete_comment(&self, request: Request<DeleteCommentRequest>) -> Result<Response<()>, Status> {
        todo!()
    }
}

#[derive(Default)]
pub struct MyDiscussionServiceBuilder {
    /* Repository */
    comment_repository: Option<Box<dyn CommentRepository>>,
}

impl MyDiscussionServiceBuilder {
    pub fn with_comment_repository(mut self, repository: Box<dyn CommentRepository>) -> Self {
        self.comment_repository = Some(repository);
        self
    }

    pub fn build(self) -> MyDiscussionService {
        MyDiscussionService {
            comment_repository: self.comment_repository.unwrap(),
        }
    }
}