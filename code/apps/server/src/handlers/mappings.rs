use actix_web::{
    web::{
        Data,
        Json,
        Path,
        Query,
    },
    Error,
    HttpRequest,
    HttpResponse,
};

use crate::{
    config::Config,
    server::{
        Directive,
        DirectiveACLAuth,
        DirectiveACLCalls,
        DirectiveACLs,
        Server,
    },
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PostDirectiveRequestAuth {
    pub key: String,
    pub secret: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PostDirectiveRequest {
    pub destination: String,
    pub expire: Option<String>,
    pub max_calls: Option<u64>,
    pub auth: Option<PostDirectiveRequestAuth>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PostDirectiveResponse {
    pub id: String,
}

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

#[derive(serde::Deserialize)]
pub struct GetDirectivesQueryParams {
    next: u16,
    cursor: Option<String>,
}

#[actix_web::get("/directives")]
pub async fn get_directives(
    _config: Data<Config>,
    srv: Data<Server>,
    Query(query): Query<GetDirectivesQueryParams>,
) -> Result<HttpResponse, Error> {
    let keys = srv.list(query.next, query.cursor).await?;
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

#[actix_web::post("/directives")]
pub async fn post_directive(
    _config: Data<Config>,
    srv: Data<Server>,
    body: Json<PostDirectiveRequest>,
) -> Result<HttpResponse, Error> {
    let id = srv.claim_id().await?;

    let auth = match &body.auth {
        | Some(v) => Some(DirectiveACLAuth {
            key: v.key.to_owned(),
            secret: v.secret.to_owned(),
        }),
        | None => None,
    };

    srv.register(Directive {
        id: id.clone(),
        url: body.destination.clone(),
        acls: DirectiveACLs {
            expire_at: body.expire.clone(),
            calls: DirectiveACLCalls { max: None, curr: 0 },
            auth,
        },
    })
    .await?;
    Ok(HttpResponse::Created().body(serde_json::to_string(&PostDirectiveResponse { id })?))
}

#[actix_web::get("/x/{directive_id}")]
pub async fn redirect(
    _config: Data<Config>,
    request: HttpRequest,
    srv: Data<Server>,
    directive_id: Path<String>,
) -> Result<HttpResponse, Error> {
    let auth_header = request.headers().get("Authorization");
    let auth = match auth_header {
        | Some(v) => {
            let decoded_auth = String::from_utf8(base64::decode(v).unwrap()).unwrap();
            let da_parts: Vec<&str> = decoded_auth.split(":").collect();
            Some((da_parts[0].to_owned(), da_parts[1].to_owned()))
        },
        | None => None,
    };
    let dir = srv.redirect(directive_id.clone(), auth).await?;
    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", dir.url))
        .body(""))
}
