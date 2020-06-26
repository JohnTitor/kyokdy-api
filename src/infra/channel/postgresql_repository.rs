use std::convert::TryFrom;
use std::sync::Arc;

use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use tokio_postgres::{Client, Row};

use crate::domain::channel::model::{Channel, DraftChannel};
use crate::domain::channel::repository::ChannelRepository;
use crate::domain::url::Url;

#[derive(Clone)]
pub struct PostgreSQLChannelRepository {
    client: Arc<Client>,
}

impl TryFrom<&Row> for Channel {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self> {
        let icon_url: String = value.try_get("icon_url")?;

        Ok(Channel {
            id: value.try_get("id")?,
            channel_id: value.try_get("channel_id")?,
            name: value.try_get("name")?,
            icon_url: Url::try_from(icon_url)?,
        })
    }
}

impl PostgreSQLChannelRepository {
    pub fn new(client: Arc<Client>) -> Self {
        PostgreSQLChannelRepository { client }
    }
}

#[async_trait]
impl ChannelRepository for PostgreSQLChannelRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Channel>> {
        let result = self
            .client
            .query_one(r#"SELECT * FROM channels WHERE channel_id=$1;"#, &[&id])
            .await?;

        match result.is_empty() {
            true => Ok(None),
            false => Ok(Some(Channel::try_from(&result)?)),
        }
    }

    async fn search_by_name(&self, title: &str) -> Result<Vec<Channel>> {
        let rows = self
            .client
            .query(
                r#"SELECT * FROM channels WHERE name LIKE '%$1%';"#,
                &[&title],
            )
            .await?;
        rows.iter().try_fold(vec![], |mut channels, row| {
            if let Ok(channel) = Channel::try_from(row) {
                channels.push(channel);
            }
            Ok(channels)
        })
    }

    async fn create(&self, channel: DraftChannel) -> Result<()> {
        let result = self
            .client
            .execute(
                r#"INSERT INTO channels(channel_id, name, icon_url) VALUES ($1, $2, $3);"#,
                &[&channel.channel_id, &channel.name, &channel.icon_url.0],
            )
            .await?;
        match result {
            0 => Err(anyhow!("Failed Insert row.data: {:?}", channel)),
            _ => Ok(()),
        }
    }
}


#[cfg(test)]
#[cfg_attr(not(feature = "integration_test"), cfg(ignore))]
mod integration_test {
    use super::*;
    use dotenv::dotenv;
    use tokio_postgres::{connect, Client, NoTls};
    use tokio::spawn;
    use std::{env::vars, collections::HashMap};

    async fn teardown(client: Arc<Client>) {
        client.execute(r#"DELETE FROM "channels";"#, &[]).await.expect("Failed clean up channels table");
    }
    #[tokio::test]
    async fn create_add_row_and_find_by_id() {
        dotenv().ok();
        let envs: HashMap<_, _> = vars().collect();
        let db_config = envs.get("TESTING_DATABASE_URL").expect("TESTING_DATABASE_URL must be set");

        let (client, pg_connection) = connect(db_config, NoTls).await.unwrap();
        let a_client = Arc::new(client);

        spawn(async move {
            pg_connection.await
        });
        let repository = PostgreSQLChannelRepository::new(a_client.clone());
        let draft_channel = DraftChannel {
            channel_id: "foo".to_string(),
            name: "bar".to_string(),
            icon_url: Url::try_from("https://example.com").unwrap()
        };
        repository.create(draft_channel).await.expect("Failed create draft channel");
        let channel = repository.find_by_id("foo").await.expect("foo is not found in channels");
        assert!(channel.is_some());
        teardown(a_client).await;
    }
}
