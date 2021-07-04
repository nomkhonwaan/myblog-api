use myblog_proto_rust::myblog::proto::discussion::{
    discussion_service_server::DiscussionService, Comment, CreateCommentRequest,
    CreateCommentResponse,
};
use tonic::{Request, Response, Status};

pub struct MyDiscussionService {}

#[tonic::async_trait]
impl DiscussionService for MyDiscussionService {
    async fn create_comment(
        &self,
        _request: Request<CreateCommentRequest>,
    ) -> Result<Response<CreateCommentResponse>, Status> {
        Ok(Response::new(CreateCommentResponse {
            comment: Some(Comment::default()),
        }))
    }
}
