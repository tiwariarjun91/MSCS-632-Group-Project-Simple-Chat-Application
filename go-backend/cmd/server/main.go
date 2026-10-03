package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/tiwariarjun91/MSCS-632-Group-Project-Simple-Chat-Application/go-backend/internal/api"
	"github.com/tiwariarjun91/MSCS-632-Group-Project-Simple-Chat-Application/go-backend/internal/chat"
)

const address = "127.0.0.1:8080"

func main() {
	chatService := chat.NewChatService()
	apiHandler := api.NewAPI(chatService)

	server := &http.Server{
		Addr:    address,
		Handler: apiHandler.Routes(),
	}

	fmt.Printf("Go chat backend: http://%s\n", address)
	fmt.Printf("Users API: http://%s/api/users\n", address)
	fmt.Println("Press Ctrl+C to stop. Message history lasts only for this session.")

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)

	go func() {
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("server error: %v", err)
		}
	}()

	<-stop

	fmt.Println("\nShutting down server...")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		log.Printf("server shutdown error: %v", err)
		return
	}

	fmt.Println("Server stopped.")
}