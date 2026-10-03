package main

import (
	"fmt"
	"log"
	"net/http"

	"github.com/tiwariarjun91/MSCS-632-Group-Project-Simple-Chat-Application/go-backend/internal/api"
	"github.com/tiwariarjun91/MSCS-632-Group-Project-Simple-Chat-Application/go-backend/internal/chat"
)

const address = "127.0.0.1:8080"

func main() {
	chatService := chat.NewChatService()
	apiHandler := api.NewAPI(chatService)

	fmt.Printf("Go chat backend: http://%s\n", address)
	fmt.Printf("Users API: http://%s/api/users\n", address)
	fmt.Println("Press Ctrl+C to stop. Message history lasts only for this session.")

	if err := http.ListenAndServe(address, apiHandler.Routes()); err != nil {
		log.Fatal(err)
	}
}
