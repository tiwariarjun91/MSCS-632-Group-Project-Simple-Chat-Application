# Simple Chat Application in Rust and Go

This MSCS 632 group project implements the same local chat application in Rust
and Go. Both backends use the shared browser interface in `frontend/`, allowing
the project to compare the languages without changing the user experience.

## Project status

- **Rust backend:** Core Day 2 functionality is implemented and tested.
- **Go backend:** Core Day 2 functionality is implemented and tested.
- **Shared frontend:** Implemented and integrated with both backends.

Both implementations expose the same HTTP API and use the same user IDs and
browser interface.

## Features

- Three simulated users with shared IDs: `alice`, `bob`, and `charlie`
- Direct messages with unique IDs and millisecond timestamps
- Conversation history between two selected users
- History filtering by participant
- Case-insensitive keyword search
- Validation for unknown users, self-messages, and blank messages
- Concurrent six-message simulation
- In-memory session storage with unique message IDs
- Shared responsive HTML, CSS, and JavaScript interface

## Repository structure

```text
.
├── frontend/
│   ├── index.html       Shared browser interface
│   ├── styles.css       Responsive presentation
│   └── app.js           API requests and UI behavior
├── rust-backend/
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
└── go-backend/
    ├── go.mod
    ├── cmd/
    │   └── server/
    │       └── main.go      Go server entry point
    └── internal/
        ├── api/
        │   ├── api.go       HTTP routes and JSON responses
        │   └── api_test.go  API tests
        └── chat/
            ├── models.go     User and message types
            ├── chat.go       Validation and message storage
            ├── search.go     Conversation and search logic
            ├── simulation.go Concurrent chat simulation
            └── chat_test.go  Chat service tests
```

## Rust prerequisites

Install the stable Rust toolchain with `rustup`. Verify the installation:

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
cargo run --manifest-path rust-backend/Cargo.toml
```

Open `http://127.0.0.1:8080` in a browser. The backend serves the shared frontend
and API from the same local address.

Press `Ctrl+C` in the terminal to stop the application. Message history is
stored only in memory and is cleared whenever the backend restarts.

## Go prerequisites

Install Go and verify the installation:

```bash
go version
```

## Run the Go application

From the repository root:

```bash
cd go-backend
go run ./cmd/server
```

The server starts at:

```text
http://127.0.0.1:8080
```

Open that address in a browser to use the shared chat interface.

Press `Ctrl+C` to stop the server. Message history is stored only in memory and
is cleared when the backend restarts.

Only one backend can use port `8080` at a time, so stop the Rust backend before
starting Go, or stop Go before starting Rust.

## Use the application

1. Select an active user and a different recipient.
2. Enter a message and select **Send message**.
3. Review the two-user conversation on the left.
4. Filter all history by participant or search by keyword on the right.
5. Select **Run simulation** to add six messages from three concurrent users.

The UI reports success only after the selected backend accepts and stores a
message.

## Test and verify the Rust backend

Run all automated tests:

```bash
cargo test --manifest-path rust-backend/Cargo.toml --locked
```

Check formatting without changing files:

```bash
cargo fmt --manifest-path rust-backend/Cargo.toml -- --check
```

Run Clippy and treat warnings as errors:

```bash
cargo clippy --manifest-path rust-backend/Cargo.toml --all-targets -- -D warnings
```

The Rust implementation includes tests covering message validation, storage,
search, conversation queries, API errors, concurrent sends, simulation behavior,
and worker behavior.

## Test and verify the Go backend

From the `go-backend` directory, run:

```bash
go test ./...
```

Run Go's static analysis:

```bash
go vet ./...
```

Run the tests with Go's race detector:

```bash
go test -race ./...
```

The Go tests cover HTTP API behavior and core chat service behavior, including
message sending, validation, message IDs, history, conversation queries,
filtering, and concurrent simulation behavior.

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
  "sender_id": "alice",
  "recipient_id": "bob",
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

## Go language features demonstrated

- Structs model users and messages with JSON field tags.
- Explicit `error` values handle message validation failures.
- Sentinel errors allow the API layer to identify specific validation failures.
- Slices store in-memory message history.
- `sync.RWMutex` protects shared message history and message IDs.
- Goroutines allow simulated users to send messages concurrently.
- Channels collect results from concurrent simulation workers.
- Go's race detector helps verify concurrent access to shared state.

## Current limitations

- The application is local and does not provide internet-based messaging.
- Users are fixed to Alice, Bob, and Charlie.
- History is not persisted to a database or file.
- Authentication, attachments, and group conversations are outside the project scope.
- Concurrent scheduling may change simulation interleaving between runs, while
  message IDs remain unique.

## Team responsibilities

- **Ruthwik Pala:** Rust backend, Rust tests, Rust/frontend integration, and Rust
  build/run/test documentation.
- **Arjun Nandkishor Tiwari:** Go backend, Go tests, Go/frontend integration, and
  Go build/run/test documentation.
- **Shared:** Frontend review, matching behavior, screenshots, comparison report,
  and presentation.
