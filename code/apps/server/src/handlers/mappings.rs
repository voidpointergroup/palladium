use actix_web::{
    web::{
        Data,
        Json,
        Path,
        Query,
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
    pub expire: Option<String>,
    pub max_calls: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PostDirectiveResponse {
    pub id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PutDirectiveRequest {
    pub destination: String,
    pub expire: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PutDirectiveResponse {}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GetDirectiveResponseACLs {
    pub expire_at: Option<String>,
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
    let dir = srv.read(directive_id.clone()).await?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&GetDirectiveResponse {
        id: directive_id.to_owned(),
        url: dir.url,
        acls: GetDirectiveResponseACLs {
            expire_at: dir.acls.expire_at,
        },
    })?))
}

#[actix_web::get("/directives")]
pub async fn get_directives(
    _config: Data<Config>,
    srv: Data<Server>,
    next: Query<u16>,
    cursor: Query<Option<String>>,
) -> Result<HttpResponse, Error> {
    let keys = srv.list(*next, cursor.0).await?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&keys)?))
}

#[actix_web::delete("/directives/{directive_id}")]
pub async fn delete_directive(
    _config: Data<Config>,
    srv: Data<Server>,
    directive_id: Path<String>,
) -> Result<HttpResponse, Error> {
    srv.delete(&directive_id).await?;
    Ok(HttpResponse::Ok().body(""))
}

#[actix_web::delete("/directives")]
pub async fn delete_directives(_config: Data<Config>, srv: Data<Server>) -> Result<HttpResponse, Error> {
    srv.clear().await?;
    Ok(HttpResponse::Ok().body(""))
}

#[actix_web::put("/directives/{directive_id}")]
pub async fn put_directive(
    _config: Data<Config>,
    srv: Data<Server>,
    directive_id: Path<String>,
    body: Json<PutDirectiveRequest>,
) -> Result<HttpResponse, Error> {
    srv.register(Directive {
        id: directive_id.clone(),
        url: body.destination.clone(),
        acls: DirectiveACLs {
            expire_at: body.expire.clone(),
        },
    })
    .await?;
    Ok(HttpResponse::Created().body(serde_json::to_string(&PutDirectiveResponse {})?))
}

#[actix_web::post("/directives")]
pub async fn post_directive(
    _config: Data<Config>,
    srv: Data<Server>,
    body: Json<PostDirectiveRequest>,
) -> Result<HttpResponse, Error> {
    let id = srv.claim_id().await?;
    srv.register(Directive {
        id: id.clone(),
        url: body.destination.clone(),
        acls: DirectiveACLs {
            expire_at: body.expire.clone(),
        },
    })
    .await?;
    Ok(HttpResponse::Created().body(serde_json::to_string(&PostDirectiveResponse { id })?))
}

#[actix_web::get("/x/{directive_id}")]
pub async fn redirect(
    _config: Data<Config>,
    srv: Data<Server>,
    directive_id: Path<String>,
) -> Result<HttpResponse, Error> {
    let dir = srv.redirect(directive_id.clone()).await?;
    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", dir.url))
        .body(""))
}
