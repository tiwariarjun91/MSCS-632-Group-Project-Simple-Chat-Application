/* Shared UI only: message storage, validation, filtering, and search stay in the backend. */
"use strict";

(() => {
  const element = (id) => document.getElementById(id);
  const activeUser = element("active-user");
  const recipient = element("recipient");
  const content = element("message-content");
  const connection = element("connection-status");
  const errorBox = element("error-message");
  const template = element("message-template");
  const state = {
    users: [], ready: false, sending: false, simulating: false,
    refreshing: false, conversationVersion: 0, historyVersion: 0,
    filters: { user_id: "", keyword: "" },
  };

  function showError(message) {
    errorBox.textContent = message;
    errorBox.hidden = false;
  }

  function clearError() {
    errorBox.textContent = "";
    errorBox.hidden = true;
  }

  function setConnection(connected) {
    connection.dataset.state = connected ? "connected" : "error";
    connection.textContent = connected ? "Connected · Local session" : "Connection unavailable · Try refreshing";
  }

  async function api(path, { method = "GET", body } = {}) {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 10000);
    try {
      const response = await fetch(path, {
        method,
        signal: controller.signal,
        cache: "no-store",
        headers: body === undefined ? {} : { "Content-Type": "application/json" },
        body: body === undefined ? undefined : JSON.stringify(body),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error?.message || `Request failed (${response.status}).`);
      }
      return data;
    } catch (error) {
      if (error.name === "AbortError" || error instanceof TypeError) {
        setConnection(false);
        const advice = method === "POST"
          ? "The outcome is unknown. Refresh history before retrying; messages may already be saved."
          : "Check that the local backend is running, then refresh.";
        throw new Error(`Could not reach the chat service. ${advice}`);
      }
      if (error instanceof SyntaxError) {
        throw new Error("The server returned an unexpected response. Check the selected backend.");
      }
      throw error;
    } finally {
      clearTimeout(timeout);
    }
  }

  function userName(id) {
    return state.users.find((user) => user.user_id === id)?.display_name || `User ${id}`;
  }

  function renderMessages(listId, emptyId, messages, emptyText, highlightSender) {
    const list = element(listId);
    const wasAtBottom = list.scrollHeight - list.scrollTop - list.clientHeight < 40;
    const oldScroll = list.scrollTop;
    const fragment = document.createDocumentFragment();
    for (const message of messages) {
      const card = template.content.firstElementChild.cloneNode(true);
      card.dataset.outgoing = String(message.sender_id === highlightSender);
      card.querySelector(".message-participants").textContent =
        `${userName(message.sender_id)} → ${userName(message.recipient_id)}`;
      card.querySelector(".message-content").textContent = message.content;
      card.querySelector(".message-id").textContent = `Message #${message.message_id}`;
      const date = new Date(message.timestamp_ms);
      const time = card.querySelector("time");
      if (Number.isFinite(date.getTime())) {
        time.dateTime = date.toISOString();
        time.textContent = date.toLocaleString();
      } else {
        time.textContent = "Timestamp unavailable";
      }
      fragment.append(card);
    }
    list.replaceChildren(fragment);
    list.scrollTop = wasAtBottom ? list.scrollHeight : oldScroll;
    const empty = element(emptyId);
    empty.textContent = emptyText;
    empty.hidden = messages.length > 0;
  }

  function populateRecipients() {
    const previous = recipient.value;
    recipient.replaceChildren();
    for (const user of state.users) {
      if (user.user_id !== activeUser.value) {
        recipient.add(new Option(user.display_name, user.user_id));
      }
    }
    if ([...recipient.options].some((option) => option.value === previous)) {
      recipient.value = previous;
    }
  }

  async function loadConversation() {
    const version = ++state.conversationVersion;
    const sender = activeUser.value;
    const other = recipient.value;
    if (!sender || !other) return;
    const query = new URLSearchParams({ user_id: sender, other_user_id: other });
    try {
      const data = await api(`/api/conversation?${query}`);
      if (version !== state.conversationVersion) return;
      renderMessages("conversation-messages", "conversation-empty", data.messages,
        "No messages yet. Start the conversation below.", sender);
      element("conversation-summary").textContent =
        `${userName(sender)} ↔ ${userName(other)} · ${data.messages.length} message(s)`;
    } catch (error) {
      if (version === state.conversationVersion) throw error;
    }
  }

  async function loadHistory() {
    const version = ++state.historyVersion;
    const query = new URLSearchParams();
    if (state.filters.user_id) query.set("user_id", state.filters.user_id);
    if (state.filters.keyword) query.set("keyword", state.filters.keyword);
    try {
      const data = await api(`/api/messages?${query}`);
      if (version !== state.historyVersion) return;
      renderMessages("history-messages", "history-empty", data.messages,
        "No messages match the current filters.");
      element("history-status").textContent = `${data.messages.length} message(s) found`;
    } catch (error) {
      if (version === state.historyVersion) throw error;
    }
  }

  async function refresh({ quiet = false } = {}) {
    if (!quiet) clearError();
    // Wait for both queries even if one fails, avoiding abandoned request work.
    const results = await Promise.allSettled([loadConversation(), loadHistory()]);
    const failure = results.find((result) => result.status === "rejected");
    if (failure) {
      setConnection(false);
      if (!quiet) showError(failure.reason.message);
    } else {
      setConnection(true);
    }
  }

  async function initialize() {
    clearError();
    connection.textContent = "Connecting to local chat…";
    try {
      const data = await api("/api/users");
      if (!Array.isArray(data.users) || data.users.length < 2) {
        throw new Error("The backend must provide at least two chat users.");
      }
      state.users = data.users;
      activeUser.replaceChildren();
      element("filter-user").replaceChildren(new Option("All users", ""));
      for (const user of state.users) {
        activeUser.add(new Option(user.display_name, user.user_id));
        element("filter-user").add(new Option(user.display_name, user.user_id));
      }
      populateRecipients();
      state.ready = true;
      for (const id of ["identity-controls", "composer-controls", "search-controls", "run-simulation"]) {
        element(id).disabled = false;
      }
      await refresh();
    } catch (error) {
      setConnection(false);
      showError(error.message);
    }
  }

  function changeConversation() {
    // Invalidate outstanding results and remove the previous participants' messages.
    ++state.conversationVersion;
    element("conversation-messages").replaceChildren();
    element("conversation-empty").hidden = false;
    element("conversation-empty").textContent = "Loading conversation…";
    element("conversation-summary").textContent = "Loading selected participants…";
    element("message-status").textContent = "";
    clearError();
    loadConversation().catch((error) => showError(error.message));
  }

  activeUser.addEventListener("change", () => {
    populateRecipients();
    changeConversation();
  });
  recipient.addEventListener("change", changeConversation);

  element("message-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    if (!state.ready || state.sending) return;
    state.sending = true;
    clearError();
    element("composer-controls").disabled = true;
    element("identity-controls").disabled = true;
    element("message-status").textContent = "Sending…";
    let accepted = false;
    try {
      const data = await api("/api/messages", {
        method: "POST",
        body: { sender_id: activeUser.value, recipient_id: recipient.value, content: content.value },
      });
      accepted = true;
      content.value = "";
      element("message-status").textContent = `Message #${data.message.message_id} saved.`;
      await refresh();
    } catch (error) {
      element("message-status").textContent = accepted ? "Message saved; refresh to view it." : "Message was not confirmed.";
      showError(error.message);
    } finally {
      state.sending = false;
      element("composer-controls").disabled = false;
      element("identity-controls").disabled = false;
      content.focus();
    }
  });

  element("search-form").addEventListener("submit", (event) => {
    event.preventDefault();
    state.filters = { user_id: element("filter-user").value, keyword: element("search-keyword").value };
    clearError();
    element("history-status").textContent = "Searching…";
    loadHistory().catch((error) => {
      element("history-status").textContent = "Search unavailable. Previous results may be outdated.";
      showError(error.message);
    });
  });
  element("clear-search").addEventListener("click", () => {
    element("search-form").reset();
    state.filters = { user_id: "", keyword: "" };
    clearError();
    loadHistory().catch((error) => showError(error.message));
  });

  element("run-simulation").addEventListener("click", async () => {
    if (!state.ready || state.simulating) return;
    state.simulating = true;
    clearError();
    element("run-simulation").disabled = true;
    element("simulation-status").textContent = "Three users are sending messages…";
    try {
      const data = await api("/api/simulation", { method: "POST" });
      element("simulation-status").textContent = `${data.accepted_count} messages saved by the simulation.`;
      await refresh();
    } catch (error) {
      element("simulation-status").textContent = "Simulation was not confirmed. Check history before retrying.";
      showError(error.message);
    } finally {
      state.simulating = false;
      element("run-simulation").disabled = false;
    }
  });

  // Poll read-only views to receive messages from other tabs. Never retry writes.
  setInterval(async () => {
    if (!state.ready || document.hidden || state.refreshing || state.sending || state.simulating) return;
    state.refreshing = true;
    try { await refresh({ quiet: true }); }
    finally { state.refreshing = false; }
  }, 3000);

  void initialize();
})();
