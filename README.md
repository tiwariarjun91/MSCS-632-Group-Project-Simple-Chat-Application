# Simple Chat Application in Rust and Go

This MSCS 632 group project implements the same local chat application in Rust
and Go. Both backends will use the shared browser interface in `frontend/`,
allowing the project to compare the languages without changing the user
experience.

## Project status

- **Rust backend:** Core Day 2 functionality is implemented and tested.
- **Shared frontend:** Implemented and integrated with the Rust backend.
- **Go backend:** Owned by Arjun and currently under development.

The README will be updated with equivalent Go commands after that backend is
integrated. The documented Rust commands below work with the current repository.

## Features

- Three simulated users: Alice, Bob, and Charlie
- Direct messages with unique IDs and millisecond timestamps
- Conversation history between two selected users
- History filtering by participant
- Case-insensitive keyword search for English letters
- Validation for unknown users, self-messages, and blank messages
- Concurrent simulation using three Tokio tasks and channels
- In-memory session storage with deterministic message-ID ordering
- Shared responsive HTML, CSS, and JavaScript interface
- Graceful shutdown with `Ctrl+C`

## Repository structure

```text
.
├── frontend/
│   ├── index.html       Shared browser interface
│   ├── styles.css       Responsive presentation
│   └── app.js           API requests and UI behavior
├── rust/
│   ├── Cargo.toml       Rust package and dependencies
│   ├── Cargo.lock       Locked dependency versions
│   └── src/
│       ├── main.rs      Server startup, static files, and shutdown
│       ├── models.rs    User, message, request, and response types
│       ├── chat.rs      Validation and in-memory message storage
│       ├── search.rs    Conversation, filter, and keyword queries
│       ├── worker.rs    Channel-based owner of mutable chat state
│       ├── simulation.rs Concurrent simulated message senders
│       └── api.rs       HTTP routes and JSON error responses
└── go-backend/          Go implementation owned by Arjun
```

## Rust prerequisites

Install the stable Rust toolchain with `rustup` from
<https://www.rust-lang.org/tools/install>. Verify the installation:

```bash
rustc --version
cargo --version
```

If a newly installed `cargo` command is not available in the current terminal,
reload the shell configuration:

```bash
source "$HOME/.cargo/env"
```

## Run the Rust application

From the repository root:

```bash
cargo run --manifest-path rust/Cargo.toml
```

Open <http://127.0.0.1:8080> in a browser. The backend serves the shared frontend
and API from the same local address. Only one backend can use port `8080` at a
time.

Press `Ctrl+C` in the terminal to stop the application cleanly. Message history
is stored only in memory and is cleared whenever the backend restarts.

## Use the application

1. Select an active user and a different recipient.
2. Enter a message and select **Send message**.
3. Review the two-user conversation on the left.
4. Filter all history by participant or search by keyword on the right.
5. Select **Run simulation** to add six messages from three concurrent users.

The UI reports success only after the backend accepts and stores a message.

## Test and verify the Rust backend

Run all automated tests:

```bash
cargo test --manifest-path rust/Cargo.toml --locked
```

Check formatting without changing files:

```bash
cargo fmt --manifest-path rust/Cargo.toml -- --check
```

Run Clippy and treat warnings as errors:

```bash
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
```

The Rust implementation currently has 22 unit and async tests covering message
validation, storage, search, conversation queries, API errors, concurrent sends,
simulation behavior, and shutdown-related worker behavior.

## HTTP API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/users` | List the simulated users |
| `POST` | `/api/messages` | Validate and store a message |
| `GET` | `/api/messages` | Return history with optional filters |
| `GET` | `/api/conversation` | Return messages between two users |
| `POST` | `/api/simulation` | Run the concurrent six-message simulation |

Example message request:

```json
{
  "sender_id": 1,
  "recipient_id": 2,
  "content": "Meeting at 3"
}
```

Optional history query parameters are `user_id` and `keyword`. Conversation
queries require `user_id` and `other_user_id`.

## Rust language features demonstrated

- Structs and enums model chat data and typed errors.
- `Result` values and pattern matching handle expected failures explicitly.
- Ownership keeps one task responsible for mutable message history.
- Borrowed slices and iterators support read-only queries.
- Tokio tasks, bounded channels, and one-shot channels coordinate concurrent work.
- Checked arithmetic and explicit timestamp conversion prevent silent overflow.
- Serde converts strongly typed Rust data to and from JSON.

## Current limitations

- The application is local and does not provide internet-based messaging.
- Users are fixed to Alice, Bob, and Charlie.
- History is not persisted to a database or file.
- Authentication, attachments, and group conversations are outside the project scope.
- Concurrent task scheduling may change simulation interleaving between runs, while
  message IDs remain unique and results remain ordered.

## Team responsibilities

- **Ruthwik Pala:** Rust backend, Rust tests, Rust/frontend integration, and Rust
  build/run/test documentation.
- **Arjun Nandkishor Tiwari:** Go backend, Go tests, and Go documentation.
- **Shared:** Frontend review, matching behavior, screenshots, comparison report,
  and presentation.
