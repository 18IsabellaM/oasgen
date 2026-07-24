use axum::extract::Path;
use axum::http::StatusCode;
use oasgen::{Server, oasgen};

/// A handler that returns no body on success still documents a 200
#[oasgen]
async fn delete_task(Path(id): Path<u64>) -> Result<(), (StatusCode, String)> {
    if id == 0 {
        return Err((StatusCode::NOT_FOUND, "Task not found".to_string()));
    }
    Ok(())
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum().delete("/tasks/{id}", delete_task);

    let spec = serde_yaml::to_string(&server.openapi).unwrap();
    let other = include_str!("07-unit-response.yaml");
    assert_eq!(spec.trim(), other.trim());
    let router = axum::Router::new().merge(server.freeze().into_router());
    router.into_make_service();
}
