package chat

import (
	"fmt"
	"sort"
	"strings"
	"sync"
	"time"
)

// ChatService manages users and message history for the application.
type ChatService struct {
	users         map[string]User
	messages      []Message
	nextMessageID uint64
	mu            sync.RWMutex
}

// NewChatService creates a chat service with the default users.
func NewChatService() *ChatService {
	users := map[string]User{
		"alice": {
			UserID:      "alice",
			DisplayName: "Alice",
		},
		"bob": {
			UserID:      "bob",
			DisplayName: "Bob",
		},
		"charlie": {
			UserID:      "charlie",
			DisplayName: "Charlie",
		},
	}

	return &ChatService{
		users:         users,
		messages:      make([]Message, 0),
		nextMessageID: 1,
	}
}

// GetUsers returns all users registered in the chat service.
func (s *ChatService) GetUsers() []User {
	users := make([]User, 0, len(s.users))

	for _, user := range s.users {
		users = append(users, user)
	}

	sort.Slice(users, func(i, j int) bool {
		return users[i].UserID < users[j].UserID
	})

	return users
}

// ValidateMessage checks whether a message can be sent.
func (s *ChatService) ValidateMessage(senderID, recipientID, content string) error {
	if _, exists := s.users[senderID]; !exists {
		return fmt.Errorf("unknown sender: %s", senderID)
	}

	if _, exists := s.users[recipientID]; !exists {
		return fmt.Errorf("unknown recipient: %s", recipientID)
	}

	if senderID == recipientID {
		return fmt.Errorf("sender and recipient cannot be the same")
	}

	if strings.TrimSpace(content) == "" {
		return fmt.Errorf("message cannot be blank")
	}

	return nil
}

// SendMessage validates and stores a new message.
func (s *ChatService) SendMessage(senderID, recipientID, content string) (Message, error) {
	if err := s.ValidateMessage(senderID, recipientID, content); err != nil {
		return Message{}, err
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	message := Message{
		MessageID:   s.nextMessageID,
		SenderID:    senderID,
		RecipientID: recipientID,
		Content:     content,
		TimestampMS: time.Now().UnixMilli(),
	}

	s.messages = append(s.messages, message)
	s.nextMessageID++

	return message, nil
}

// GetMessages returns a copy of the stored message history.
func (s *ChatService) GetMessages() []Message {
	s.mu.RLock()
	defer s.mu.RUnlock()

	messages := make([]Message, len(s.messages))
	copy(messages, s.messages)

	return messages
}
