use axum::extract::{Query, Json};
use oasgen::{OaSchema, oasgen, Server};
use serde::{Deserialize};

/// Filter assets
#[derive(Deserialize, OaSchema)]
pub struct AssetFilter {
    pub owner: i32,
    // `Option<T>` query parameter must render the inner type's schema with no
    // `nullable` flag; optionality is expressed by the parameter being
    // non-required.
    pub disabled_assets: Option<bool>,
}

#[oasgen]
async fn list_assets(Query(_filter): Query<AssetFilter>) -> Json<()> {
    Json(())
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .get("/assets", list_assets)
        ;

    let spec = serde_yaml::to_string(&server.openapi).unwrap();
    let other = include_str!("05-query-option.yaml");
    assert_eq!(spec.trim(), other);
    let router = axum::Router::new()
        .merge(server.freeze().into_router());
    router.into_make_service();
}
