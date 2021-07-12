use myblog_proto_rust::myblog::proto::{
    auth::User,
    discussion::{
        CreateCommentRequest, CreateCommentResponse,
        discussion_service_server::DiscussionService,
    },
};
use tonic::{Request, Response, Status};

use crate::auth::Claims;
use crate::discussion::comment::CommentRepository;

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