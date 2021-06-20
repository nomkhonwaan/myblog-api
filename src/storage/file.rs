use std::time::SystemTime;

use mongodb::bson::Document;
use myblog_proto_rust::myblog::proto::storage::File;
use prost_types::Timestamp;

use crate::encoding::bson::Unmarshal;

impl Unmarshal for File {
    fn unmarshal_bson(document: &Document) -> Result<Self, mongodb::bson::document::ValueAccessError> where Self: Sized {
        Ok(
            File {
                id: document.get_object_id("_id")?.to_hex(),
                file_name: document.get_str("fileName")?.to_owned(),
                slug: document.get_str("slug")?.to_owned(),
                uploaded_file_path: document.get_str("uploadedFilePath")?.to_owned(),
                mime_type: document.get_str("mimeType")?.to_owned(),
                provider: document.get_str("provider")?.to_owned(),
                region: document.get_str("region")?.to_owned(),
                bucket: document.get_str("bucket")?.to_owned(),
                uploaded_at: Some(document.get_datetime("uploadedAt")
                    .and_then(|uploaded_at| Ok(Timestamp::from(SystemTime::from(uploaded_at.to_owned()))))?),
                modified_at: match document.get_datetime("modifiedAt") {
                    Ok(modified_at) => Some(Timestamp::from(SystemTime::from(modified_at.to_owned()))),
                    _ => None,
                },
            }
        )
    }
}