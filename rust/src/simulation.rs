//! Concurrent simulated users send through the same worker as normal requests.

use std::fmt;

use tokio::task::JoinSet;

use crate::models::{SendMessageRequest, SimulationResponse};
use crate::worker::{ChatHandle, WorkerError};

#[derive(Debug)]
pub enum SimulationError {
    Worker(WorkerError),
    TaskFailed(tokio::task::JoinError),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Worker(error) => write!(f, "Simulation could not send a message: {error}"),
            Self::TaskFailed(_) => f.write_str("A simulation task failed to complete."),
        }
    }
}

impl std::error::Error for SimulationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Worker(error) => Some(error),
            Self::TaskFailed(error) => Some(error),
        }
    }
}

/// Append six messages using three independently scheduled Tokio tasks.
///
/// Each task awaits acceptance of its first message before sending its second.
/// Ordering between users can vary. The response contains only this invocation's
/// accepted messages, sorted by ID, even when other clients send concurrently.
/// On failure, finish joining the tasks and return an error; already stored
/// messages remain in history. Cancelling this future drops the JoinSet and
/// aborts its tasks, but commands already queued at the worker may still complete.
pub async fn run(handle: &ChatHandle) -> Result<SimulationResponse, SimulationError> {
    let mut tasks = JoinSet::new();
    for (sender_id, recipient_id, name) in [(1, 2, "Alice"), (2, 3, "Bob"), (3, 1, "Charlie")] {
        let client = handle.clone();
        tasks.spawn(async move {
            let mut accepted = Vec::with_capacity(2);
            for number in 1..=2 {
                let message = client
                    .send(SendMessageRequest {
                        sender_id,
                        recipient_id,
                        content: format!("Simulation {name} {number}"),
                    })
                    .await?;
                accepted.push(message);
            }
            Ok::<_, WorkerError>(accepted)
        });
    }

    let mut messages = Vec::with_capacity(6);
    let mut first_error = None;
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(accepted)) => messages.extend(accepted),
            Ok(Err(error)) => {
                first_error.get_or_insert(SimulationError::Worker(error));
            }
            Err(error) => {
                first_error.get_or_insert(SimulationError::TaskFailed(error));
            }
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }
    messages.sort_by_key(|message| message.message_id);
    Ok(SimulationResponse {
        accepted_count: messages.len(),
        messages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HistoryQuery;
    use crate::worker;

    #[tokio::test]
    async fn simulation_stores_six_messages_with_expected_participants_and_content() {
        let (handle, worker) = worker::start();
        let result = run(&handle).await.unwrap();
        assert_eq!(result.accepted_count, 6);
        assert_eq!(result.messages.len(), 6);
        assert_eq!(
            result.messages,
            handle.history(HistoryQuery::default()).await.unwrap()
        );
        let ids: Vec<_> = result
            .messages
            .iter()
            .map(|message| message.message_id)
            .collect();
        assert_eq!(ids, vec![1, 2, 3, 4, 5, 6]);

        for (sender_id, recipient_id, name) in [(1, 2, "Alice"), (2, 3, "Bob"), (3, 1, "Charlie")] {
            let sent: Vec<_> = result
                .messages
                .iter()
                .filter(|message| message.sender_id == sender_id)
                .collect();
            assert_eq!(sent.len(), 2);
            for (index, message) in sent.iter().enumerate() {
                assert_eq!(message.recipient_id, recipient_id);
                assert_eq!(message.content, format!("Simulation {name} {}", index + 1));
                assert!(message.timestamp_ms > 0);
            }
        }

        drop(handle);
        worker.await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn overlapping_and_repeated_runs_preserve_history_and_separate_results() {
        let (handle, worker) = worker::start();
        let original = handle
            .send(SendMessageRequest {
                sender_id: 1,
                recipient_id: 2,
                content: "Existing message".to_owned(),
            })
            .await
            .unwrap();

        let (first, second, normal) = tokio::join!(
            run(&handle),
            run(&handle),
            handle.send(SendMessageRequest {
                sender_id: 2,
                recipient_id: 1,
                content: "Concurrent normal message".to_owned(),
            })
        );
        let first = first.unwrap();
        let second = second.unwrap();
        let normal = normal.unwrap();
        let third = run(&handle).await.unwrap();
        let mut expected = vec![original, normal];
        for result in [first, second, third] {
            assert_eq!(result.accepted_count, 6);
            assert_eq!(result.messages.len(), 6);
            assert!(
                result
                    .messages
                    .windows(2)
                    .all(|pair| pair[0].message_id < pair[1].message_id)
            );
            expected.extend(result.messages);
        }
        expected.sort_by_key(|message| message.message_id);
        let stored = handle.history(HistoryQuery::default()).await.unwrap();
        assert_eq!(stored, expected);
        let ids: Vec<_> = stored.iter().map(|message| message.message_id).collect();
        assert_eq!(ids, (1..=20).collect::<Vec<u64>>());

        drop(handle);
        worker.await.unwrap();
    }

    #[tokio::test]
    async fn unavailable_worker_returns_an_error_instead_of_success() {
        let (handle, worker) = worker::start();
        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        assert!(matches!(
            run(&handle).await,
            Err(SimulationError::Worker(WorkerError::Unavailable))
        ));
    }
}
