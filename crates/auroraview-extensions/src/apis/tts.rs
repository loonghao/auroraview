//! Chrome TTS (Text-to-Speech) API Implementation
//!
//! Provides text-to-speech functionality for extensions.
//!
//! ## Features
//! - Speak text
//! - Stop speech
//! - Get available voices
//! - Pause/resume speech

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Voice gender
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceGender {
    Male,
    Female,
}

/// TTS event type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtsEventType {
    Start,
    End,
    Word,
    Sentence,
    Marker,
    Interrupted,
    Cancelled,
    Error,
    Pause,
    Resume,
}

/// TTS voice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsVoice {
    /// Voice name
    pub voice_name: String,
    /// Language code (e.g., "en-US")
    pub lang: String,
    /// Voice gender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<VoiceGender>,
    /// Whether this is a remote voice
    pub remote: bool,
    /// Extension ID that provides this voice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_id: Option<String>,
    /// Supported event types
    pub event_types: Vec<TtsEventType>,
}

/// Speak options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpeakOptions {
    /// Voice name to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_name: Option<String>,
    /// Language code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Voice gender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<VoiceGender>,
    /// Speaking rate (0.1 to 10.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f32>,
    /// Pitch (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f32>,
    /// Volume (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,
    /// Whether to enqueue (true) or interrupt (false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enqueue: Option<bool>,
    /// Required event types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_event_types: Option<Vec<TtsEventType>>,
    /// Desired event types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desired_event_types: Option<Vec<TtsEventType>>,
}

/// TTS state
#[derive(Debug, Clone, Default)]
struct TtsState {
    /// Whether currently speaking
    speaking: bool,
    /// Whether paused
    paused: bool,
    /// Current utterance
    current_text: Option<String>,
}

/// TTS API handler
pub struct TtsApi {
    /// TTS state
    state: Arc<RwLock<TtsState>>,
    /// Available voices
    voices: Arc<RwLock<Vec<TtsVoice>>>,
}

impl Default for TtsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsApi {
    /// Create a new TtsApi instance
    pub fn new() -> Self {
        let api = Self {
            state: Arc::new(RwLock::new(TtsState::default())),
            voices: Arc::new(RwLock::new(Vec::new())),
        };
        api.init_default_voices();
        api
    }

    /// Initialize default voices
    fn init_default_voices(&self) {
        let mut voices = self.voices.write().unwrap();

        // Add some default system voices
        voices.push(TtsVoice {
            voice_name: "Microsoft David".to_string(),
            lang: "en-US".to_string(),
            gender: Some(VoiceGender::Male),
            remote: false,
            extension_id: None,
            event_types: vec![TtsEventType::Start, TtsEventType::End, TtsEventType::Error],
        });

        voices.push(TtsVoice {
            voice_name: "Microsoft Zira".to_string(),
            lang: "en-US".to_string(),
            gender: Some(VoiceGender::Female),
            remote: false,
            extension_id: None,
            event_types: vec![TtsEventType::Start, TtsEventType::End, TtsEventType::Error],
        });

        voices.push(TtsVoice {
            voice_name: "Google 中文".to_string(),
            lang: "zh-CN".to_string(),
            gender: None,
            remote: true,
            extension_id: None,
            event_types: vec![TtsEventType::Start, TtsEventType::End, TtsEventType::Error],
        });
    }

    /// Speak text
    pub fn speak(&self, utterance: &str, _options: SpeakOptions) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();

        // In a real implementation, this would use the system TTS API
        // For now, we just update the state
        state.speaking = true;
        state.paused = false;
        state.current_text = Some(utterance.to_string());

        Ok(json!(null))
    }

    /// Stop speaking
    pub fn stop(&self) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        state.speaking = false;
        state.paused = false;
        state.current_text = None;
        Ok(json!(null))
    }

    /// Pause speaking
    pub fn pause(&self) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        if state.speaking {
            state.paused = true;
        }
        Ok(json!(null))
    }

    /// Resume speaking
    pub fn resume(&self) -> ExtensionResult<Value> {
        let mut state = self.state.write().unwrap();
        if state.speaking && state.paused {
            state.paused = false;
        }
        Ok(json!(null))
    }

    /// Check if speaking
    pub fn is_speaking(&self) -> ExtensionResult<Value> {
        let state = self.state.read().unwrap();
        Ok(json!(state.speaking && !state.paused))
    }

    /// Get available voices
    pub fn get_voices(&self) -> ExtensionResult<Value> {
        let voices = self.voices.read().unwrap();
        Ok(serde_json::to_value(voices.clone())?)
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "speak" => {
                let utterance = params
                    .get("utterance")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing utterance".into()))?;
                let options: SpeakOptions = params
                    .get("options")
                    .cloned()
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();
                self.speak(utterance, options)
            }
            "stop" => self.stop(),
            "pause" => self.pause(),
            "resume" => self.resume(),
            "isSpeaking" => self.is_speaking(),
            "getVoices" => self.get_voices(),
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speak_and_stop() {
        let api = TtsApi::new();

        api.speak("Hello world", SpeakOptions::default()).unwrap();

        let speaking = api.is_speaking().unwrap();
        assert_eq!(speaking, json!(true));

        api.stop().unwrap();

        let speaking = api.is_speaking().unwrap();
        assert_eq!(speaking, json!(false));
    }

    #[test]
    fn test_get_voices() {
        let api = TtsApi::new();
        let result = api.get_voices().unwrap();
        let voices: Vec<TtsVoice> = serde_json::from_value(result).unwrap();
        assert!(!voices.is_empty());
    }

    #[test]
    fn test_pause_resume() {
        let api = TtsApi::new();

        api.speak("Hello world", SpeakOptions::default()).unwrap();
        api.pause().unwrap();

        let speaking = api.is_speaking().unwrap();
        assert_eq!(speaking, json!(false)); // Paused = not speaking

        api.resume().unwrap();

        let speaking = api.is_speaking().unwrap();
        assert_eq!(speaking, json!(true));
    }
}
