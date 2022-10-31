use std::{
    error::Error,
    time::Duration,
};

use mongodb::bson::doc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DirectiveACLs {
    pub expire_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Directive {
    #[serde(rename = "_id")]
    pub id: String,
    pub url: String,
    pub acls: DirectiveACLs,
}

#[derive(Debug, Clone)]
pub struct Server {
    #[allow(dead_code)]
    config: crate::config::Config,
    directives: std::sync::Arc<mongodb::Collection<Directive>>,
}

impl Server {
    pub async fn new(config: crate::config::Config) -> Result<Self, Box<dyn Error>> {
        let mut client_options = mongodb::options::ClientOptions::parse(&config.mongodb.endpoint).await?;
        client_options.app_name = Some("palladium".to_string());
        let client = mongodb::Client::with_options(client_options)?;
        let db = client.database("directives");
        let coll = db.collection("directives");

        let expire_index_name = "expire".to_owned();
        if !coll.list_index_names().await?.contains(&expire_index_name) {
            coll.create_index(
                mongodb::IndexModel::builder()
                    .keys(doc! {
                        "acls.expire_at": 1
                    })
                    .options(Some(
                        mongodb::options::IndexOptions::builder()
                            .name(expire_index_name)
                            .expire_after(Duration::from_secs(0))
                            .build(),
                    ))
                    .build(),
                None,
            )
            .await?;
        }

        Ok(Server {
            config,
            directives: std::sync::Arc::new(coll),
        })
    }

    pub async fn claim_id(&self) -> Result<String, Box<dyn Error>> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    pub async fn register(&self, directive: Directive) -> Result<(), Box<dyn Error>> {
        self.directives.insert_one(directive, None).await?;
        Ok(())
    }

    pub async fn list(&self, next: u16, cursor: Option<String>) -> Result<Vec<String>, Box<dyn Error>> {
        let filter = match cursor {
            | Some(c) => Some(doc! {
                "_id": {
                    "$gt": c
                }
            }),
            | None => None,
        };
        let mut bson_cursor = self.directives.find(filter, None).await?;

        let mut res = Vec::<String>::new();
        let mut remaining = next;
        while remaining > 0 {
            if !bson_cursor.advance().await? {
                break;
            }
            res.push(bson_cursor.deserialize_current()?.id);
            remaining -= 1;
        }
        Ok(res)
    }

    pub async fn delete(&self, id: &str) -> Result<(), Box<dyn Error>> {
        self.directives
            .find_one_and_delete(
                doc! {
                    "_id": id,
                },
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), Box<dyn Error>> {
        self.directives.delete_many(doc! {}, None).await?;
        Ok(())
    }

    pub async fn read(&self, id: String) -> Result<Directive, Box<dyn Error>> {
        self.directives
            .find_one(
                doc! {
                    "_id": id,
                },
                None,
            )
            .await?
            .ok_or(Box::new(crate::error::Runtime::new("not found")))
    }

    pub async fn redirect(&self, id: String) -> Result<Directive, Box<dyn Error>> {
        self.directives
            .find_one(
                doc! {
                    "_id": id,
                },
                None,
            )
            .await?
            .ok_or(Box::new(crate::error::Runtime::new("not found")))
    }
}
