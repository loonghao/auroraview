// Copyright 2025 AuroraView contributors.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Simplified OAuth 2.0 server implementation for local MCP server.
//!
//! This module provides a MINIMAL OAuth 2.0 implementation suitable for
//! local DCC integration scenarios. It supports:
//!
//! - OAuth 2.0 Authorization Code Grant with PKCE (S256)
//! - In-memory token storage (suitable for single-user local scenarios)
//! - Configurable enforcement (can be disabled for local development)
//!
//! **Security Note**: This is a SIMPLIFIED implementation. For production
//! multi-user scenarios, use a full OAuth 2.0 server with database backing.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use dashmap::DashMap;

/// OAuth 2.0 client configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    /// Client ID (UUID v4).
    pub client_id: String,
    /// Client secret (hashed for storage).
    pub client_secret_hash: String,
    /// Client name (for display purposes).
    pub name: String,
    /// Allowed redirect URIs.
    pub redirect_uris: Vec<String>,
    /// Granted scopes.
    pub scope: String,
}

/// OAuth 2.0 authorization code (short-lived, single-use).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    /// The authorization code.
    pub code: String,
    /// Associated client ID.
    pub client_id: String,
    /// Redirect URI (must match token request).
    pub redirect_uri: String,
    /// PKCE code challenge (S256).
    pub code_challenge: String,
    /// Granted scopes.
    pub scope: String,
    /// Expiry time (Unix timestamp).
    pub expires_at: i64,
}

/// OAuth 2.0 access token (JWT).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    /// JWT issuer.
    pub iss: String,
    /// JWT subject (client ID).
    pub sub: String,
    /// JWT audience (resource server).
    pub aud: String,
    /// Expiry time (Unix timestamp).
    pub exp: i64,
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Granted scopes.
    pub scope: String,
}

/// OAuth 2.0 token response.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// Access token (JWT).
    pub access_token: String,
    /// Token type (always "Bearer").
    pub token_type: &'static str,
    /// Expiry in seconds.
    pub expires_in: i64,
    /// Refresh token (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Granted scopes.
    pub scope: String,
}

