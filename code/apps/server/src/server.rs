use std::{
    error::Error,
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DirectiveACLs {
    pub max_calls: Option<u64>,
    pub expiry: Option<String>,
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
            .cmd("SET")
            .arg(self.key(&id))
            .arg(serde_json::to_string(&directive)?);
        match directive.acls.expiry {
            | Some(v) => {
                let time_exp = chrono::DateTime::<chrono::Utc>::from_str(&v)?.timestamp();
                let diff = chrono::Utc::now().timestamp() - time_exp;
                redis_command = redis_command.cmd("EXPIRE").arg(self.key(&id)).arg(diff.to_string());
            },
            | None => {},
        }
        redis_command.query::<()>(&mut self.redis.get_connection()?)?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let keys = redis::pipe()
            .cmd("KEYS")
            .arg("*")
            .query::<Vec<String>>(&mut self.redis.get_connection()?)?;
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
        redis::pipe()
            .cmd("DEL")
            .arg(&format!("{}:{}", self.key_root(), "*"))
            .query::<()>(&mut self.redis.get_connection()?)?;
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