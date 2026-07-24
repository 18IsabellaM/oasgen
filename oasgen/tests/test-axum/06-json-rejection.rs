use axum::{Json, http::StatusCode};
use oasgen::{OaSchema, Server, oasgen};
use serde::{Deserialize, Serialize};

/// A task to create
#[derive(Deserialize, OaSchema)]
pub struct NewTask {
    /// Human readable title
    pub title: String,
}

/// The created task
#[derive(Serialize, OaSchema)]
pub struct CreatedTask {
    /// Identifier assigned by the server
    pub id: u32,
}

/// Both `Json` rejection codes are documented automatically
#[oasgen]
async fn create_task(Json(_task): Json<NewTask>) -> Json<CreatedTask> {
    Json(CreatedTask { id: 1 })
}

/// A handler documenting 422 itself keeps its own description
#[oasgen]
async fn replace_task(
    Json(task): Json<NewTask>,
) -> Result<Json<CreatedTask>, (StatusCode, String)> {
    if task.title.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Task title cannot be empty".to_string(),
        ));
    }
    Ok(Json(CreatedTask { id: 1 }))
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .post("/tasks", create_task)
        .put("/tasks", replace_task);

    let spec = serde_yaml::to_string(&server.openapi).unwrap();
    let other = include_str!("06-json-rejection.yaml");
    assert_eq!(spec.trim(), other.trim());
    let router = axum::Router::new().merge(server.freeze().into_router());
    router.into_make_service();
}
