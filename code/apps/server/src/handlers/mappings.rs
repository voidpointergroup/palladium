use actix_web::{
    web::{
        Data,
        Json,
        Path,
    },
    Error,
    HttpResponse,
};

use crate::{
    config::Config,
    server::{
        Directive,
        DirectiveACLs,
        Server,
    },
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PostDirectiveRequest {
    pub destination: String,
    pub expire_at: Option<String>,
    pub max_calls: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PostDirectiveResponse {
    pub id: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GetDirectiveResponseACLs {
    pub max_calls: Option<u64>,
    pub expiry: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GetDirectiveResponse {
    pub id: String,
    pub url: String,
    pub acls: GetDirectiveResponseACLs,
}

#[actix_web::get("/directives/{directive_id}")]
pub async fn get_directive(
    _config: Data<Config>,
    srv: Data<Server>,
    directive_id: Path<String>,
) -> Result<HttpResponse, Error> {
    let dir = srv.redirect(directive_id.clone()).await?;
    Ok(
        actix_web::HttpResponse::Created().body(serde_json::to_string(&GetDirectiveResponse {
            id: directive_id.to_owned(),
            url: dir.url,
            acls: GetDirectiveResponseACLs {
                max_calls: dir.acls.max_calls,
                expiry: dir.acls.expiry,
            },
        })?),
    )
}

#[actix_web::post("/directives")]
pub async fn post_directive(
    _config: Data<Config>,
    srv: Data<Server>,
    body: Json<PostDirectiveRequest>,
) -> Result<HttpResponse, Error> {
    let id = srv
        .register(Directive {
            url: body.destination.clone(),
            acls: DirectiveACLs {
                max_calls: Some(2u64),
                expiry: None,
            },
        })
        .await?;
    Ok(actix_web::HttpResponse::Created().body(serde_json::to_string(&PostDirectiveResponse { id })?))
}

#[actix_web::get("/x/{directive_id}")]
pub async fn redirect(
    _config: Data<Config>,
    srv: Data<Server>,
    directive_id: Path<String>,
) -> Result<HttpResponse, Error> {
    let dir = srv.redirect(directive_id.clone()).await?;
    Ok(actix_web::HttpResponse::TemporaryRedirect()
        .append_header(("Location", dir.url))
        .body(""))
}
