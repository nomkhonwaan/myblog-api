use std::str::FromStr;

use mongodb::{bson::doc, bson::Document, bson::oid::ObjectId, Collection, Cursor};
use mongodb::options::FindOptions;
use myblog_proto_rust::myblog::proto::blog::{Taxonomy, TaxonomyType};
use tokio_stream::StreamExt;

use crate::encoding::bson::Unmarshaler;

/// A taxonomy repository definition.
#[tonic::async_trait]
pub trait TaxonomyRepository: Send + Sync + 'static {
    async fn find_by_id(&self, id: &str) -> Result<Option<Taxonomy>, Box<dyn std::error::Error>>;
    async fn find_all(&self, q: TaxonomyQuery) -> Result<Vec<Taxonomy>, Box<dyn std::error::Error>>;
    async fn find_all_by_ids(&self, ids: &Vec<&str>) -> Result<Vec<Taxonomy>, Box<dyn std::error::Error>>;
}

/// A taxonomy query builder.
#[derive(Default)]
pub struct TaxonomyQuery {
    /* Filters */
    taxonomy_type: TaxonomyType,
}

impl TaxonomyQuery {
    pub fn builder() -> Self {
        TaxonomyQuery {
            taxonomy_type: TaxonomyType::Category,
        }
    }

    pub fn with_type(mut self, taxonomy_type: TaxonomyType) -> Self {
        self.taxonomy_type = taxonomy_type;
        self
    }
}

/// An implementation of the TaxonomyRepository specifies with MongoDB.
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
        let filter = doc! {"_id": ObjectId::from_str(id)? };

        if let Some(document) = self.collection.find_one(filter, None).await? {
            return Ok(Some(Taxonomy::unmarshal_bson(&document)?));
        }

        Ok(None)
    }

    async fn find_all(
        &self,
        q: TaxonomyQuery,
    ) -> Result<Vec<Taxonomy>, Box<dyn std::error::Error>> {
        let filter = doc! {"type": q.taxonomy_type as i32};
        let find_options = FindOptions::builder().sort(doc! {"name": 1}).build();

        let mut cursor: Cursor<Document> = self.collection.find(filter, find_options).await?;
        let mut result: Vec<Taxonomy> = vec![];

        while let Some(document) = cursor.try_next().await? {
            result.push(Taxonomy::unmarshal_bson(&document)?);
        }

        Ok(result)
    }

    async fn find_all_by_ids(
        &self,
        ids: &Vec<&str>,
    ) -> Result<Vec<Taxonomy>, Box<dyn std::error::Error>> {
        let object_ids: Result<Vec<_>, _> = ids.iter().map(|id| ObjectId::from_str(id)).collect();
        let filter = doc! {"_id": {"$in": object_ids?}};

        let mut cursor: Cursor<Document> = self.collection.find(filter, None).await?;
        let mut result: Vec<Taxonomy> = vec![];

        while let Some(document) = cursor.try_next().await? {
            result.push(Taxonomy::unmarshal_bson(&document)?);
        }

        Ok(result)
    }
}

impl Unmarshaler for Taxonomy {
    fn unmarshal_bson(
        document: &Document,
    ) -> Result<Self, mongodb::bson::document::ValueAccessError>
        where
            Self: Sized,
    {
        Ok(Taxonomy {
            id: document.get_object_id("_id")?.to_hex(),
            name: document.get_str("name")?.to_owned(),
            slug: document.get_str("slug")?.to_owned(),
            r#type: document.get_i32("type")?.to_owned(),
        })
    }
}
