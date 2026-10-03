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
}
