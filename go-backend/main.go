package main

import "fmt"

func main() {
	fmt.Println("Simple Chat Application - Go Backend")

	chatService := NewChatService()

	users := chatService.GetUsers()
	for _, user := range users {
		fmt.Printf("User: %s (%s)\n", user.DisplayName, user.UserID)
	}
}