use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mynd_server::app;
use todo::{persist::binary::TodosBin, Todos};
use tower::ServiceExt;

#[tokio::test]
async fn todo_can_be_added_and_loaded_over_http() {
    let directory = tempfile::tempdir().unwrap();
    let state = Todos::new(TodosBin::at(directory.path().join("todos.bin")));
    let app = app(state);

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"call mom"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let response = app
        .clone()
        .oneshot(Request::get("/api/todos").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let todos: Vec<todo::Todo> = serde_json::from_slice(&body).unwrap();

    assert_eq!(todos[0].message, "call mom");

    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/api/todos/{}/complete", todos[0].id.0))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let completed: Vec<todo::Todo> = serde_json::from_slice(&body).unwrap();
    assert!(completed[0].done);

    let response = app
        .oneshot(
            Request::delete("/api/todos/completed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let remaining: Vec<todo::Todo> = serde_json::from_slice(&body).unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn embedded_pwa_serves_its_manifest_and_navigation_fallback() {
    let app = app(Todos::new_inmemory());

    for path in ["/manifest.webmanifest", "/some/client/route"] {
        let response = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        if path == "/manifest.webmanifest" {
            assert_eq!(response.headers()["cache-control"], "no-cache");
        }
    }
}
