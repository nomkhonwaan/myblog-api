use mongodb::bson::Document;
use myblog_proto_rust::myblog::proto::blog::Taxonomy;

use super::Unmarshal;

impl Unmarshal for Taxonomy {
    fn unmarshal_bson(document: &Document) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized {
        Ok(Taxonomy {
            id: document.get_object_id("_id")?.to_hex(),
            name: document.get_str("name")?.to_owned(),
            slug: document.get_str("slug")?.to_owned(),
            term_group: document.get_str("term_group")?.to_owned(),
        })
    }
}