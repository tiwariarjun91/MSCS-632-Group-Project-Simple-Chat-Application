//! Read-only queries over the chat service's session history.
//!
//! Iterators borrow stored messages and clone only matching results, giving API
//! callers owned responses without allowing them to modify the stored history.

use crate::chat::{ChatError, ChatService};
use crate::models::{ConversationQuery, HistoryQuery, Message};

/// Return history with optional participant and keyword restrictions.
///
/// A participant matches either sender or recipient. Both restrictions must match
/// when supplied. ASCII letters are case-insensitive; other characters are
/// literal. An empty keyword matches all content, and whitespace is preserved.
pub fn history(chat: &ChatService, query: &HistoryQuery) -> Result<Vec<Message>, ChatError> {
    if let Some(user_id) = query.user_id {
        chat.require_user(user_id)?;
    }
    let keyword = query.keyword.as_ref().map(|text| text.to_ascii_lowercase());

    let messages = chat
        .history()
        .iter()
        .filter(|message| match query.user_id {
            Some(user_id) => message.sender_id == user_id || message.recipient_id == user_id,
            None => true,
        })
        .filter(|message| match keyword.as_deref() {
            Some("") | None => true,
            Some(keyword) => message.content.to_ascii_lowercase().contains(keyword),
        })
        .cloned()
        .collect();

    // Filtering preserves the service's ascending message-ID order.
    Ok(messages)
}

/// Return both directions of a conversation between two distinct known users.
pub fn conversation(
    chat: &ChatService,
    query: &ConversationQuery,
) -> Result<Vec<Message>, ChatError> {
    chat.require_participants(query.user_id, query.other_user_id)?;

    Ok(chat
        .history()
        .iter()
        .filter(|message| {
            (message.sender_id == query.user_id && message.recipient_id == query.other_user_id)
                || (message.sender_id == query.other_user_id
                    && message.recipient_id == query.user_id)
        })
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SendMessageRequest;

    fn sample_chat() -> ChatService {
        let mut chat = ChatService::new();
        for (sender_id, recipient_id, content) in [
            (1, 2, "Meeting at 3"),
            (2, 1, "See you there"),
            (3, 1, "MEETING notes"),
            (2, 3, "Lunch plans"),
        ] {
            chat.send(SendMessageRequest {
                sender_id,
                recipient_id,
                content: content.to_owned(),
            })
            .unwrap();
        }
        chat
    }

    fn ids(messages: &[Message]) -> Vec<u64> {
        messages.iter().map(|message| message.message_id).collect()
    }

    fn query(user_id: Option<u64>, keyword: Option<&str>) -> HistoryQuery {
        HistoryQuery {
            user_id,
            keyword: keyword.map(str::to_owned),
        }
    }

    #[test]
    fn unrestricted_history_preserves_order_and_returns_independent_values() {
        let chat = sample_chat();
        let mut results = history(&chat, &HistoryQuery::default()).unwrap();
        assert_eq!(results, chat.history());
        assert_eq!(ids(&results), vec![1, 2, 3, 4]);
        results[0].content.clear();
        assert_eq!(chat.history()[0].content, "Meeting at 3");
    }

    #[test]
    fn participant_filter_matches_sender_and_recipient() {
        let chat = sample_chat();
        let results = history(&chat, &query(Some(3), None)).unwrap();
        assert_eq!(ids(&results), vec![3, 4]);
    }

    #[test]
    fn keyword_is_a_case_insensitive_substring_and_combines_with_participant() {
        let chat = sample_chat();
        let results = history(&chat, &query(None, Some("eEtInG"))).unwrap();
        assert_eq!(ids(&results), vec![1, 3]);
        let combined = history(&chat, &query(Some(2), Some("meeting"))).unwrap();
        assert_eq!(ids(&combined), vec![1]);
        assert!(
            history(&chat, &query(None, Some("missing")))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn empty_keyword_adds_no_restriction_and_whitespace_is_literal() {
        let chat = sample_chat();
        assert_eq!(
            history(&chat, &query(None, Some(""))).unwrap(),
            chat.history()
        );
        let results = history(&chat, &query(None, Some("Meeting "))).unwrap();
        assert_eq!(ids(&results), vec![1, 3]);
        assert!(
            history(&chat, &query(None, Some(" meeting")))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn non_ascii_characters_match_literally() {
        let mut chat = ChatService::new();
        chat.send(SendMessageRequest {
            sender_id: 1,
            recipient_id: 2,
            content: "CAFÉ".to_owned(),
        })
        .unwrap();
        assert_eq!(
            ids(&history(&chat, &query(None, Some("cafÉ"))).unwrap()),
            vec![1]
        );
        assert!(
            history(&chat, &query(None, Some("café")))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn conversation_includes_both_directions_and_excludes_third_participant() {
        let chat = sample_chat();
        for (user_id, other_user_id) in [(1, 2), (2, 1)] {
            let results = conversation(
                &chat,
                &ConversationQuery {
                    user_id,
                    other_user_id,
                },
            )
            .unwrap();
            assert_eq!(ids(&results), vec![1, 2]);
        }
    }

    #[test]
    fn empty_history_still_validates_users_and_conversation_participants() {
        let chat = ChatService::new();
        assert!(history(&chat, &query(Some(1), None)).unwrap().is_empty());
        assert_eq!(
            history(&chat, &query(Some(99), None)),
            Err(ChatError::UnknownUser)
        );
        for (user_id, other_user_id, expected) in [
            (99, 2, ChatError::UnknownUser),
            (1, 99, ChatError::UnknownUser),
            (99, 99, ChatError::UnknownUser),
            (1, 1, ChatError::SameUser),
        ] {
            assert_eq!(
                conversation(
                    &chat,
                    &ConversationQuery {
                        user_id,
                        other_user_id
                    }
                ),
                Err(expected)
            );
        }
        assert!(
            conversation(
                &chat,
                &ConversationQuery {
                    user_id: 1,
                    other_user_id: 2
                }
            )
            .unwrap()
            .is_empty()
        );
    }
}
