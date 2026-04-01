//! Integration tests for ChatSession, SessionManager, Message, MessageContent, and ToolCall.

use auroraview_ai_agent::message::{ContentPart, Message, MessageContent, MessageRole, ToolCall};
use auroraview_ai_agent::session::{ChatSession, SessionManager};
use rstest::*;

// ─────────────────────────────────────────────────────────────
// MessageContent
// ─────────────────────────────────────────────────────────────

#[test]
fn message_content_text_roundtrip() {
    let content = MessageContent::text("hello");
    assert_eq!(content.as_text(), "hello");
}

#[test]
fn message_content_from_str() {
    let content: MessageContent = "world".into();
    assert_eq!(content.as_text(), "world");
}

#[test]
fn message_content_from_string() {
    let content: MessageContent = String::from("owned").into();
    assert_eq!(content.as_text(), "owned");
}

#[test]
fn message_content_parts_concatenates_text() {
    let content = MessageContent::parts(vec![
        ContentPart::text("Hello, "),
        ContentPart::text("World!"),
    ]);
    assert_eq!(content.as_text(), "Hello, World!");
}

#[test]
fn message_content_parts_skips_images() {
    let content = MessageContent::parts(vec![
        ContentPart::text("before"),
        ContentPart::image_url("https://example.com/img.png"),
        ContentPart::text("after"),
    ]);
    // Images are skipped, only text parts joined
    assert_eq!(content.as_text(), "beforeafter");
}

#[test]
fn message_content_empty_parts() {
    let content = MessageContent::parts(vec![]);
    assert_eq!(content.as_text(), "");
}

#[test]
fn content_part_image_base64() {
    let part = ContentPart::image_base64("abc123", "image/png");
    let json = serde_json::to_string(&part).unwrap();
    assert!(json.contains("image_url"));
    assert!(json.contains("data:image/png;base64,abc123"));
}

// ─────────────────────────────────────────────────────────────
// Message
// ─────────────────────────────────────────────────────────────

#[rstest]
#[case(MessageRole::System)]
#[case(MessageRole::User)]
#[case(MessageRole::Assistant)]
#[case(MessageRole::Tool)]
fn message_roles(#[case] role: MessageRole) {
    let msg = Message::new(role, "test content");
    assert_eq!(msg.role, role);
    assert!(!msg.id.is_empty());
}

#[test]
fn message_system() {
    let msg = Message::system("Be helpful");
    assert_eq!(msg.role, MessageRole::System);
    assert_eq!(msg.content.as_text(), "Be helpful");
    assert!(msg.tool_calls.is_none());
    assert!(msg.tool_call_id.is_none());
}

#[test]
fn message_user() {
    let msg = Message::user("Hello AI");
    assert_eq!(msg.role, MessageRole::User);
    assert_eq!(msg.content.as_text(), "Hello AI");
}

#[test]
fn message_assistant() {
    let msg = Message::assistant("Hello human");
    assert_eq!(msg.role, MessageRole::Assistant);
    assert_eq!(msg.content.as_text(), "Hello human");
}

#[test]
fn message_tool_result() {
    let msg = Message::tool_result("call-123", "42");
    assert_eq!(msg.role, MessageRole::Tool);
    assert_eq!(msg.content.as_text(), "42");
    assert_eq!(msg.tool_call_id, Some("call-123".to_string()));
}

#[test]
fn message_with_name() {
    let msg = Message::user("hi").with_name("alice");
    assert_eq!(msg.name, Some("alice".to_string()));
}

#[test]
fn message_with_tool_calls() {
    let calls = vec![
        ToolCall::new("navigate", r#"{"url":"https://rust-lang.org"}"#),
    ];
    let msg = Message::assistant("I'll navigate there").with_tool_calls(calls.clone());
    let tc = msg.tool_calls.as_ref().unwrap();
    assert_eq!(tc.len(), 1);
    assert_eq!(tc[0].name, "navigate");
}

#[test]
fn message_unique_ids() {
    let m1 = Message::user("a");
    let m2 = Message::user("b");
    assert_ne!(m1.id, m2.id);
}

#[test]
fn message_serialization_roundtrip() {
    let msg = Message::assistant("Hello!").with_name("bot");
    let json = serde_json::to_string(&msg).unwrap();
    let restored: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.content.as_text(), "Hello!");
    assert_eq!(restored.role, MessageRole::Assistant);
    assert_eq!(restored.name, Some("bot".to_string()));
}

// ─────────────────────────────────────────────────────────────
// ToolCall
// ─────────────────────────────────────────────────────────────

