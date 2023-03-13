use mongodb::bson::doc;
use mongodb::error::Result;
use mongodb::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone)]
pub struct MongoStateImpl {
    client: Client,
    db_name: String,
}

impl MongoStateImpl {
    pub async fn new(conn_url: &str, db_name: &str) -> Result<Self> {
        let client = Client::with_uri_str(conn_url).await?;

        client
            .database(db_name)
            .run_command(doc! {"ping": 1}, None)
            .await?;

        Ok(Self {
            client,
            db_name: String::from(db_name),
        })
    }
}

impl MongoStateImpl {
    pub async fn find_one_by_id<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        id: &str,
    ) -> Result<Option<T>> {
        self.client
            .database(&self.db_name)
            .collection::<T>(collection_name)
            .find_one(doc! {"_id": id}, None)
            .await
    }

    pub async fn find_one_by_field<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        field_name: &str,
        field_value: &str,
    ) -> Result<Option<T>> {
        self.client
            .database(&self.db_name)
            .collection::<T>(collection_name)
            .find_one(doc! {field_name: field_value}, None)
            .await
    }

    pub async fn insert_one<T: Serialize + Clone>(
        &self,
        collection_name: &str,
        document: &T,
    ) -> Result<T> {
        self.client
            .database(&self.db_name)
            .collection::<T>(collection_name)
            .insert_one(document.clone(), None)
            .await?;
        Ok(document.clone())
    }

    pub async fn update_one<T: Serialize + Clone>(
        &self,
        collection_name: &str,
        field_name: &str,
        field_value: &str,
        document: &T,
    ) -> Result<T> {
        self.client
            .database(&self.db_name)
            .collection::<T>(collection_name)
            .update_one(
                doc! {field_name: field_value},
                doc! {"$set": mongodb::bson::to_document(&document).unwrap()},
                None,
            )
            .await?;
        Ok(document.clone())
    }
}
