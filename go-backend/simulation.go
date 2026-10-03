package main

import "sort"

// simulationMessage represents one message to send during the simulation.
type simulationMessage struct {
	senderID    string
	recipientID string
	content     string
}

// simulationResult carries the result of a goroutine back through a channel.
type simulationResult struct {
	message Message
	err     error
}

// RunSimulation sends messages concurrently using goroutines and channels.
func (s *ChatService) RunSimulation() ([]Message, error) {
	requests := []simulationMessage{
		{senderID: "alice", recipientID: "bob", content: "Hi Bob"},
		{senderID: "alice", recipientID: "bob", content: "Meeting at 3"},
		{senderID: "bob", recipientID: "charlie", content: "Hi Charlie"},
		{senderID: "bob", recipientID: "charlie", content: "How are you?"},
		{senderID: "charlie", recipientID: "alice", content: "Hi Alice"},
		{senderID: "charlie", recipientID: "alice", content: "See you later"},
	}

	results := make(chan simulationResult, len(requests))

	for _, request := range requests {
		request := request

		go func() {
			message, err := s.SendMessage(
				request.senderID,
				request.recipientID,
				request.content,
			)

			results <- simulationResult{
				message: message,
				err:     err,
			}
		}()
	}

	messages := make([]Message, 0, len(requests))

	for range requests {
		result := <-results

		if result.err != nil {
			return nil, result.err
		}

		messages = append(messages, result.message)
	}

	sort.Slice(messages, func(i, j int) bool {
		return messages[i].MessageID < messages[j].MessageID
	})

	return messages, nil
}
