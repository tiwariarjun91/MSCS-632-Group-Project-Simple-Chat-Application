//! Owns session state and applies the rules for accepting messages.
//!
//! A single owner (the forthcoming channel-driven worker) will call this service.
//! Mutations require `&mut self`; queries borrow state without exposing mutation.

use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{ErrorCode, Message, SendMessageRequest, User};

/// Expected validation failures and exceptional storage failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatError {
    UnknownUser,
    SameUser,
    EmptyContent,
    ClockUnavailable,
    IdExhausted,
}

impl ChatError {
    pub fn code(self) -> ErrorCode {
        match self {
            Self::UnknownUser => ErrorCode::UnknownUser,
            Self::SameUser => ErrorCode::SameUser,
            Self::EmptyContent => ErrorCode::EmptyContent,
            Self::ClockUnavailable | Self::IdExhausted => ErrorCode::InternalError,
        }
    }
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnknownUser => "A specified user does not exist.",
            Self::SameUser => "Choose two different users.",
            Self::EmptyContent => "Message content cannot be blank.",
            Self::ClockUnavailable => "Unable to assign a message timestamp.",
            Self::IdExhausted => "No more message IDs are available.",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ChatError {}

pub struct ChatService {
    users: HashMap<u64, User>,
    history: Vec<Message>,
    next_message_id: u64,
}

impl ChatService {
    pub fn new() -> Self {
        let users = [(1, "Alice"), (2, "Bob"), (3, "Charlie")]
            .into_iter()
            .map(|(user_id, name)| {
                (
                    user_id,
                    User {
                        user_id,
                        display_name: name.to_owned(),
                    },
                )
            })
            .collect();

        Self {
            users,
            history: Vec::new(),
            next_message_id: 1,
        }
    }

    /// HashMap iteration is unordered, so sort the owned response by user ID.
    pub fn users(&self) -> Vec<User> {
        let mut users: Vec<User> = self.users.values().cloned().collect();
        users.sort_by_key(|user| user.user_id);
        users
    }

    pub fn require_user(&self, user_id: u64) -> Result<(), ChatError> {
        if self.users.contains_key(&user_id) {
            Ok(())
        } else {
            Err(ChatError::UnknownUser)
        }
    }

    /// Shared validation for sending and for querying a conversation.
    pub fn require_participants(&self, first: u64, second: u64) -> Result<(), ChatError> {
        self.require_user(first)?;
        self.require_user(second)?;
        if first == second {
            return Err(ChatError::SameUser);
        }
        Ok(())
    }

    /// A read-only borrow lets search inspect history without copying it first.
    pub fn history(&self) -> &[Message] {
        &self.history
    }

    /// Validate fully before changing state or consuming an ID.
    pub fn send(&mut self, request: SendMessageRequest) -> Result<Message, ChatError> {
        self.require_participants(request.sender_id, request.recipient_id)?;
        if request.content.trim().is_empty() {
            return Err(ChatError::EmptyContent);
        }

        let next_id = self
            .next_message_id
            .checked_add(1)
            .ok_or(ChatError::IdExhausted)?;
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ChatError::ClockUnavailable)?;
        let timestamp_ms =
            u64::try_from(elapsed.as_millis()).map_err(|_| ChatError::ClockUnavailable)?;

        let message = Message {
            message_id: self.next_message_id,
            sender_id: request.sender_id,
            recipient_id: request.recipient_id,
            // Move the String from the request, preserving the original content.
            content: request.content,
            timestamp_ms,
        };

        // History owns one copy; the caller receives an independent response.
        self.history.push(message.clone());
        self.next_message_id = next_id;
        Ok(message)
    }
}

impl Default for ChatService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(sender_id: u64, recipient_id: u64, content: &str) -> SendMessageRequest {
        SendMessageRequest {
            sender_id,
            recipient_id,
            content: content.to_owned(),
        }
    }

    #[test]
    fn seeds_users_in_stable_order_and_starts_with_empty_history() {
        let chat = ChatService::new();
        let users = chat.users();
        let entries: Vec<_> = users
            .iter()
            .map(|user| (user.user_id, user.display_name.as_str()))
            .collect();
        assert_eq!(entries, vec![(1, "Alice"), (2, "Bob"), (3, "Charlie")]);
        assert!(chat.history().is_empty());
    }

    #[test]
    fn stores_messages_in_order_with_timestamps_and_preserves_content() {
        let mut chat = ChatService::new();
        let first = chat.send(request(1, 2, "  Meeting at 3  ")).unwrap();
        let second = chat.send(request(2, 1, "See you there!")).unwrap();

        assert_eq!(first.message_id, 1);
        assert_eq!(second.message_id, 2);
        assert_eq!((first.sender_id, first.recipient_id), (1, 2));
        assert_eq!(first.content, "  Meeting at 3  ");
        assert!(first.timestamp_ms > 0);
        assert_eq!(chat.history(), &[first, second]);
    }

    #[test]
    fn rejected_messages_leave_history_and_next_id_unchanged() {
        let mut chat = ChatService::new();
        let first = chat.send(request(1, 2, "Existing message")).unwrap();
        let cases = [
            (request(99, 2, "Hello"), ChatError::UnknownUser),
            (request(1, 99, "Hello"), ChatError::UnknownUser),
            (request(0, 2, "Hello"), ChatError::UnknownUser),
            (request(1, 1, "Hello"), ChatError::SameUser),
            (request(1, 2, ""), ChatError::EmptyContent),
            (request(1, 2, " \n\t\u{2003}"), ChatError::EmptyContent),
            // User existence takes precedence over other validation failures.
            (request(99, 99, ""), ChatError::UnknownUser),
            (request(1, 1, ""), ChatError::SameUser),
        ];

        for (input, expected) in cases {
            assert_eq!(chat.send(input), Err(expected));
            assert_eq!(chat.history(), std::slice::from_ref(&first));
        }
        assert_eq!(chat.send(request(2, 3, "Valid")).unwrap().message_id, 2);
    }

    #[test]
    fn id_exhaustion_does_not_store_a_message() {
        let mut chat = ChatService::new();
        chat.next_message_id = u64::MAX;
        assert_eq!(chat.send(request(1, 2, "Hello")), Err(ChatError::IdExhausted));
        assert!(chat.history().is_empty());
    }
}
