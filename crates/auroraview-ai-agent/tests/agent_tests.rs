//! Integration tests for AI Agent

use auroraview_ai_agent::{AIAgent, AIConfig, ProviderType};
use rstest::rstest;

// ============================================================================
// AIConfig creation
// ============================================================================

#[test]
fn config_openai() {
    let config = AIConfig::openai();
    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.provider_type(), ProviderType::OpenAI);
}

#[test]
fn config_anthropic() {
    let config = AIConfig::anthropic();
    assert_eq!(config.model, "claude-3-5-sonnet-20241022");
    assert_eq!(config.provider_type(), ProviderType::Anthropic);
}

#[test]
fn config_gemini() {
    let config = AIConfig::gemini();
    assert_eq!(config.model, "gemini-2.0-flash-exp");
    assert_eq!(config.provider_type(), ProviderType::Gemini);
}

#[test]
fn config_deepseek() {
    let config = AIConfig::deepseek();
    assert_eq!(config.model, "deepseek-chat");
    assert_eq!(config.provider_type(), ProviderType::DeepSeek);
}

#[test]
fn config_ollama() {
    let config = AIConfig::ollama("llama3");
    assert_eq!(config.model, "llama3");
}

#[test]
fn config_for_model() {
    let config = AIConfig::for_model("custom-model");
    assert_eq!(config.model, "custom-model");
}

#[test]
fn config_default() {
    let config = AIConfig::default();
    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.temperature, 0.7);
    assert_eq!(config.max_tokens, 4096);
    assert!(config.system_prompt.is_none());
    assert!(config.stream);
}

// ============================================================================
// AIConfig builder methods
// ============================================================================

#[test]
fn config_builder_chain() {
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
fn config_temperature_min() {
    let config = AIConfig::openai().with_temperature(0.0);
    assert_eq!(config.temperature, 0.0);
}

#[test]
fn config_temperature_max() {
    let config = AIConfig::openai().with_temperature(2.0);
    assert_eq!(config.temperature, 2.0);
}

#[test]
fn config_temperature_clamped_below_zero() {
    let config = AIConfig::openai().with_temperature(-1.0);
    assert_eq!(config.temperature, 0.0);
}

#[test]
fn config_temperature_clamped_above_two() {
    let config = AIConfig::openai().with_temperature(5.0);
    assert_eq!(config.temperature, 2.0);
}

#[test]
fn config_max_tokens_zero() {
    let config = AIConfig::openai().with_max_tokens(0);
    assert_eq!(config.max_tokens, 0);
}

#[test]
fn config_max_tokens_large() {
    let config = AIConfig::openai().with_max_tokens(100000);
    assert_eq!(config.max_tokens, 100000);
}

#[test]
fn config_system_prompt_empty() {
    let config = AIConfig::openai().with_system_prompt("");
    assert_eq!(config.system_prompt, Some("".to_string()));
}

#[test]
fn config_system_prompt_unicode() {
    let config = AIConfig::openai().with_system_prompt("你是一个有用的助手。");
    assert_eq!(
        config.system_prompt,
        Some("你是一个有用的助手。".to_string())
    );
}

#[test]
fn config_streaming_default_true() {
    let config = AIConfig::openai();
    assert!(config.stream);
}

#[test]
fn config_streaming_disabled() {
    let config = AIConfig::openai().with_streaming(false);
    assert!(!config.stream);
}

// ============================================================================
// ProviderType
// ============================================================================

#[rstest]
#[case("gpt-4o", ProviderType::OpenAI)]
#[case("gpt-3.5-turbo", ProviderType::OpenAI)]
#[case("claude-3-5-sonnet-20241022", ProviderType::Anthropic)]
#[case("gemini-2.0-flash-exp", ProviderType::Gemini)]
#[case("deepseek-chat", ProviderType::DeepSeek)]
fn config_provider_type(#[case] model: &str, #[case] expected: ProviderType) {
    let config = AIConfig::for_model(model);
    assert_eq!(config.provider_type(), expected);
}

#[test]
fn provider_type_eq() {
    assert_eq!(ProviderType::OpenAI, ProviderType::OpenAI);
    assert_ne!(ProviderType::OpenAI, ProviderType::Anthropic);
}

#[test]
fn provider_type_debug() {
    let variants = [
        ProviderType::OpenAI,
        ProviderType::Anthropic,
        ProviderType::Gemini,
        ProviderType::DeepSeek,
    ];
    for v in &variants {
        let debug = format!("{:?}", v);
        assert!(!debug.is_empty());
    }
}

// ============================================================================
// AIAgent creation
// ============================================================================

#[test]
fn agent_creation() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    assert_eq!(agent.config().model, "gpt-4o");
}

#[test]
fn agent_creation_anthropic() {
    let config = AIConfig::anthropic();
    let agent = AIAgent::new(config);
    assert_eq!(agent.config().model, "claude-3-5-sonnet-20241022");
}

#[test]
fn agent_creation_with_system_prompt() {
    let config = AIConfig::openai().with_system_prompt("Be helpful");
    let agent = AIAgent::new(config);
    assert_eq!(agent.config().system_prompt, Some("Be helpful".to_string()));
}

// ============================================================================
// Async tests
// ============================================================================

#[tokio::test]
async fn action_names_contains_navigate() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let names = agent.action_names().await;
    assert!(names.contains(&"navigate".to_string()));
}

#[tokio::test]
async fn action_names_contains_search() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let names = agent.action_names().await;
    assert!(names.contains(&"search".to_string()));
}

#[tokio::test]
async fn action_names_contains_click() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let names = agent.action_names().await;
    assert!(names.contains(&"click".to_string()));
}

#[tokio::test]
async fn action_names_not_empty() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let names = agent.action_names().await;
    assert!(!names.is_empty());
}

#[tokio::test]
async fn session_management() {
    let config = AIConfig::openai().with_system_prompt("Test prompt");
    let agent = AIAgent::new(config);

    let session = agent.new_session().await;
    assert_eq!(session.title, "New Chat");
    assert_eq!(session.system_prompt, Some("Test prompt".to_string()));

    agent.clear_session().await;
    let session = agent.current_session().await.unwrap();
    assert!(session.messages.is_empty());
}

#[tokio::test]
async fn new_session_has_empty_messages() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let session = agent.new_session().await;
    assert!(session.messages.is_empty());
}

#[tokio::test]
async fn current_session_after_new() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let _ = agent.new_session().await;
    let current = agent.current_session().await;
    assert!(current.is_some());
}
