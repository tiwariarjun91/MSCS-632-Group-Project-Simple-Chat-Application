package chat

import "testing"

func TestSendMessage(t *testing.T) {
	service := NewChatService()

	message, err := service.SendMessage("alice", "bob", "Hello Bob")

	if err != nil {
		t.Fatalf("expected message to be sent, got error: %v", err)
	}

	if message.MessageID != 1 {
		t.Errorf("expected message ID 1, got %d", message.MessageID)
	}

	if message.SenderID != "alice" {
		t.Errorf("expected sender alice, got %s", message.SenderID)
	}

	if message.RecipientID != "bob" {
		t.Errorf("expected recipient bob, got %s", message.RecipientID)
	}

	if message.Content != "Hello Bob" {
		t.Errorf("expected content %q, got %q", "Hello Bob", message.Content)
	}

	messages := service.GetMessages()

	if len(messages) != 1 {
		t.Fatalf("expected 1 stored message, got %d", len(messages))
	}

	if messages[0].MessageID != message.MessageID {
		t.Errorf(
			"expected stored message ID %d, got %d",
			message.MessageID,
			messages[0].MessageID,
		)
	}
}

func TestMessageIDsIncrease(t *testing.T) {
	service := NewChatService()

	first, err := service.SendMessage("alice", "bob", "First message")
	if err != nil {
		t.Fatalf("failed to send first message: %v", err)
	}

	second, err := service.SendMessage("bob", "alice", "Second message")
	if err != nil {
		t.Fatalf("failed to send second message: %v", err)
	}

	if first.MessageID != 1 {
		t.Errorf("expected first message ID 1, got %d", first.MessageID)
	}

	if second.MessageID != 2 {
		t.Errorf("expected second message ID 2, got %d", second.MessageID)
	}
}

func TestInvalidMessage(t *testing.T) {
	service := NewChatService()

	_, err := service.SendMessage("alice", "alice", "Hello")

	if err == nil {
		t.Fatal("expected error when sending a message to the same user")
	}

	if len(service.GetMessages()) != 0 {
		t.Errorf("invalid message should not be stored")
	}
}
