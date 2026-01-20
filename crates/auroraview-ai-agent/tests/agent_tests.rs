//! Integration tests for AI Agent

use auroraview_ai_agent::{AIAgent, AIConfig, ProviderType};

#[test]
fn test_config_creation() {
    let config = AIConfig::openai();
    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.provider_type(), ProviderType::OpenAI);

    let config = AIConfig::anthropic();
    assert_eq!(config.model, "claude-3-5-sonnet-20241022");
    assert_eq!(config.provider_type(), ProviderType::Anthropic);

    let config = AIConfig::gemini();
    assert_eq!(config.model, "gemini-2.0-flash-exp");
    assert_eq!(config.provider_type(), ProviderType::Gemini);

    let config = AIConfig::deepseek();
    assert_eq!(config.model, "deepseek-chat");
    assert_eq!(config.provider_type(), ProviderType::DeepSeek);
}

#[test]
fn test_config_builder() {
    let config = AIConfig::openai()
        .with_temperature(0.5)
        .with_max_tokens(2048)
        .with_system_prompt("You are a helpful assistant.")
        .with_streaming(false);

    assert_eq!(config.temperature, 0.5);
    assert_eq!(config.max_tokens, 2048);
    assert_eq!(
        config.system_prompt,
        Some("You are a helpful assistant.".to_string())
    );
    assert!(!config.stream);
}

#[test]
fn test_agent_creation() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);

    assert_eq!(agent.config().model, "gpt-4o");
}

#[tokio::test]
async fn test_action_names() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);

    let names = agent.action_names().await;
    assert!(names.contains(&"navigate".to_string()));
    assert!(names.contains(&"search".to_string()));
    assert!(names.contains(&"click".to_string()));
}

#[tokio::test]
async fn test_session_management() {
    let config = AIConfig::openai().with_system_prompt("Test prompt");
    let agent = AIAgent::new(config);

    // Create new session
    let session = agent.new_session().await;
    assert_eq!(session.title, "New Chat");
    assert_eq!(session.system_prompt, Some("Test prompt".to_string()));

    // Clear session
    agent.clear_session().await;
    let session = agent.current_session().await.unwrap();
    assert!(session.messages.is_empty());
}
