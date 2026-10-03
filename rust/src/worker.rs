//! Channel-based access to a single owner of mutable chat state.
//!
//! HTTP handlers and simulated senders clone `ChatHandle`, not `ChatService`.
//! A bounded queue provides backpressure. Each command carries a one-use reply
//! channel, so a successful send means the worker has already stored the message.

use std::fmt;

use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::chat::{ChatError, ChatService};
use crate::models::{ConversationQuery, HistoryQuery, Message, SendMessageRequest, User};
use crate::search;

const QUEUE_CAPACITY: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerError {
    Chat(ChatError),
    Unavailable,
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chat(error) => write!(f, "{error}"),
            Self::Unavailable => f.write_str("The chat worker is unavailable."),
        }
    }
}

impl std::error::Error for WorkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Chat(error) => Some(error),
            Self::Unavailable => None,
        }
    }
}

type Reply<T> = oneshot::Sender<Result<T, ChatError>>;

/// Each variant pairs its input with a reply channel of the correct type.
enum Command {
    Users(Reply<Vec<User>>),
    Send(SendMessageRequest, Reply<Message>),
    History(HistoryQuery, Reply<Vec<Message>>),
    Conversation(ConversationQuery, Reply<Vec<Message>>),
}

#[derive(Clone)]
pub struct ChatHandle {
    sender: mpsc::Sender<Command>,
}

impl ChatHandle {
    async fn request<T>(
        &self,
        command: impl FnOnce(Reply<T>) -> Command,
    ) -> Result<T, WorkerError> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(command(reply))
            .await
            .map_err(|_| WorkerError::Unavailable)?;
        response
            .await
            .map_err(|_| WorkerError::Unavailable)?
            .map_err(WorkerError::Chat)
    }

    pub async fn users(&self) -> Result<Vec<User>, WorkerError> {
        self.request(Command::Users).await
    }

    pub async fn send(&self, input: SendMessageRequest) -> Result<Message, WorkerError> {
        self.request(|reply| Command::Send(input, reply)).await
    }

    pub async fn history(&self, query: HistoryQuery) -> Result<Vec<Message>, WorkerError> {
        self.request(|reply| Command::History(query, reply)).await
    }

    pub async fn conversation(
        &self,
        query: ConversationQuery,
    ) -> Result<Vec<Message>, WorkerError> {
        self.request(|reply| Command::Conversation(query, reply))
            .await
    }
}

/// Start inside a Tokio runtime and retain the task handle for shutdown.
///
/// For clean shutdown, stop incoming HTTP requests, finish active work, drop all
/// `ChatHandle` clones, and then await the returned task. The worker drains queued
/// commands before exiting when all senders are gone.
pub fn start() -> (ChatHandle, JoinHandle<()>) {
    let (sender, mut receiver) = mpsc::channel(QUEUE_CAPACITY);
    let task = tokio::spawn(async move {
        let mut chat = ChatService::new();
        while let Some(command) = receiver.recv().await {
            // A disconnected caller must not crash the worker. Once queued, a
            // send can still be accepted even if its caller stops awaiting it.
            match command {
                Command::Users(reply) => {
                    let _ = reply.send(Ok(chat.users()));
                }
                Command::Send(input, reply) => {
                    let _ = reply.send(chat.send(input));
                }
                Command::History(query, reply) => {
                    let _ = reply.send(search::history(&chat, &query));
                }
                Command::Conversation(query, reply) => {
                    let _ = reply.send(search::conversation(&chat, &query));
                }
            }
        }
    });
    (ChatHandle { sender }, task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::task::JoinSet;

    fn message(sender_id: u64, recipient_id: u64, content: &str) -> SendMessageRequest {
        SendMessageRequest {
            sender_id,
            recipient_id,
            content: content.to_owned(),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_senders_store_every_message_with_unique_ordered_ids() {
        let (handle, worker) = start();
        let mut senders = JoinSet::new();
        for index in 0..100 {
            let client = handle.clone();
            senders.spawn(async move {
                client
                    .send(message(1, 2, &format!("Message {index}")))
                    .await
                    .unwrap()
            });
        }

        let mut accepted = Vec::new();
        while let Some(result) = senders.join_next().await {
            accepted.push(result.unwrap());
        }
        accepted.sort_by_key(|message| message.message_id);
        let stored = handle.history(HistoryQuery::default()).await.unwrap();
        assert_eq!(stored.len(), 100);
        assert_eq!(stored, accepted);
        let ids: Vec<_> = stored.iter().map(|message| message.message_id).collect();
        assert_eq!(ids, (1..=100).collect::<Vec<u64>>());

        drop(handle);
        worker.await.unwrap();
    }

    #[tokio::test]
    async fn validation_errors_do_not_stop_worker_and_queries_use_stored_state() {
        let (handle, worker) = start();
        assert_eq!(handle.users().await.unwrap().len(), 3);
        assert_eq!(
            handle.send(message(1, 1, "Invalid")).await,
            Err(WorkerError::Chat(ChatError::SameUser))
        );
        let accepted = handle.send(message(1, 2, "Meeting at 3")).await.unwrap();
        assert_eq!(accepted.message_id, 1);
        let filtered = handle
            .history(HistoryQuery {
                user_id: Some(2),
                keyword: Some("meeting".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(filtered, vec![accepted.clone()]);
        let conversation = handle
            .conversation(ConversationQuery {
                user_id: 2,
                other_user_id: 1,
            })
            .await
            .unwrap();
        assert_eq!(conversation, vec![accepted]);

        drop(handle);
        worker.await.unwrap();
    }

    #[tokio::test]
    async fn stopped_worker_returns_unavailable() {
        let (handle, worker) = start();
        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        assert_eq!(handle.users().await, Err(WorkerError::Unavailable));
    }

    #[tokio::test]
    async fn shutdown_drains_queued_commands_even_if_a_caller_disconnects() {
        let (handle, worker) = start();
        let (abandoned_reply, abandoned_response) = oneshot::channel();
        drop(abandoned_response);
        handle
            .sender
            .send(Command::Send(
                message(1, 2, "Caller disconnected"),
                abandoned_reply,
            ))
            .await
            .unwrap();

        let (reply, response) = oneshot::channel();
        handle
            .sender
            .send(Command::History(HistoryQuery::default(), reply))
            .await
            .unwrap();
        drop(handle);
        worker.await.unwrap();

        let history = response.await.unwrap().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Caller disconnected");
    }
}
