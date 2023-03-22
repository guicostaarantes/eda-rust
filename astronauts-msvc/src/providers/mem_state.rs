use redis::Client;
use redis::RedisError;

#[derive(Clone)]
pub struct RedisMemStateImpl {
    client: Client,
}

impl RedisMemStateImpl {
    pub fn new(conn_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(conn_url)?;

        Ok(Self { client })
    }
}

impl RedisMemStateImpl {
    pub async fn get(&self, key: &str) -> Result<String, RedisError> {
        let mut con = self.client.get_async_connection().await?;

        redis::cmd("GET").arg(key).query_async(&mut con).await
    }
}

impl RedisMemStateImpl {
    pub async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;

        redis::cmd("SET")
            .arg(&[key, value])
            .query_async(&mut con)
            .await?;

        if let Some(seconds) = ttl {
            redis::cmd("EXPIRE")
                .arg(&[key, &format!("{}", seconds)])
                .query_async(&mut con)
                .await?;
        }

        Ok(())
    }
}

impl RedisMemStateImpl {
    pub async fn unset(&self, key: &str) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;

        redis::cmd("EXPIRE")
            .arg(&[key, "0"])
            .query_async(&mut con)
            .await?;

        Ok(())
    }
}
