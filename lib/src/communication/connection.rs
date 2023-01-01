use anyhow::{anyhow, Result};
use std::sync::Arc;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;

use std::collections::HashMap;
use zenoh::publication::Publisher;

#[derive(Clone)]
pub struct Connection<'a> {
    zenoh_session: Arc<Session>,
    publishers: HashMap<String, Publisher<'a>>,
}

impl<'a> Connection<'a> {
    pub async fn new() -> Connection<'a> {
        let session = zenoh::open(Config::default()).res().await.unwrap();

        Self {
            zenoh_session: Arc::new(session),
            publishers: HashMap::new(),
        }
    }

    pub async fn create_publisher(self: &mut Self, path: String) {
        if !self.publishers.contains_key(&path) {
            let publisher = self
                .zenoh_session
                .declare_publisher(path.clone())
                .res()
                .await
                .unwrap();

            self.publishers.insert(path, publisher);
        }
    }

    pub async fn publish<T>(self: &Self, path: &String, data: T) -> Result<()>
    where
        T: serde::Serialize,
    {
        match self.publishers.get(path) {
            Some(ref publisher) => {
                let json = serde_json::to_string(&data)?;
                publisher
                    .put(json)
                    .res()
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            }
            None => Err(anyhow!("Error")),
        }
    }
}
