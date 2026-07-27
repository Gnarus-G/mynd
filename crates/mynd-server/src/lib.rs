use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use todo::{persist::TodosDatabase, Todo, Todos};

#[derive(RustEmbed)]
#[folder = "../../build/"]
struct Assets;

pub fn app<DB>(todos: Todos<DB>) -> Router
where
    DB: TodosDatabase + Send + Sync + 'static,
{
    Router::new()
        .route("/api/health", get(health))
        .route("/api/config", get(config))
        .route("/api/todos", get(list::<DB>).post(add::<DB>))
        .route("/api/todos/completed", delete(delete_completed::<DB>))
        .route("/api/todos/{id}", delete(delete_one::<DB>))
        .route("/api/todos/{id}/complete", post(complete::<DB>))
        .route("/api/todos/{id}/move-up", post(move_up::<DB>))
        .route("/api/todos/{id}/move-down", post(move_down::<DB>))
        .route("/api/todos/{id}/move-below", post(move_below::<DB>))
        .fallback(get(asset))
        .with_state(Arc::new(todos))
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[derive(Serialize)]
struct PublicConfig {
    web_url: Option<String>,
}

async fn config() -> Json<PublicConfig> {
    Json(PublicConfig {
        web_url: todo::config::load_config()
            .unwrap_or_default()
            .web_url
            .filter(|url| url.starts_with("https://")),
    })
}

async fn list<DB>(State(todos): State<Arc<Todos<DB>>>) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    Ok(Json(todos.get_all().map_err(ApiError::from)?))
}

#[derive(Deserialize)]
struct AddTodo {
    message: String,
}

async fn add<DB>(
    State(todos): State<Arc<Todos<DB>>>,
    Json(input): Json<AddTodo>,
) -> ApiResult<(StatusCode, Json<Vec<Todo>>)>
where
    DB: TodosDatabase,
{
    todos
        .add_message(input.message.trim())
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(todos.get_all().map_err(ApiError::from)?),
    ))
}

async fn complete<DB>(
    State(todos): State<Arc<Todos<DB>>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    todos.mark_done(&id).map_err(ApiError::from)?;
    updated(&todos)
}

async fn delete_one<DB>(
    State(todos): State<Arc<Todos<DB>>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    todos.remove(&id).map_err(ApiError::from)?;
    updated(&todos)
}

async fn delete_completed<DB>(State(todos): State<Arc<Todos<DB>>>) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    todos.remove_done().map_err(ApiError::from)?;
    updated(&todos)
}

async fn move_up<DB>(
    State(todos): State<Arc<Todos<DB>>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    todos.move_up(id).map_err(ApiError::from)?;
    updated(&todos)
}

async fn move_down<DB>(
    State(todos): State<Arc<Todos<DB>>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    todos.move_down(id).map_err(ApiError::from)?;
    updated(&todos)
}

#[derive(Deserialize)]
struct MoveBelow {
    target_id: String,
}

async fn move_below<DB>(
    State(todos): State<Arc<Todos<DB>>>,
    Path(id): Path<String>,
    Json(input): Json<MoveBelow>,
) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    todos
        .move_below(&id, &input.target_id)
        .map_err(ApiError::from)?;
    updated(&todos)
}

fn updated<DB>(todos: &Todos<DB>) -> ApiResult<Json<Vec<Todo>>>
where
    DB: TodosDatabase,
{
    Ok(Json(todos.get_all().map_err(ApiError::from)?))
}

async fn asset(uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    if requested.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };
    let (path, asset) = Assets::get(path)
        .map(|asset| (path, asset))
        .or_else(|| Assets::get("index.html").map(|asset| ("index.html", asset)))
        .expect("frontend build must contain index.html");
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let cache_control =
        if path == "index.html" || path.ends_with("sw.js") || path.ends_with(".webmanifest") {
            "no-cache"
        } else {
            "public, max-age=31536000, immutable"
        };

    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CACHE_CONTROL, cache_control)
        .body(Body::from(asset.data))
        .expect("valid static asset response")
}

type ApiResult<T> = Result<T, ApiError>;

struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        eprintln!("[ERROR] {error:#}");
        let status = if error.to_string().contains("didn't find") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        let message = if status == StatusCode::NOT_FOUND {
            "todo not found"
        } else {
            "failed to update todos"
        };
        Self { status, message }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}
