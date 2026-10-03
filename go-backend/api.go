package main

import (
	"encoding/json"
	"net/http"
)

// API connects HTTP requests to the chat service.
type API struct {
	chatService *ChatService
}

// NewAPI creates an API backed by the provided chat service.
func NewAPI(chatService *ChatService) *API {
	return &API{
		chatService: chatService,
	}
}

// Routes registers all HTTP endpoints for the chat application.
func (a *API) Routes() http.Handler {
	mux := http.NewServeMux()

	mux.HandleFunc("/api/users", a.handleUsers)
	mux.HandleFunc("/api/messages", a.handleMessages)
	mux.HandleFunc("/api/conversation", a.handleConversation)
	mux.HandleFunc("/api/simulation", a.handleSimulation)

	return mux
}

// handleUsers returns all registered users.
func (a *API) handleUsers(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method_not_allowed", "Method not allowed.")
		return
	}

	writeJSON(w, http.StatusOK, a.chatService.GetUsers())
}

// handleMessages supports retrieving and sending messages.
func (a *API) handleMessages(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodGet:
		a.handleGetMessages(w, r)

	case http.MethodPost:
		a.handleSendMessage(w, r)

	default:
		writeError(w, http.StatusMethodNotAllowed, "method_not_allowed", "Method not allowed.")
	}
}

// handleGetMessages returns message history with optional filters.
func (a *API) handleGetMessages(w http.ResponseWriter, r *http.Request) {
	userID := r.URL.Query().Get("user_id")
	keyword := r.URL.Query().Get("keyword")

	messages := a.chatService.FilterMessages(userID, keyword)

	writeJSON(w, http.StatusOK, messages)
}

// handleSendMessage creates a new chat message.
func (a *API) handleSendMessage(w http.ResponseWriter, r *http.Request) {
	var request struct {
		SenderID    string `json:"sender_id"`
		RecipientID string `json:"recipient_id"`
		Content     string `json:"content"`
	}

	if err := json.NewDecoder(r.Body).Decode(&request); err != nil {
		writeError(w, http.StatusBadRequest, "invalid_json", "Invalid JSON request.")
		return
	}

	message, err := a.chatService.SendMessage(
		request.SenderID,
		request.RecipientID,
		request.Content,
	)

	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid_message", err.Error())
		return
	}

	writeJSON(w, http.StatusCreated, map[string]Message{
		"message": message,
	})
}

// handleConversation returns messages exchanged between two users.
func (a *API) handleConversation(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method_not_allowed", "Method not allowed.")
		return
	}

	userID := r.URL.Query().Get("user_id")
	otherUserID := r.URL.Query().Get("other_user_id")

	if userID == "" || otherUserID == "" {
		writeError(
			w,
			http.StatusBadRequest,
			"missing_user",
			"Both user_id and other_user_id are required.",
		)
		return
	}

	messages := a.chatService.GetConversation(userID, otherUserID)

	writeJSON(w, http.StatusOK, messages)
}

// handleSimulation runs the concurrent chat simulation.
func (a *API) handleSimulation(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method_not_allowed", "Method not allowed.")
		return
	}

	messages, err := a.chatService.RunSimulation()
	if err != nil {
		writeError(w, http.StatusInternalServerError, "simulation_failed", err.Error())
		return
	}

	writeJSON(w, http.StatusOK, messages)
}

// writeJSON writes a JSON response with the provided HTTP status.
func writeJSON(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)

	_ = json.NewEncoder(w).Encode(value)
}

// writeError writes a consistent JSON error response.
func writeError(w http.ResponseWriter, status int, code, message string) {
	writeJSON(w, status, map[string]any{
		"error": map[string]string{
			"code":    code,
			"message": message,
		},
	})
}