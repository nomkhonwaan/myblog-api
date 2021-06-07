use mongodb::bson::Document;

pub mod service;
pub mod post;
pub mod taxonomy;

/// Alias for Result<T, Box<dyn std::error::Error>>
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Provide function for un-marshaling data into self struct.
trait Unmarshal {
    fn unmarshal_bson(document: &Document) -> Result<Self> where Self: Sized;
}

