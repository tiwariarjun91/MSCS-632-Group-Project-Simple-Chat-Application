package main

import (
	"fmt"
	"strings"
)

// ChatService manages users and message history for the application.
type ChatService struct {
	users    map[string]User
	messages []Message
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
		users:    users,
		messages: make([]Message, 0),
	}
}

// GetUsers returns all users registered in the chat service.
func (s *ChatService) GetUsers() []User {
	users := make([]User, 0, len(s.users))

	for _, user := range s.users {
		users = append(users, user)
	}

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
