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

func TestGetConversation(t *testing.T) {
	handler := newTestHandler()

	messages := []string{
		`{"sender_id":"alice","recipient_id":"bob","content":"Hi Bob"}`,
		`{"sender_id":"bob","recipient_id":"alice","content":"Hi Alice"}`,
		`{"sender_id":"charlie","recipient_id":"alice","content":"Hi Alice from Charlie"}`,
	}

	for _, message := range messages {
		request := httptest.NewRequest(
			http.MethodPost,
			"/api/messages",
			strings.NewReader(message),
		)
		request.Header.Set("Content-Type", "application/json")

		response := httptest.NewRecorder()
		handler.ServeHTTP(response, request)

		if response.Code != http.StatusCreated {
			t.Fatalf("expected send status 201, got %d", response.Code)
		}
	}

	request := httptest.NewRequest(
		http.MethodGet,
		"/api/conversation?user_id=alice&other_user_id=bob",
		nil,
	)

	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)

	if response.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", response.Code)
	}

	var body struct {
		Messages []chat.Message `json:"messages"`
	}

	if err := json.NewDecoder(response.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if len(body.Messages) != 2 {
		t.Fatalf("expected 2 conversation messages, got %d", len(body.Messages))
	}

	for _, message := range body.Messages {
		inConversation :=
			(message.SenderID == "alice" && message.RecipientID == "bob") ||
				(message.SenderID == "bob" && message.RecipientID == "alice")

		if !inConversation {
			t.Errorf(
				"unexpected message in Alice-Bob conversation: %s -> %s",
				message.SenderID,
				message.RecipientID,
			)
		}
	}
}

func TestFilterMessages(t *testing.T) {
	handler := newTestHandler()

	messages := []string{
		`{"sender_id":"alice","recipient_id":"bob","content":"Meeting at 3"}`,
		`{"sender_id":"bob","recipient_id":"charlie","content":"Lunch at 1"}`,
		`{"sender_id":"charlie","recipient_id":"alice","content":"Meeting tomorrow"}`,
	}

	for _, message := range messages {
		request := httptest.NewRequest(
			http.MethodPost,
			"/api/messages",
			strings.NewReader(message),
		)
		request.Header.Set("Content-Type", "application/json")

		response := httptest.NewRecorder()
		handler.ServeHTTP(response, request)

		if response.Code != http.StatusCreated {
			t.Fatalf("expected send status 201, got %d", response.Code)
		}
	}

	request := httptest.NewRequest(
		http.MethodGet,
		"/api/messages?user_id=alice&keyword=meeting",
		nil,
	)

	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)

	if response.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", response.Code)
	}

	var body struct {
		Messages []chat.Message `json:"messages"`
	}

	if err := json.NewDecoder(response.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if len(body.Messages) != 2 {
		t.Fatalf("expected 2 matching messages, got %d", len(body.Messages))
	}

	for _, message := range body.Messages {
		involvesAlice :=
			message.SenderID == "alice" ||
				message.RecipientID == "alice"

		containsMeeting := strings.Contains(
			strings.ToLower(message.Content),
			"meeting",
		)

		if !involvesAlice || !containsMeeting {
			t.Errorf(
				"unexpected filtered message: %+v",
				message,
			)
		}
	}
}

func TestRunSimulation(t *testing.T) {
	handler := newTestHandler()

	request := httptest.NewRequest(
		http.MethodPost,
		"/api/simulation",
		nil,
	)

	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)

	if response.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", response.Code)
	}

	var body struct {
		AcceptedCount int            `json:"accepted_count"`
		Messages      []chat.Message `json:"messages"`
	}

	if err := json.NewDecoder(response.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if body.AcceptedCount != 6 {
		t.Errorf(
			"expected 6 accepted messages, got %d",
			body.AcceptedCount,
		)
	}

	if len(body.Messages) != 6 {
		t.Fatalf(
			"expected 6 simulation messages, got %d",
			len(body.Messages),
		)
	}

	seenIDs := make(map[uint64]bool)

	for _, message := range body.Messages {
		if seenIDs[message.MessageID] {
			t.Errorf(
				"duplicate message ID found: %d",
				message.MessageID,
			)
		}

		seenIDs[message.MessageID] = true
	}

	for expectedID := uint64(1); expectedID <= 6; expectedID++ {
		if !seenIDs[expectedID] {
			t.Errorf(
				"expected message ID %d was not found",
				expectedID,
			)
		}
	}
}
