//! Data exchanged between the Rust chat service and the shared browser interface.
//!
//! These types describe data; user lookup, validation, and history ownership belong
//! to the chat service. Serde maps field names directly to the shared JSON format.

use serde::{Deserialize, Serialize};

/// A simulated user with an ID that remains stable for the application session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct User {
    pub user_id: u64,
    pub display_name: String,
}

/// An accepted message. Only the backend assigns its ID and timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Message {
    pub message_id: u64,
    pub sender_id: u64,
    pub recipient_id: u64,
    pub content: String,
    /// Milliseconds since the Unix epoch, recorded at backend acceptance.
    pub timestamp_ms: u64,
}

/// The JSON body for sending a message, before service-level validation.
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub sender_id: u64,
    pub recipient_id: u64,
    pub content: String,
}

/// Optional history filters. When both are present, both must match.
#[derive(Debug, Default, Deserialize)]
pub struct HistoryQuery {
    pub user_id: Option<u64>,
    pub keyword: Option<String>,
}

/// Required participants for a conversation in either direction.
#[derive(Debug, Deserialize)]
pub struct ConversationQuery {
    pub user_id: u64,
    pub other_user_id: u64,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: Message,
}

/// Used for history, filtered results, and conversations; no matches means `[]`.
#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}

/// Contains only the messages accepted during this simulation invocation.
#[derive(Debug, Serialize)]
pub struct SimulationResponse {
    pub accepted_count: usize,
    pub messages: Vec<Message>,
}

/// Typed error codes prevent inconsistent strings across API handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    UnknownUser,
    SameUser,
    EmptyContent,
    NotFound,
    MethodNotAllowed,
    UnsupportedMediaType,
    InternalError,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
}

/// Consistent JSON envelope for validation, routing, and internal errors.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}
