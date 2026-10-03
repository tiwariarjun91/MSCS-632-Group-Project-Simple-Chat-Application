package api

import (
	"encoding/json"
	"errors"
	"net/http"

	"github.com/tiwariarjun91/MSCS-632-Group-Project-Simple-Chat-Application/go-backend/internal/chat"
)

// API connects HTTP requests to the chat service.
type API struct {
	chatService *chat.ChatService
}

// NewAPI creates an API backed by the provided chat service.
func NewAPI(chatService *chat.ChatService) *API {
	return &API{
		chatService: chatService,
	}
}

// Routes registers the HTTP endpoints.
func (a *API) Routes() http.Handler {
	mux := http.NewServeMux()

	mux.HandleFunc("/api/users", a.handleUsers)
	mux.HandleFunc("/api/messages", a.handleMessages)
	mux.HandleFunc("/api/conversation", a.handleConversation)
	mux.HandleFunc("/api/simulation", a.handleSimulation)

	// Serve the shared frontend.
	mux.Handle("/", http.FileServer(http.Dir("../frontend")))

	return mux
}

func (a *API) handleUsers(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(
			w,
			http.StatusMethodNotAllowed,
			"method_not_allowed",
			"This HTTP method is not supported for this route.",
		)
		return
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"users": a.chatService.GetUsers(),
	})
}

func (a *API) handleMessages(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodGet:
		userID := r.URL.Query().Get("user_id")
		keyword := r.URL.Query().Get("keyword")

		writeJSON(w, http.StatusOK, map[string]any{
			"messages": a.chatService.FilterMessages(userID, keyword),
		})

	case http.MethodPost:
		a.handleSendMessage(w, r)

	default:
		writeError(
			w,
			http.StatusMethodNotAllowed,
			"method_not_allowed",
			"This HTTP method is not supported for this route.",
		)
	}
}

func (a *API) handleSendMessage(w http.ResponseWriter, r *http.Request) {
	if r.Header.Get("Content-Type") != "application/json" {
		writeError(
			w,
			http.StatusUnsupportedMediaType,
			"unsupported_media_type",
			"Send the message with Content-Type: application/json.",
		)
		return
	}

	var request struct {
		SenderID    string `json:"sender_id"`
		RecipientID string `json:"recipient_id"`
		Content     string `json:"content"`
	}

	if err := json.NewDecoder(r.Body).Decode(&request); err != nil {
		writeError(
			w,
			http.StatusBadRequest,
			"invalid_request",
			"Provide valid JSON with sender_id, recipient_id, and content.",
		)
		return
	}

	message, err := a.chatService.SendMessage(
		request.SenderID,
		request.RecipientID,
		request.Content,
	)

	if err != nil {
		switch {
		case errors.Is(err, chat.ErrUnknownUser):
			writeError(
				w,
				http.StatusBadRequest,
				"unknown_user",
				err.Error(),
			)

		case errors.Is(err, chat.ErrSameUser):
			writeError(
				w,
				http.StatusBadRequest,
				"same_user",
				err.Error(),
			)

		case errors.Is(err, chat.ErrEmptyContent):
			writeError(
				w,
				http.StatusBadRequest,
				"empty_content",
				err.Error(),
			)

		default:
			writeError(
				w,
				http.StatusInternalServerError,
				"internal_error",
				"The backend could not complete the request.",
			)
		}

		return
	}

	writeJSON(w, http.StatusCreated, map[string]any{
		"message": message,
	})
}

func (a *API) handleConversation(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(
			w,
			http.StatusMethodNotAllowed,
			"method_not_allowed",
			"This HTTP method is not supported for this route.",
		)
		return
	}

	userID := r.URL.Query().Get("user_id")
	otherUserID := r.URL.Query().Get("other_user_id")

	if userID == "" || otherUserID == "" {
		writeError(
			w,
			http.StatusBadRequest,
			"invalid_request",
			"Provide both user_id and other_user_id.",
		)
		return
	}

	messages := a.chatService.GetConversation(userID, otherUserID)

	writeJSON(w, http.StatusOK, map[string]any{
		"messages": messages,
	})
}

func (a *API) handleSimulation(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(
			w,
			http.StatusMethodNotAllowed,
			"method_not_allowed",
			"This HTTP method is not supported for this route.",
		)
		return
	}

	messages, err := a.chatService.RunSimulation()
	if err != nil {
		writeError(
			w,
			http.StatusInternalServerError,
			"internal_error",
			"The backend could not complete the request.",
		)
		return
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"accepted_count": len(messages),
		"messages":       messages,
	})
}

func writeJSON(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)

	_ = json.NewEncoder(w).Encode(value)
}

func writeError(w http.ResponseWriter, status int, code, message string) {
	writeJSON(w, status, map[string]any{
		"error": map[string]string{
			"code":    code,
			"message": message,
		},
	})
}