/// In-memory OAuth store.
#[derive(Clone)]
pub struct OAuthStore {
    clients: Arc<DashMap<String, OAuthClient>>,
    codes: Arc<DashMap<String, AuthorizationCode>>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl OAuthStore {
    /// Create a new OAuth store with auto-generated JWT secret.
    pub fn new() -> Self {
        let jwt_secret = std::env::var("AURORAVIEW_JWT_SECRET")
            .unwrap_or_else(|_| Uuid::new_v4().to_string());

        let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        Self {
            clients: Arc::new(DashMap::new()),
            codes: Arc::new(DashMap::new()),
            encoding_key,
            decoding_key,
        }
    }
}

impl Default for OAuthStore {
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthStore {
    /// Register a new OAuth client (dynamic registration).
    pub fn register_client(
        &self,
        name: String,
        redirect_uris: Vec<String>,
        scope: String,
    ) -> (OAuthClient, String) {
        let client_id = Uuid::new_v4().to_string();
        let client_secret = Uuid::new_v4().to_string();

        let client = OAuthClient {
            client_id: client_id.clone(),
            client_secret_hash: bcrypt::hash(&client_secret, bcrypt::DEFAULT_COST)
                .unwrap_or_default(),
            name,
            redirect_uris,
            scope,
        };

        self.clients.insert(client_id.clone(), client.clone());

        (client, client_secret)
    }

    /// Validate client credentials.
    pub fn validate_client(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Option<OAuthClient> {
        let client = self.clients.get(client_id)?;

        if bcrypt::verify(client_secret, &client.client_secret_hash).unwrap_or(false) {
            Some(client.clone())
        } else {
            None
        }
    }

    /// Issue a new authorization code.
    pub fn issue_code(
        &self,
        client_id: String,
        redirect_uri: String,
        code_challenge: String,
        scope: String,
    ) -> String {
        let code = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now().timestamp() + 600; // 10 minutes

        let auth_code = AuthorizationCode {
            code: code.clone(),
            client_id,
            redirect_uri,
            code_challenge,
            scope,
            expires_at,
        };

        self.codes.insert(code.clone(), auth_code);

        code
    }

    /// Exchange authorization code for access token.
    pub fn exchange_code(
        &self,
        code: &str,
        client_id: &str,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> Option<TokenResponse> {
        // Get the code (DashMap::get returns Option<Ref<K, V>>)
        let auth_code = self.codes.get(code)?;

        // Validate code.
        if auth_code.client_id != client_id {
            return None;
        }
        if auth_code.redirect_uri != redirect_uri {
            return None;
        }
        if chrono::Utc::now().timestamp() > auth_code.expires_at {
            return None;
        }

        // Validate PKCE challenge.
        let verifier_hash = base64_url::encode(
            &sha2::Sha256::digest(code_verifier.as_bytes())[..],
        );
        if verifier_hash != auth_code.code_challenge {
            return None;
        }

        // Extract needed data before possible removal.
        let scope = auth_code.scope.clone();
        let client_id_owned = auth_code.client_id.clone();

        // Explicitly drop the Ref guard before removing.
        drop(auth_code);

        // Remove the code (single-use).
        self.codes.remove(code);

        // Issue JWT access token.
        let now = chrono::Utc::now().timestamp();
        let claims = AccessTokenClaims {
            iss: "auroraview-mcp".to_string(),
            sub: client_id_owned,
            aud: "auroraview-mcp".to_string(),
            exp: now + 3600, // 1 hour
            iat: now,
            scope: scope.clone(),
        };

        let access_token = encode(&Header::default(), &claims, &self.encoding_key).ok()?;

        Some(TokenResponse {
            access_token,
            token_type: "Bearer",
            expires_in: 3600,
            refresh_token: None, // Simplified: no refresh token.
            scope,
        })
    }

    /// Validate JWT access token.
    pub fn validate_token(&self, token: &str) -> Option<AccessTokenClaims> {
        let mut validation = Validation::default();
        validation.iss = Some(HashSet::from(["auroraview-mcp".to_string()]));
        validation.aud = Some(HashSet::from(["auroraview-mcp".to_string()]));
        match decode::<AccessTokenClaims>(token, &self.decoding_key, &validation) {
            Ok(data) => Some(data.claims),
            Err(e) => {
                eprintln!("Token validation error: {:?}", e);
                None
            }
        }
    }
}

/// Extract bearer token from Authorization header.
pub fn extract_bearer_token(header: &str) -> Option<String> {
    header
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use jsonwebtoken::crypto::{CryptoProvider, aws_lc};

    static CRYPTO_INIT: OnceLock<()> = OnceLock::new();

    fn setup_crypto() {
        CRYPTO_INIT.get_or_init(|| {
            CryptoProvider::install_default(&aws_lc::DEFAULT_PROVIDER).unwrap();
        });
    }

    #[tokio::test]
    async fn oauth_store_creation() {
        setup_crypto();
        let _store = OAuthStore::new();
        // Should not panic - store is created successfully.
        // (jwt_secret is now internal to EncodingKey/DecodingKey)
    }

    #[tokio::test]
    async fn client_registration() {
        setup_crypto();
        let store = OAuthStore::new();
        let (client, secret) = store
            .register_client(
                "Test Client".to_string(),
                vec!["http://localhost:8080/callback".to_string()],
                "mcp:tools".to_string(),
            );

        assert_eq!(client.name, "Test Client");
        assert!(!secret.is_empty());
    }

    #[test]
    fn extract_bearer_token_valid() {
        let header = "Bearer abc123";
        let token = extract_bearer_token(header);
        assert_eq!(token, Some("abc123".to_string()));
    }

    #[test]
    fn extract_bearer_token_invalid() {
        let header = "Basic abc123";
        let token = extract_bearer_token(header);
        assert_eq!(token, None);
    }

    #[tokio::test]
    async fn issue_code_and_exchange() {
        setup_crypto();
        
        let store = OAuthStore::new();
        let (client, _secret) = store
            .register_client(
                "Test Client".to_string(),
                vec!["http://localhost:8080/callback".to_string()],
                "mcp:tools".to_string(),
            );

        // Simulate PKCE challenge
        let code_verifier = "test_verifier_12345678901234567890123456789012";
        let code_challenge = base64_url::encode(&sha2::Sha256::digest(code_verifier.as_bytes()));

        // Issue authorization code
        let code = store
            .issue_code(
                client.client_id.clone(),
                "http://localhost:8080/callback".to_string(),
                code_challenge,
                "mcp:tools".to_string(),
            );

        assert!(!code.is_empty());

        // Exchange code for token
        let token_resp = store
            .exchange_code(
                &code,
                &client.client_id,
                "http://localhost:8080/callback",
                code_verifier,
            );

        assert!(token_resp.is_some());
        let token_resp = token_resp.unwrap();
        assert!(!token_resp.access_token.is_empty());
        assert_eq!(token_resp.token_type, "Bearer");
        assert_eq!(token_resp.expires_in, 3600);
        assert_eq!(token_resp.scope, "mcp:tools");
    }

    #[tokio::test]
    async fn validate_token_success() {
        setup_crypto();
        
        let store = OAuthStore::new();
        // Use the full flow to get a valid token
        let (client, _secret) = store
            .register_client(
                "Test Client".to_string(),
                vec!["http://localhost:8080/callback".to_string()],
                "mcp:tools".to_string(),
            );

        let code_verifier = "test_verifier_12345678901234567890123456789012";
        let code_challenge = base64_url::encode(&sha2::Sha256::digest(code_verifier.as_bytes()));

        let code = store
            .issue_code(
                client.client_id.clone(),
                "http://localhost:8080/callback".to_string(),
                code_challenge,
                "mcp:tools".to_string(),
            );

        let token_resp = store
            .exchange_code(
                &code,
                &client.client_id,
                "http://localhost:8080/callback",
                code_verifier,
            )
            .unwrap();

        // Validate the token
        let validated = store.validate_token(&token_resp.access_token);
        assert!(validated.is_some());
        let claims = validated.unwrap();
        assert_eq!(claims.sub, client.client_id);
        assert_eq!(claims.scope, "mcp:tools");
    }

    #[tokio::test]
    async fn exchange_code_invalid_code_returns_none() {
        setup_crypto();
        
        let store = OAuthStore::new();
        let result = store
            .exchange_code(
                "invalid_code",
                "client_id",
                "http://localhost:8080/callback",
                "verifier",
            );

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn exchange_code_wrong_client_returns_none() {
        setup_crypto();
        
        let store = OAuthStore::new();
        let (client, _) = store
            .register_client(
                "Test Client".to_string(),
                vec!["http://localhost:8080/callback".to_string()],
                "mcp:tools".to_string(),
            );

        let code_verifier = "test_verifier_12345678901234567890123456789012";
        let code_challenge = base64_url::encode(&sha2::Sha256::digest(code_verifier.as_bytes()));

        let code = store
            .issue_code(
                client.client_id.clone(),
                "http://localhost:8080/callback".to_string(),
                code_challenge,
                "mcp:tools".to_string(),
            );

        // Try to exchange with wrong client_id
        let result = store
            .exchange_code(
                &code,
                "wrong_client_id",
                "http://localhost:8080/callback",
                code_verifier,
            );

        assert!(result.is_none());
    }
}
