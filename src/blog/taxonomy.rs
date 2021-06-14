use mongodb::{bson::doc, bson::Document, bson::oid::ObjectId, Collection, Cursor};
use myblog_proto_rust::myblog::proto::blog::Taxonomy;
use tokio::stream::StreamExt;

use super::Unmarshal;

#[tonic::async_trait]
pub trait TaxonomyRepository: Send + Sync + 'static {
    async fn find_by_id(&self, id: &str) -> Result<Option<Taxonomy>, Box<dyn std::error::Error>>;
    async fn find_all_by_ids(&self, ids: &Vec<&str>) -> Result<Vec<Taxonomy>, Box<dyn std::error::Error>>;
}

pub struct MongoTaxonomyRepository {
    collection: Collection<Document>,
}

impl MongoTaxonomyRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        MongoTaxonomyRepository { collection }
    }
}

#[tonic::async_trait]
impl TaxonomyRepository for MongoTaxonomyRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Taxonomy>, Box<dyn std::error::Error>> {
        let filter = doc! {"_id": ObjectId::with_string(id)? };

        if let Some(document) = self.collection.find_one(filter, None).await? {
            return Ok(Some(Taxonomy::unmarshal_bson(&document)?));
        }

        Ok(None)
    }

    async fn find_all_by_ids(&self, ids: &Vec<&str>) -> Result<Vec<Taxonomy>, Box<dyn std::error::Error>> {
        let object_ids: std::result::Result<Vec<_>, _> = ids.iter().map(|id| { ObjectId::with_string(id) }).collect();
        let filter = doc! {"_id": {"$in": object_ids?}};

        let mut cursor: Cursor = self.collection.find(filter, None).await?;
        let mut result: Vec<Taxonomy> = Vec::new();

        while let Some(document) = cursor.next().await {
            result.push(Taxonomy::unmarshal_bson(&document?)?);
        }

        Ok(result)
    }
}

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
