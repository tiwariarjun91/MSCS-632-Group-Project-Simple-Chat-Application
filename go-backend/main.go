package main

import (
	"fmt"
	"log"
	"net/http"
)

const address = "127.0.0.1:8080"

func main() {
	chatService := NewChatService()
	api := NewAPI(chatService)

	fmt.Printf("Go chat backend: http://%s\n", address)
	fmt.Printf("Users API: http://%s/api/users\n", address)
	fmt.Println("Press Ctrl+C to stop. Message history lasts only for this session.")

	if err := http.ListenAndServe(address, api.Routes()); err != nil {
		log.Fatal(err)
	}
}
