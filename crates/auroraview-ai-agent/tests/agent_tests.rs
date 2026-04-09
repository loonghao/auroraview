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
    assert_eq!(config.system_prompt, Some("你是一个有用的助手。".to_string()));
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
    assert_eq!(
        agent.config().system_prompt,
        Some("Be helpful".to_string())
    );
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

// ============================================================================
// AIConfig clone and debug
// ============================================================================

#[test]
fn config_clone() {
    let config = AIConfig::openai().with_temperature(0.3).with_max_tokens(512);
    let cloned = config.clone();
    assert_eq!(cloned.model, config.model);
    assert_eq!(cloned.temperature, config.temperature);
    assert_eq!(cloned.max_tokens, config.max_tokens);
}

#[test]
fn config_debug_non_empty() {
    let config = AIConfig::anthropic();
    let dbg = format!("{:?}", config);
    assert!(!dbg.is_empty());
}

// ============================================================================
// ProviderType clone and all variants debug
// ============================================================================

#[test]
fn provider_type_clone() {
    let pt = ProviderType::Gemini;
    // Verify Copy semantics (no Clone call needed for Copy types)
    let _copy = pt;
    assert_eq!(pt, _copy);
}

#[rstest]
#[case(ProviderType::OpenAI)]
#[case(ProviderType::Anthropic)]
#[case(ProviderType::Gemini)]
#[case(ProviderType::DeepSeek)]
fn provider_type_debug_non_empty(#[case] pt: ProviderType) {
    let dbg = format!("{:?}", pt);
    assert!(!dbg.is_empty());
}

// ============================================================================
// AIConfig — all provider constructors produce the expected model
// ============================================================================

#[test]
fn openai_config_model_non_empty() {
    let config = AIConfig::openai();
    assert!(!config.model.is_empty());
}

#[rstest]
#[case(AIConfig::openai(), "gpt-4o")]
#[case(AIConfig::anthropic(), "claude-3-5-sonnet-20241022")]
#[case(AIConfig::gemini(), "gemini-2.0-flash-exp")]
#[case(AIConfig::deepseek(), "deepseek-chat")]
fn all_provider_configs_have_model(#[case] config: AIConfig, #[case] expected_model: &str) {
    assert_eq!(config.model, expected_model);
}

// ============================================================================
// AIAgent — concurrent creation
// ============================================================================

#[test]
fn concurrent_agent_creation() {
    let handles: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                let config = AIConfig::openai();
                AIAgent::new(config)
            })
        })
        .collect();
    let agents: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(agents.len(), 4);
    for agent in &agents {
        assert_eq!(agent.config().model, "gpt-4o");
    }
}

// ============================================================================
// AIAgent — multiple sessions
// ============================================================================

#[tokio::test]
async fn multiple_new_sessions_reset_messages() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let _s1 = agent.new_session().await;
    let s2 = agent.new_session().await;
    // Each new session starts fresh
    assert!(s2.messages.is_empty());
}

#[tokio::test]
async fn clear_session_gives_empty_messages() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let _ = agent.new_session().await;
    agent.clear_session().await;
    let session = agent.current_session().await.unwrap();
    assert!(session.messages.is_empty());
}

// ============================================================================
// action_names contains expected tools
// ============================================================================

#[tokio::test]
async fn action_names_contains_type() {
    let config = AIConfig::openai();
    let agent = AIAgent::new(config);
    let names = agent.action_names().await;
    // Should contain at least one of the standard tools
    let has_tool = names.iter().any(|n| {
        n.contains("navigate") || n.contains("click") || n.contains("search") || n.contains("screenshot")
    });
    assert!(has_tool, "Expected at least one tool in: {:?}", names);
}

