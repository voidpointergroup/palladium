mod args;
mod config;
mod error;
mod server;
mod handlers {
    pub mod health;
    pub mod mappings;
}

use std::error::Error;

use actix_web::{
    web::Data,
    App,
    HttpServer,
};
use server::Server;

#[actix_web::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Serve { config } => {
            serve(config).await?;
            Ok(())
        },
    }
}

/// Main server function, starting an actix HTTP server with the various
/// endpoints.
async fn serve(config: crate::config::Config) -> std::result::Result<(), Box<dyn Error>> {
    let redis = redis::Client::open(config.redis.endpoint.clone())?;
    if let Err(e) = redis.get_connection() {
        let e = &format!("could not connect to {}, {}", &config.redis.endpoint, e.to_string());
        return Err(Box::new(crate::error::StartupError::new(e)));
    }
    let bind = config.server.address.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Server::new(config.clone(), redis.clone())))
            .service(crate::handlers::health::handler)
    })
    .bind(&bind)?
    .run()
    .await?;
    Ok(())
}
