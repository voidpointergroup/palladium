use std::error::Error;

use redis::Commands;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectiveExpiry {
    At(String),
    Seconds(u64),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DirectiveACLs {
    pub expiry: Option<DirectiveExpiry>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Directive {
    pub url: String,
    pub acls: DirectiveACLs,
}

#[derive(Debug, Clone)]
pub struct Server {
    #[allow(dead_code)]
    config: crate::config::Config,
    redis: std::sync::Arc<redis::Client>,
}

impl Server {
    pub fn new(config: crate::config::Config, redis: redis::Client) -> Self {
        let redis_connection_arc = std::sync::Arc::new(redis);

        Server {
            config,
            redis: redis_connection_arc,
        }
    }

    fn key_root(&self) -> String {
        "palladium:directive".to_owned()
    }

    fn key(&self, id: &str) -> String {
        format!("{}:{}", self.key_root(), id)
    }

    pub async fn register(&self, directive: Directive) -> Result<String, Box<dyn Error>> {
        let id = uuid::Uuid::new_v4().to_string();
        self.register_with_id(id.clone(), directive).await?;
        Ok(id)
    }

    pub async fn register_with_id(&self, id: String, directive: Directive) -> Result<(), Box<dyn Error>> {
        let mut redis_command = &mut redis::pipe();
        redis_command = redis_command
            .atomic()
            .cmd("SET")
            .arg(self.key(&id))
            .arg(serde_json::to_string(&directive)?);
        match directive.acls.expiry {
            | Some(v) => match v {
                | DirectiveExpiry::At(v) => {
                    let time_exp = chrono::DateTime::parse_from_rfc3339(&v)?.timestamp();
                    redis_command = redis_command.cmd("EXPIREAT").arg(self.key(&id)).arg(time_exp);
                },
                | DirectiveExpiry::Seconds(v) => {
                    redis_command = redis_command.cmd("EXPIRE").arg(self.key(&id)).arg(v);
                },
            },
            | None => {},
        }
        redis_command.query::<()>(&mut self.redis.get_connection()?)?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let prefix = format!("{}:", self.key_root());
        let pattern = format!("{}{}", prefix, "*");
        let keys = self
            .redis
            .get_connection()?
            .scan_match::<String, String>(pattern)?
            .map(|d| d.trim_start_matches(&prefix).to_owned())
            .collect();
        Ok(keys)
    }

    pub async fn delete(&self, id: &str) -> Result<(), Box<dyn Error>> {
        redis::pipe()
            .cmd("DEL")
            .arg(self.key(&id))
            .query::<()>(&mut self.redis.get_connection()?)?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), Box<dyn Error>> {
        let mut pipe = &mut redis::pipe();
        let keys = self.list().await?;
        for k in keys {
            pipe = pipe.cmd("DEL").arg(self.key(&k))
        }
        pipe.query::<()>(&mut self.redis.get_connection()?)?;
        Ok(())
    }

    pub async fn redirect(&self, id: String) -> Result<Directive, Box<dyn Error>> {
        let data = redis::cmd("GET")
            .arg(self.key(&id))
            .query::<String>(&mut self.redis.get_connection()?)?;
        let dir = serde_json::from_str::<Directive>(&data)?;
        Ok(dir)
    }
}
