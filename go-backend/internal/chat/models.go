package chat

// User represents a participant in the chat application.
type User struct {
	UserID      string `json:"user_id"`
	DisplayName string `json:"display_name"`
}

// Message represents a message exchanged between two users.
type Message struct {
	MessageID   uint64 `json:"message_id"`
	SenderID    string `json:"sender_id"`
	RecipientID string `json:"recipient_id"`
	Content     string `json:"content"`
	TimestampMS int64  `json:"timestamp_ms"`
}
