use myblog_proto_rust::myblog::proto::auth::auth_service_server::AuthService;

pub struct MyAuthService {}

#[tonic::async_trait]
impl AuthService for MyAuthService {}
