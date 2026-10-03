package main

// ChatService manages users and message history for the application.
type ChatService struct {
	users    map[string]User
	messages []Message
}