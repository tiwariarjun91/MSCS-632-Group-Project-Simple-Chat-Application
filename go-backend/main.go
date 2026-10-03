package main

import (
	"fmt"
)

func main() {
	fmt.Println("Simple Chat Application - Go Backend")

	chatService := NewChatService()

	users := chatService.GetUsers()

	// Go does not guarantee map iteration order
	for _, user := range users {
		fmt.Printf("User: %s (%s)\n", user.DisplayName, user.UserID)
	}

	//To be deleted
	messages, err := chatService.RunSimulation()
	if err != nil {
		fmt.Println("Simulation failed:", err)
		return
	}

	fmt.Println("Simulation complete:")

	for _, message := range messages {
		fmt.Printf(
			"Message %d: %s -> %s: %s\n",
			message.MessageID,
			message.SenderID,
			message.RecipientID,
			message.Content,
		)
	}
}
