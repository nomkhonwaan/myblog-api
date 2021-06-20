use mongodb::bson::Document;

/// Provide a MongoDB specific un-marshaling function.
pub trait Unmarshal {
    fn unmarshal_bson(document: &Document) -> Result<Self, mongodb::bson::document::ValueAccessError> where Self: Sized;
}