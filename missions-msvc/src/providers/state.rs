use futures_util::TryStreamExt;
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

    pub async fn find_one_by_two_fields<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        field_name_1: &str,
        field_value_1: &str,
        field_name_2: &str,
        field_value_2: &str,
    ) -> Result<Option<T>> {
        self.client
            .database(&self.db_name)
            .collection::<T>(collection_name)
            .find_one(
                doc! {field_name_1: field_value_1, field_name_2: field_value_2},
                None,
            )
            .await
    }

    pub async fn find_all_by_field<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        field_name: &str,
        field_value: &str,
    ) -> Result<Vec<T>> {
        self.client
            .database(&self.db_name)
            .collection::<T>(collection_name)
            .find(doc! {field_name: field_value}, None)
            .await?
            .try_collect()
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

    pub async fn delete_one_by_id<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        id: &str,
    ) -> Result<()> {
        match self
            .client
            .database(&self.db_name)
            .collection::<T>(collection_name)
            .delete_one(doc! {"_id": id}, None)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
