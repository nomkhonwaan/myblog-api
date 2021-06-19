use mongodb::bson::Document;

pub mod service;
pub mod post;
pub mod taxonomy;

trait Unmarshal {
    fn unmarshal_bson(document: &Document) -> Result<Self,mongodb::bson::document::ValueAccessError> where Self: Sized;
}

