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

    fn key(&self, id: &str) -> String {
        format!("palladium:directive:{}", id)
    }

    pub async fn register(&self, directive: Directive) -> Result<String, Box<dyn Error>> {
        let id = uuid::Uuid::new_v4().to_string();

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
        Ok(id)
    }

    pub async fn redirect(&self, id: String) -> Result<Directive, Box<dyn Error>> {
        let data = redis::cmd("GET")
            .arg(self.key(&id))
            .query::<String>(&mut self.redis.get_connection()?)?;
        let dir = serde_json::from_str::<Directive>(&data)?;
        Ok(dir)
    }
}
