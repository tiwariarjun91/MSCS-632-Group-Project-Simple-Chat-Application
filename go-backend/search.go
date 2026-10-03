package main

import "strings"

// GetConversation returns messages exchanged between two users.
func (s *ChatService) GetConversation(userID, otherUserID string) []Message {
	messages := make([]Message, 0)

	for _, message := range s.messages {
		if (message.SenderID == userID && message.RecipientID == otherUserID) ||
			(message.SenderID == otherUserID && message.RecipientID == userID) {
			messages = append(messages, message)
		}
	}

	return messages
}

// FilterMessages returns messages matching the optional user and keyword filters.
func (s *ChatService) FilterMessages(userID, keyword string) []Message {
	messages := make([]Message, 0)
	keyword = strings.ToLower(strings.TrimSpace(keyword))

	for _, message := range s.messages {
		matchesUser := userID == "" ||
			message.SenderID == userID ||
			message.RecipientID == userID

		matchesKeyword := keyword == "" ||
			strings.Contains(strings.ToLower(message.Content), keyword)

		if matchesUser && matchesKeyword {
			messages = append(messages, message)
		}
	}

	return messages
}
