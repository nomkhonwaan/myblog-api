use mongodb::bson::Document;

/// Provide a MongoDB specific marshaling function.
pub trait Marshaler {
    fn marshal_bson(&self) -> Document;
}

/// Provide a MongoDB specific un-marshaling function.
pub trait Unmarshaler {
    fn unmarshal_bson(
        document: &Document,
    ) -> Result<Self, mongodb::bson::document::ValueAccessError>
    where
        Self: Sized;
}
