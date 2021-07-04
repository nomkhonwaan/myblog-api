use myblog_proto_rust::myblog::proto::auth::{
    auth_service_server::AuthService,
    CreateUserRequest, CreateUserResponse,
    User,
};
use tonic::{Request, Response, Status};

use crate::auth::Claims;
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
        let sub = match request.extensions().get::<Claims>() {
            Some(claims) => Ok(claims.sub.clone()),
            None => Err(Status::unauthenticated("Forbidden")),
        }?;

        let r = request.into_inner();
        if r.user.is_none() {
            return Err(Status::invalid_argument("Missing required 'user' field"));
        }

        let mut user = r.user.unwrap();
        user.user = sub;
        
        let existing_user = self.user_repository.find_by_user(user.user.as_str()).await
            .or_else(|err| Err(Status::internal(err.to_string())))?;
        if existing_user.is_some() {
            return Ok(Response::new(CreateUserResponse { user: existing_user }));
        }

        match self.user_repository.create(&mut user).await {
            Ok(_) => Ok(Response::new(CreateUserResponse { user: Some(user) })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
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