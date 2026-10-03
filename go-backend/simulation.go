package main

import "sort"

// simulationUser represents one user participating in the simulation.
type simulationUser struct {
	senderID    string
	recipientID string
	messages    []string
}

// simulationResult carries a message or error from a goroutine.
type simulationResult struct {
	message Message
	err     error
}

// RunSimulation runs three users concurrently.
// Each user sends two messages sequentially.
func (s *ChatService) RunSimulation() ([]Message, error) {
	users := []simulationUser{
		{
			senderID:    "alice",
			recipientID: "bob",
			messages:    []string{"Simulation Alice 1", "Simulation Alice 2"},
		},
		{
			senderID:    "bob",
			recipientID: "charlie",
			messages:    []string{"Simulation Bob 1", "Simulation Bob 2"},
		},
		{
			senderID:    "charlie",
			recipientID: "alice",
			messages:    []string{"Simulation Charlie 1", "Simulation Charlie 2"},
		},
	}

	results := make(chan simulationResult, 6)

	for _, user := range users {
		user := user

		go func() {
			for _, content := range user.messages {
				message, err := s.SendMessage(
					user.senderID,
					user.recipientID,
					content,
				)

				results <- simulationResult{
					message: message,
					err:     err,
				}
			}
		}()
	}

	messages := make([]Message, 0, 6)

	for i := 0; i < 6; i++ {
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
