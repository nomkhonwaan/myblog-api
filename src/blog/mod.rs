pub mod service;
pub mod repository;

pub mod myblog {
    pub(crate) mod proto {
        pub(crate) mod blog {
            tonic::include_proto!("myblog.proto.blog");
        }

        pub(crate) mod storage {
            tonic::include_proto!("myblog.proto.storage");
        }
    }
}