#[test]
fn tool_call_new() {
    let call = ToolCall::new("search", r#"{"query":"rust"}"#);
    assert_eq!(call.name, "search");
    assert!(!call.id.is_empty());
}

#[test]
fn tool_call_unique_ids() {
    let c1 = ToolCall::new("fn", "{}");
    let c2 = ToolCall::new("fn", "{}");
    assert_ne!(c1.id, c2.id);
}

#[test]
fn tool_call_parse_arguments() {
    #[derive(serde::Deserialize)]
    struct Args {
        url: String,
    }

    let call = ToolCall::new("navigate", r#"{"url":"https://example.com"}"#);
    let args: Args = call.parse_arguments().unwrap();
    assert_eq!(args.url, "https://example.com");
}

#[test]
fn tool_call_parse_invalid_json_returns_err() {
    #[derive(serde::Deserialize)]
    struct Args {
        _x: String,
    }
    let call = ToolCall::new("fn", "not-json");
    let result = call.parse_arguments::<Args>();
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// ChatSession
// ─────────────────────────────────────────────────────────────

#[test]
fn chat_session_new() {
    let s = ChatSession::new();
    assert_eq!(s.title, "New Chat");
    assert!(s.messages.is_empty());
    assert!(s.system_prompt.is_none());
    assert!(!s.id.is_empty());
}

#[test]
fn chat_session_default() {
    let s = ChatSession::default();
    assert_eq!(s.title, "New Chat");
}

#[test]
fn chat_session_with_system_prompt() {
    let s = ChatSession::with_system_prompt("You are a Rust expert.");
    assert_eq!(s.system_prompt, Some("You are a Rust expert.".to_string()));
    assert_eq!(s.title, "New Chat");
}

#[test]
fn session_add_user_message_sets_title() {
    let mut s = ChatSession::new();
    s.add_user_message("What is Rust?");
    assert_eq!(s.title, "What is Rust?");
    assert_eq!(s.message_count(), 1);
}

#[test]
fn session_title_truncated_at_50_chars() {
    let mut s = ChatSession::new();
    let long_msg = "A".repeat(60);
    s.add_user_message(&long_msg);
    assert!(s.title.ends_with("..."));
    assert!(s.title.len() <= 53); // 50 + "..."
}

#[test]
fn session_title_not_overwritten_by_second_user_message() {
    let mut s = ChatSession::new();
    s.add_user_message("First message");
    s.add_user_message("Second message");
    assert_eq!(s.title, "First message");
}

#[test]
fn session_add_assistant_message() {
    let mut s = ChatSession::new();
    s.add_assistant_message("Hello!");
    assert_eq!(s.message_count(), 1);
    assert_eq!(s.messages[0].role, MessageRole::Assistant);
}

#[test]
fn session_add_tool_result() {
    let mut s = ChatSession::new();
    s.add_tool_result("call-1", "result-data");
    let msg = &s.messages[0];
    assert_eq!(msg.role, MessageRole::Tool);
    assert_eq!(msg.tool_call_id, Some("call-1".to_string()));
}

#[test]
fn session_add_assistant_with_tools() {
    let mut s = ChatSession::new();
    let calls = vec![ToolCall::new("click", r##"{"selector":"#btn"}"##)];
    s.add_assistant_with_tools("Using tool", calls);
    let msg = s.last_assistant_message().unwrap();
    assert!(msg.tool_calls.is_some());
}

#[test]
fn session_last_message() {
    let mut s = ChatSession::new();
    assert!(s.last_message().is_none());
    s.add_user_message("Hi");
    assert_eq!(s.last_message().unwrap().content.as_text(), "Hi");
    s.add_assistant_message("Hello");
    assert_eq!(s.last_message().unwrap().content.as_text(), "Hello");
}

#[test]
fn session_last_assistant_message() {
    let mut s = ChatSession::new();
    s.add_user_message("Hi");
    assert!(s.last_assistant_message().is_none());
    s.add_assistant_message("Hello");
    s.add_user_message("Thanks");
    // Still returns assistant message even with later user message
    assert_eq!(
        s.last_assistant_message().unwrap().content.as_text(),
        "Hello"
    );
}

#[test]
fn session_get_messages_for_api_with_system_prompt() {
    let mut s = ChatSession::with_system_prompt("Be concise.");
    s.add_user_message("Hello");
    s.add_assistant_message("Hi!");

    let msgs = s.get_messages_for_api();
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[0].role, MessageRole::System);
    assert_eq!(msgs[1].role, MessageRole::User);
    assert_eq!(msgs[2].role, MessageRole::Assistant);
}

#[test]
fn session_get_messages_for_api_without_system_prompt() {
    let mut s = ChatSession::new();
    s.add_user_message("Hello");
    let msgs = s.get_messages_for_api();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].role, MessageRole::User);
}

#[test]
fn session_clear() {
    let mut s = ChatSession::with_system_prompt("Be helpful.");
    s.add_user_message("First");
    s.add_assistant_message("Response");
    assert_eq!(s.message_count(), 2);

    s.clear();
    assert_eq!(s.message_count(), 0);
    assert_eq!(s.title, "New Chat");
    // System prompt should still be preserved
    assert_eq!(s.system_prompt, Some("Be helpful.".to_string()));
}

#[test]
fn session_estimate_tokens() {
    let mut s = ChatSession::with_system_prompt("short");
    s.add_user_message("four char"); // 9 chars
    // "short" (5) + "four char" (9) = 14 chars / 4 ≈ 3 tokens
    let tokens = s.estimate_tokens();
    assert!(tokens >= 1);
}

#[test]
fn session_truncate_to_fit() {
    let mut s = ChatSession::new();
    // Add many messages
    for i in 0..20 {
        s.add_user_message(format!("Message number {} with some content", i));
        s.add_assistant_message(format!("Response to message {}", i));
    }
    let count_before = s.message_count();
    s.truncate_to_fit(5); // very small limit
    let count_after = s.message_count();
    assert!(count_after < count_before);
    assert!(count_after >= 1); // keeps at least 1
}

#[test]
fn session_serialization_roundtrip() {
    let mut s = ChatSession::with_system_prompt("test");
    s.add_user_message("hello");
    s.add_assistant_message("world");
    let id = s.id.clone();

    let json = serde_json::to_string(&s).unwrap();
    let restored: ChatSession = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.id, id);
    assert_eq!(restored.message_count(), 2);
    assert_eq!(restored.system_prompt, Some("test".to_string()));
}

// ─────────────────────────────────────────────────────────────
// SessionManager
// ─────────────────────────────────────────────────────────────

#[test]
fn session_manager_new_starts_empty() {
    let mgr = SessionManager::new();
    assert!(mgr.active_session().is_none());
    assert_eq!(mgr.all_sessions().len(), 0);
}

#[test]
fn session_manager_new_session_becomes_active() {
    let mut mgr = SessionManager::new();
    let session = mgr.new_session();
    let id = session.id.clone();

    assert!(mgr.active_session().is_some());
    assert_eq!(mgr.active_session().unwrap().id, id);
}

#[test]
fn session_manager_get_session() {
    let mut mgr = SessionManager::new();
    let session = mgr.new_session();
    let id = session.id.clone();

    assert!(mgr.get_session(&id).is_some());
    assert!(mgr.get_session("nonexistent").is_none());
}

#[test]
fn session_manager_set_active() {
    let mut mgr = SessionManager::new();
    let s1 = mgr.new_session();
    let id1 = s1.id.clone();
    let s2 = mgr.new_session();
    let id2 = s2.id.clone();

    // After second new_session, id2 should be active
    assert_eq!(mgr.active_session().unwrap().id, id2);

    // Switch back to first
    assert!(mgr.set_active(&id1));
    assert_eq!(mgr.active_session().unwrap().id, id1);
}

#[test]
fn session_manager_set_active_nonexistent_returns_false() {
    let mut mgr = SessionManager::new();
    assert!(!mgr.set_active("ghost-id"));
}

#[test]
fn session_manager_delete_session() {
    let mut mgr = SessionManager::new();
    let s1 = mgr.new_session();
    let id1 = s1.id.clone();
    mgr.new_session();

    assert!(mgr.delete_session(&id1));
    assert!(mgr.get_session(&id1).is_none());
}

#[test]
fn session_manager_delete_active_switches_to_another() {
    let mut mgr = SessionManager::new();
    let s1 = mgr.new_session();
    let id1 = s1.id.clone();
    mgr.new_session();
    mgr.set_active(&id1);

    mgr.delete_session(&id1);
    // Active should now point to the remaining session (or None)
    // It should not reference id1 anymore
    if let Some(active) = mgr.active_session() {
        assert_ne!(active.id, id1);
    }
}

#[test]
fn session_manager_delete_nonexistent_returns_false() {
    let mut mgr = SessionManager::new();
    assert!(!mgr.delete_session("ghost"));
}

#[test]
fn session_manager_sessions_by_recent_sorted() {
    let mut mgr = SessionManager::new();
    let s1 = mgr.new_session();
    let id1 = s1.id.clone();

    // Add messages to advance last_modified timestamp
    let s2 = mgr.new_session();
    let id2 = s2.id.clone();

    std::thread::sleep(std::time::Duration::from_millis(5));

    // Add a message to s1 to update its timestamp
    if let Some(s) = mgr.get_session_mut(&id1) {
        s.add_user_message("bump");
    }

    let recent = mgr.sessions_by_recent();
    assert_eq!(recent.len(), 2);
    // id1 was modified more recently
    assert_eq!(recent[0].id, id1);
    let _ = id2;
}

#[test]
fn session_manager_active_session_mut() {
    let mut mgr = SessionManager::new();
    mgr.new_session();

    {
        let active = mgr.active_session_mut().unwrap();
        active.add_user_message("mutated");
    }

    assert_eq!(
        mgr.active_session().unwrap().messages[0].content.as_text(),
        "mutated"
    );
}

#[test]
fn session_manager_multiple_sessions_all() {
    let mut mgr = SessionManager::new();
    mgr.new_session();
    mgr.new_session();
    mgr.new_session();
    assert_eq!(mgr.all_sessions().len(), 3);
}
