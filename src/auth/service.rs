use myblog_proto_rust::myblog::proto::auth::{
    auth_service_server::AuthService,
    CreateUserRequest, CreateUserResponse,
};
use tonic::{Request, Response, Status};

use crate::auth::user::UserRepository;

pub struct MyAuthService {
    user_repository: Box<dyn UserRepository>,
}

impl MyAuthService {
    pub fn builder() -> MyAuthServiceBuilder { MyAuthServiceBuilder::default() }
}

#[tonic::async_trait]
impl AuthService for MyAuthService {
    async fn create_user(&self, request: Request<CreateUserRequest>) -> Result<Response<CreateUserResponse>, Status> {
        todo!()
    }
}

#[derive(Default)]
pub struct MyAuthServiceBuilder {
    /* Repository */
    user_repository: Option<Box<dyn UserRepository>>,
}

impl MyAuthServiceBuilder {
    pub fn with_user_repository(mut self, repository: Box<dyn UserRepository>) -> Self {
        self.user_repository = Some(repository);
        self
    }

    pub fn build(self) -> MyAuthService {
        MyAuthService {
            user_repository: self.user_repository.unwrap(),
        }
    }
}