//! HTTP/JSON boundary for the Rust chat backend.
//!
//! Handlers delegate storage and queries to the worker; they never own history.

use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::chat::ChatError;
use crate::models::{
    ConversationQuery, ErrorCode, ErrorDetail, ErrorResponse, HistoryQuery,
    MessageResponse, MessagesResponse, SendMessageRequest, SimulationResponse, UsersResponse,
};
use crate::simulation;
use crate::worker::{ChatHandle, WorkerError};

/// Build the API router. The entry point will serve frontend assets separately.
/// Unknown API paths must retain this JSON fallback when adding the frontend.
pub fn router(handle: ChatHandle) -> Router {
    Router::new()
        .route("/api/users", get(users))
        .route("/api/messages", get(history).post(send_message))
        .route("/api/conversation", get(conversation))
        .route("/api/simulation", post(simulate))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(handle)
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: ErrorCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into() }
    }

    fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            "The backend could not complete the request.",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorResponse {
            error: ErrorDetail { code: self.code, message: self.message },
        })).into_response()
    }
}

impl From<WorkerError> for ApiError {
    fn from(error: WorkerError) -> Self {
        match error {
            WorkerError::Chat(error @ (ChatError::UnknownUser
                | ChatError::SameUser | ChatError::EmptyContent)) => {
                Self::new(StatusCode::BAD_REQUEST, error.code(), error.to_string())
            }
            // Keep operational details out of browser responses.
            _ => Self::internal(),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(error: JsonRejection) -> Self {
        if error.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE {
            Self::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                ErrorCode::UnsupportedMediaType,
                "Send the message with Content-Type: application/json.",
            )
        } else {
            Self::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidRequest,
                "Provide valid JSON with integer sender_id and recipient_id and string content.",
            )
        }
    }
}

impl From<QueryRejection> for ApiError {
    fn from(_: QueryRejection) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "Provide valid query parameters and all required user IDs.",
        )
    }
}

async fn users(State(handle): State<ChatHandle>) -> Result<Json<UsersResponse>, ApiError> {
    Ok(Json(UsersResponse { users: handle.users().await? }))
}

async fn send_message(
    State(handle): State<ChatHandle>,
    body: Result<Json<SendMessageRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    let Json(input) = body?;
    let message = handle.send(input).await?;
    Ok((StatusCode::CREATED, Json(MessageResponse { message })))
}

async fn history(
    State(handle): State<ChatHandle>,
    query: Result<Query<HistoryQuery>, QueryRejection>,
) -> Result<Json<MessagesResponse>, ApiError> {
    let Query(query) = query?;
    Ok(Json(MessagesResponse { messages: handle.history(query).await? }))
}

async fn conversation(
    State(handle): State<ChatHandle>,
    query: Result<Query<ConversationQuery>, QueryRejection>,
) -> Result<Json<MessagesResponse>, ApiError> {
    let Query(query) = query?;
    Ok(Json(MessagesResponse { messages: handle.conversation(query).await? }))
}

async fn simulate(
    State(handle): State<ChatHandle>,
) -> Result<Json<SimulationResponse>, ApiError> {
    let result = simulation::run(&handle).await.map_err(|_| ApiError::internal())?;
    Ok(Json(result))
}

async fn not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, "API route not found.")
}

async fn method_not_allowed() -> ApiError {
    ApiError::new(
        StatusCode::METHOD_NOT_ALLOWED,
        ErrorCode::MethodNotAllowed,
        "This HTTP method is not supported for this route.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use crate::worker;

    #[tokio::test]
    async fn accepted_message_is_created_and_available_to_history_handler() {
        let (handle, worker) = worker::start();
        let (status, Json(response)) = send_message(
            State(handle.clone()),
            Ok(Json(SendMessageRequest {
                sender_id: 1, recipient_id: 2, content: "Meeting at 3".to_owned(),
            })),
        ).await.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        let Json(found) = history(
            State(handle.clone()),
            Ok(Query(HistoryQuery { user_id: Some(2), keyword: Some("meeting".to_owned()) })),
        ).await.unwrap();
        assert_eq!(found.messages, vec![response.message]);
        drop(handle);
        worker.await.unwrap();
    }

    #[tokio::test]
    async fn validation_failure_uses_json_error_envelope_and_leaves_history_empty() {
        let (handle, worker) = worker::start();
        let error = send_message(
            State(handle.clone()),
            Ok(Json(SendMessageRequest {
                sender_id: 1, recipient_id: 1, content: "Invalid".to_owned(),
            })),
        ).await.unwrap_err().into_response();
        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.headers()["content-type"], "application/json");
        let body = to_bytes(error.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "same_user");
        assert!(handle.history(HistoryQuery::default()).await.unwrap().is_empty());
        drop(handle);
        worker.await.unwrap();
    }

    #[test]
    fn operational_failures_map_to_generic_internal_errors() {
        for error in [
            WorkerError::Unavailable,
            WorkerError::Chat(ChatError::ClockUnavailable),
            WorkerError::Chat(ChatError::IdExhausted),
        ] {
            let api = ApiError::from(error);
            assert_eq!(api.status, StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(api.code, ErrorCode::InternalError);
            assert_eq!(api.message, "The backend could not complete the request.");
        }
    }
}
