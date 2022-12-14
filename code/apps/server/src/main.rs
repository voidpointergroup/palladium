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
    let bind = config.server.address.clone();
    let service = Server::new(config.clone()).await?;
    HttpServer::new(move || {
        App::new()
            .service(crate::handlers::health::handler)
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(service.clone()))
            .service(crate::handlers::mappings::redirect)
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(service.clone()))
            .service(crate::handlers::mappings::get_directive)
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(service.clone()))
            .service(crate::handlers::mappings::delete_directive)
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(service.clone()))
            .service(crate::handlers::mappings::get_directives)
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(service.clone()))
            .app_data(
                actix_web::web::JsonConfig::default()
                    .content_type_required(true)
                    .content_type(|m| m == "application/json")
                    .limit(1024 * 4), // 4kb
            )
            .service(crate::handlers::mappings::post_directive)
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(service.clone()))
            .service(crate::handlers::mappings::delete_directives)
    })
    .bind(&bind)?
    .run()
    .await?;
    Ok(())
}
