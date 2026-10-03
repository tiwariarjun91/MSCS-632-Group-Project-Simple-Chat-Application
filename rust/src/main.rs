//! Starts the local Rust chat application and coordinates graceful shutdown.

mod api;
mod chat;
mod models;
mod search;
mod simulation;
mod worker;

use std::error::Error;
use std::path::PathBuf;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use models::{ErrorCode, ErrorDetail, ErrorResponse};

const ADDRESS: &str = "127.0.0.1:8080";

/// Resolve assets relative to the Cargo project, independent of the shell's cwd.
fn frontend_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend")
}

/// Serve browser assets while keeping unknown API routes as JSON errors.
async fn frontend(request: Request) -> Response {
    let path = request.uri().path();
    if path == "/api" || path.starts_with("/api/") {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: ErrorCode::NotFound,
                    message: "API route not found.".to_owned(),
                },
            }),
        )
            .into_response();
    }

    // ServeDir handles index.html, content types, and path traversal protection.
    match ServeDir::new(frontend_directory()).try_call(request).await {
        Ok(response) => response.into_response(),
        Err(error) => {
            eprintln!("Unable to serve frontend asset: {error}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Unable to load this file.").into_response()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Bind before starting the worker so a busy port leaves no background task.
    let listener = TcpListener::bind(ADDRESS).await.map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!("Could not listen on {ADDRESS}. Stop any other backend on this port: {error}"),
        )
    })?;
    let (handle, worker_task) = worker::start();
    // Move the last local handle into the router. When the server finishes and
    // request handlers release their clones, the worker can drain and terminate.
    let app = api::router(handle).fallback(frontend);

    println!("Rust chat backend: http://{ADDRESS}");
    println!("Users API: http://{ADDRESS}/api/users");
    if !frontend_directory().join("index.html").is_file() {
        println!("Frontend not present yet; the JSON API is available.");
    }
    println!("Press Ctrl+C to stop. Message history lasts only for this session.");

    let server_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;

    // Await the worker even if the server returns an error.
    let worker_result = worker_task.await;
    server_result?;
    worker_result?;
    println!("Chat server stopped.");
    Ok(())
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => println!("Shutting down; finishing active requests..."),
        Err(error) => {
            eprintln!("Could not listen for Ctrl+C; shutting down: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};

    #[tokio::test]
    async fn missing_api_routes_never_fall_through_to_frontend_files() {
        for path in ["/api", "/api/unknown", "/api/unknown?keyword=hello"] {
            let request = Request::builder().uri(path).body(Body::empty()).unwrap();
            let response = frontend(request).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            assert_eq!(response.headers()["content-type"], "application/json");
            let body = to_bytes(response.into_body(), 4096).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["error"]["code"], "not_found");
        }
    }
}
