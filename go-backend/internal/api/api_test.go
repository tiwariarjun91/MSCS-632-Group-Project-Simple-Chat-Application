package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/tiwariarjun91/MSCS-632-Group-Project-Simple-Chat-Application/go-backend/internal/chat"
)

func newTestHandler() http.Handler {
	chatService := chat.NewChatService()
	api := NewAPI(chatService)

	return api.Routes()
}

func TestSendMessage(t *testing.T) {
	handler := newTestHandler()

	requestBody := `{
		"sender_id": "alice",
		"recipient_id": "bob",
		"content": "Hello Bob"
	}`

	request := httptest.NewRequest(
		http.MethodPost,
		"/api/messages",
		strings.NewReader(requestBody),
	)
	request.Header.Set("Content-Type", "application/json")

	response := httptest.NewRecorder()

	handler.ServeHTTP(response, request)

	if response.Code != http.StatusCreated {
		t.Fatalf("expected status 201, got %d", response.Code)
	}

	var body struct {
		Message chat.Message `json:"message"`
	}

	if err := json.NewDecoder(response.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if body.Message.SenderID != "alice" {
		t.Errorf("expected sender alice, got %s", body.Message.SenderID)
	}

	if body.Message.RecipientID != "bob" {
		t.Errorf("expected recipient bob, got %s", body.Message.RecipientID)
	}

	if body.Message.Content != "Hello Bob" {
		t.Errorf("expected content Hello Bob, got %s", body.Message.Content)
	}

	if body.Message.MessageID != 1 {
		t.Errorf("expected message ID 1, got %d", body.Message.MessageID)
	}
}

func TestGetMessages(t *testing.T) {
	handler := newTestHandler()

	// First, send a message.
	requestBody := `{
		"sender_id": "alice",
		"recipient_id": "bob",
		"content": "Meeting at 3"
	}`

	sendRequest := httptest.NewRequest(
		http.MethodPost,
		"/api/messages",
		strings.NewReader(requestBody),
	)
	sendRequest.Header.Set("Content-Type", "application/json")

	sendResponse := httptest.NewRecorder()
	handler.ServeHTTP(sendResponse, sendRequest)

	if sendResponse.Code != http.StatusCreated {
		t.Fatalf("expected send status 201, got %d", sendResponse.Code)
	}

	// Now retrieve the message history.
	getRequest := httptest.NewRequest(
		http.MethodGet,
		"/api/messages",
		nil,
	)

	getResponse := httptest.NewRecorder()
	handler.ServeHTTP(getResponse, getRequest)

	if getResponse.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", getResponse.Code)
	}

	var body struct {
		Messages []chat.Message `json:"messages"`
	}

	if err := json.NewDecoder(getResponse.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if len(body.Messages) != 1 {
		t.Fatalf("expected 1 message, got %d", len(body.Messages))
	}

	if body.Messages[0].Content != "Meeting at 3" {
		t.Errorf(
			"expected content %q, got %q",
			"Meeting at 3",
			body.Messages[0].Content,
		)
	}
}
