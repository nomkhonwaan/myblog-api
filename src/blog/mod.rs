use mongodb::bson::Document;

pub mod service;
pub mod post;
pub mod taxonomy;

/// Provide function for un-marshaling data into self struct.
trait Unmarshal {
    fn unmarshal_bson(document: &Document) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
}
