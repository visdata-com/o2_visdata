// Copyright 2025 VisData Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Configuration for VisData module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for VisData module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisdataConfig {
    /// Whether RBAC is enabled
    pub rbac_enabled: bool,
    /// Whether SSO is enabled
    pub sso_enabled: bool,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Encryption key for sensitive data (base64 encoded, 32 bytes for AES-256)
    pub encryption_key: Option<String>,

    // ========================================================================
    // Enterprise Configuration (OpenFGA + Dex)
    // ========================================================================

    /// OpenFGA HTTP API URL
    #[serde(default = "default_openfga_url")]
    pub openfga_url: String,

    /// OpenFGA store name
    #[serde(default = "default_openfga_store_name")]
    pub openfga_store_name: String,

    /// Dex gRPC URL
    #[serde(default = "default_dex_grpc_url")]
    pub dex_grpc_url: String,

    /// Dex OIDC issuer URL
    #[serde(default = "default_dex_issuer_url")]
    pub dex_issuer_url: String,

    /// Dex OAuth2 client ID
    #[serde(default = "default_dex_client_id")]
    pub dex_client_id: String,

    /// Dex OAuth2 client secret
    #[serde(default)]
    pub dex_client_secret: String,

    /// Dex OAuth2 redirect URI
    #[serde(default = "default_dex_redirect_uri")]
    pub dex_redirect_uri: String,

    // ========================================================================
    // Log Patterns Configuration
    // ========================================================================

    /// Maximum number of logs to analyze for pattern extraction
    #[serde(default = "default_log_patterns_max_logs")]
    pub log_patterns_max_logs: usize,

    /// Minimum cluster size for a pattern to be considered valid
    #[serde(default = "default_log_patterns_min_cluster_size")]
    pub log_patterns_min_cluster_size: usize,

    /// Similarity threshold for grouping logs (0.0-1.0)
    #[serde(default = "default_log_patterns_similarity_threshold")]
    pub log_patterns_similarity_threshold: f64,

    /// Drain algorithm tree depth
    #[serde(default = "default_log_patterns_drain_depth")]
    pub log_patterns_drain_depth: usize,

    /// Maximum child nodes per tree node
    #[serde(default = "default_log_patterns_drain_max_child")]
    pub log_patterns_drain_max_child: usize,

    /// Maximum number of clusters/patterns to extract
    #[serde(default = "default_log_patterns_max_clusters")]
    pub log_patterns_max_clusters: usize,
}

fn default_openfga_url() -> String {
    "http://localhost:8080".to_string()
}

fn default_openfga_store_name() -> String {
    "openobserve".to_string()
}

fn default_dex_grpc_url() -> String {
    "http://localhost:5557".to_string()
}

fn default_dex_issuer_url() -> String {
    "http://localhost:5556".to_string()
}

fn default_dex_client_id() -> String {
    "openobserve".to_string()
}

fn default_dex_redirect_uri() -> String {
    "http://localhost:5080/config/redirect".to_string()
}

// Log Patterns defaults
fn default_log_patterns_max_logs() -> usize {
    10000
}

fn default_log_patterns_min_cluster_size() -> usize {
    2
}

fn default_log_patterns_similarity_threshold() -> f64 {
    0.6
}

fn default_log_patterns_drain_depth() -> usize {
    4
}

fn default_log_patterns_drain_max_child() -> usize {
    100
}

fn default_log_patterns_max_clusters() -> usize {
    1000
}

impl Default for VisdataConfig {
    fn default() -> Self {
        Self {
            rbac_enabled: true,
            sso_enabled: true,
            cache: CacheConfig::default(),
            encryption_key: None,
            // Enterprise defaults
            openfga_url: default_openfga_url(),
            openfga_store_name: default_openfga_store_name(),
            dex_grpc_url: default_dex_grpc_url(),
            dex_issuer_url: default_dex_issuer_url(),
            dex_client_id: default_dex_client_id(),
            dex_client_secret: String::new(),
            dex_redirect_uri: default_dex_redirect_uri(),
            // Log Patterns defaults
            log_patterns_max_logs: default_log_patterns_max_logs(),
            log_patterns_min_cluster_size: default_log_patterns_min_cluster_size(),
            log_patterns_similarity_threshold: default_log_patterns_similarity_threshold(),
            log_patterns_drain_depth: default_log_patterns_drain_depth(),
            log_patterns_drain_max_child: default_log_patterns_drain_max_child(),
            log_patterns_max_clusters: default_log_patterns_max_clusters(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether permission cache is enabled
    pub enabled: bool,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
    /// Maximum number of entries in cache
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 300, // 5 minutes
            max_entries: 10000,
        }
    }
}

/// OIDC provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OIDCConfig {
    /// OIDC issuer URL
    pub issuer_url: String,
    /// Client ID
    pub client_id: String,
    /// Client secret (encrypted in database)
    pub client_secret: String,
    /// Scopes to request
    #[serde(default = "default_oidc_scopes")]
    pub scopes: Vec<String>,
    /// Redirect URI after authentication
    pub redirect_uri: String,
    /// Claim name for email
    #[serde(default = "default_email_claim")]
    pub email_claim: String,
    /// Claim name for display name
    #[serde(default = "default_name_claim")]
    pub name_claim: String,
    /// Claim name for groups (optional)
    pub groups_claim: Option<String>,
    /// Mapping from external groups to internal roles
    #[serde(default)]
    pub group_role_mappings: HashMap<String, String>,
    /// Whether to auto-create users on first login
    #[serde(default = "default_true")]
    pub auto_create_users: bool,
    /// Default role for auto-created users
    pub default_role: Option<String>,
}

fn default_oidc_scopes() -> Vec<String> {
    vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
    ]
}

fn default_email_claim() -> String {
    "email".to_string()
}

fn default_name_claim() -> String {
    "name".to_string()
}

fn default_true() -> bool {
    true
}

/// LDAP provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LDAPConfig {
    /// LDAP server URL (e.g., ldap://ldap.example.com:389 or ldaps://...)
    pub server_url: String,
    /// Bind DN for connecting to LDAP
    pub bind_dn: String,
    /// Bind password (encrypted in database)
    pub bind_password: String,
    /// Base DN for user searches
    pub user_base_dn: String,
    /// LDAP filter for user searches (use {username} for username placeholder)
    #[serde(default = "default_user_filter")]
    pub user_filter: String,
    /// Attribute name for user email
    #[serde(default = "default_ldap_email_attr")]
    pub user_attr_email: String,
    /// Attribute name for user display name
    #[serde(default = "default_ldap_name_attr")]
    pub user_attr_name: String,
    /// Base DN for group searches (optional)
    #[serde(default)]
    pub group_base_dn: Option<String>,
    /// LDAP filter for group searches
    #[serde(default)]
    pub group_filter: Option<String>,
    /// Attribute name for group name
    #[serde(default = "default_ldap_group_attr")]
    pub group_attr_name: String,
    /// Mapping from LDAP groups to internal roles
    #[serde(default)]
    pub group_role_mappings: HashMap<String, String>,
    /// Whether to use SSL/TLS
    #[serde(default = "default_true")]
    pub use_ssl: bool,
    /// Skip TLS certificate verification (not recommended for production)
    #[serde(default)]
    pub skip_ssl_verify: bool,
    /// Connection timeout in seconds
    #[serde(default = "default_ldap_timeout")]
    pub timeout_seconds: u64,
}

fn default_user_filter() -> String {
    "(&(objectClass=person)(uid={0}))".to_string()
}

fn default_ldap_email_attr() -> String {
    "mail".to_string()
}

fn default_ldap_name_attr() -> String {
    "cn".to_string()
}

// Default group filter - kept for potential future use with serde
#[allow(dead_code)]
fn default_group_filter() -> String {
    "(&(objectClass=groupOfNames)(member={0}))".to_string()
}

fn default_ldap_group_attr() -> String {
    "cn".to_string()
}

fn default_ldap_timeout() -> u64 {
    10
}

/// SSO provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SSOProviderType {
    OIDC,
    LDAP,
}

impl std::fmt::Display for SSOProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SSOProviderType::OIDC => write!(f, "oidc"),
            SSOProviderType::LDAP => write!(f, "ldap"),
        }
    }
}

impl std::str::FromStr for SSOProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "oidc" => Ok(SSOProviderType::OIDC),
            "ldap" => Ok(SSOProviderType::LDAP),
            _ => Err(format!("Invalid SSO provider type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // VisdataConfig Tests
    // ========================================================================

    #[test]
    fn test_visdata_config_default() {
        let config = VisdataConfig::default();

        assert!(config.rbac_enabled);
        assert!(config.sso_enabled);
        assert!(config.encryption_key.is_none());

        // OpenFGA defaults
        assert_eq!(config.openfga_url, "http://localhost:8080");
        assert_eq!(config.openfga_store_name, "openobserve");

        // Dex defaults
        assert_eq!(config.dex_grpc_url, "http://localhost:5557");
        assert_eq!(config.dex_issuer_url, "http://localhost:5556");
        assert_eq!(config.dex_client_id, "openobserve");
        assert_eq!(config.dex_client_secret, "");
        assert_eq!(config.dex_redirect_uri, "http://localhost:5080/config/redirect");

        // Log patterns defaults
        assert_eq!(config.log_patterns_max_logs, 10000);
        assert_eq!(config.log_patterns_min_cluster_size, 2);
        assert!((config.log_patterns_similarity_threshold - 0.6).abs() < f64::EPSILON);
        assert_eq!(config.log_patterns_drain_depth, 4);
        assert_eq!(config.log_patterns_drain_max_child, 100);
        assert_eq!(config.log_patterns_max_clusters, 1000);
    }

    #[test]
    fn test_visdata_config_serialization() {
        let config = VisdataConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("rbac_enabled"));
        assert!(json.contains("sso_enabled"));
        assert!(json.contains("openfga_url"));
        assert!(json.contains("dex_grpc_url"));

        let deserialized: VisdataConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.rbac_enabled, deserialized.rbac_enabled);
        assert_eq!(config.openfga_url, deserialized.openfga_url);
    }

    #[test]
    fn test_visdata_config_partial_deserialization() {
        // Test that missing fields use defaults
        let json = r#"{
            "rbac_enabled": false,
            "sso_enabled": true,
            "cache": {"enabled": true, "ttl_seconds": 300, "max_entries": 10000}
        }"#;

        let config: VisdataConfig = serde_json::from_str(json).unwrap();

        assert!(!config.rbac_enabled);
        assert!(config.sso_enabled);
        // Check defaults are applied
        assert_eq!(config.openfga_url, "http://localhost:8080");
        assert_eq!(config.dex_client_id, "openobserve");
    }

    // ========================================================================
    // CacheConfig Tests
    // ========================================================================

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();

        assert!(config.enabled);
        assert_eq!(config.ttl_seconds, 300);
        assert_eq!(config.max_entries, 10000);
    }

    #[test]
    fn test_cache_config_serialization() {
        let config = CacheConfig {
            enabled: false,
            ttl_seconds: 600,
            max_entries: 5000,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CacheConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.ttl_seconds, deserialized.ttl_seconds);
        assert_eq!(config.max_entries, deserialized.max_entries);
    }

    // ========================================================================
    // OIDCConfig Tests
    // ========================================================================

    #[test]
    fn test_oidc_config_default_scopes() {
        let scopes = default_oidc_scopes();

        assert_eq!(scopes.len(), 3);
        assert!(scopes.contains(&"openid".to_string()));
        assert!(scopes.contains(&"profile".to_string()));
        assert!(scopes.contains(&"email".to_string()));
    }

    #[test]
    fn test_oidc_config_default_claims() {
        assert_eq!(default_email_claim(), "email");
        assert_eq!(default_name_claim(), "name");
    }

    #[test]
    fn test_oidc_config_serialization() {
        let config = OIDCConfig {
            issuer_url: "https://auth.example.com".to_string(),
            client_id: "my-app".to_string(),
            client_secret: "secret123".to_string(),
            scopes: default_oidc_scopes(),
            redirect_uri: "https://app.example.com/callback".to_string(),
            email_claim: "email".to_string(),
            name_claim: "name".to_string(),
            groups_claim: Some("groups".to_string()),
            group_role_mappings: HashMap::new(),
            auto_create_users: true,
            default_role: Some("viewer".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("https://auth.example.com"));
        assert!(json.contains("my-app"));
        assert!(json.contains("openid"));
        assert!(json.contains("groups"));

        let deserialized: OIDCConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.issuer_url, deserialized.issuer_url);
        assert_eq!(config.groups_claim, deserialized.groups_claim);
    }

    #[test]
    fn test_oidc_config_with_group_mappings() {
        let mut mappings = HashMap::new();
        mappings.insert("admins".to_string(), "admin".to_string());
        mappings.insert("developers".to_string(), "editor".to_string());

        let config = OIDCConfig {
            issuer_url: "https://auth.example.com".to_string(),
            client_id: "app".to_string(),
            client_secret: "secret".to_string(),
            scopes: vec!["openid".to_string()],
            redirect_uri: "https://app.example.com/cb".to_string(),
            email_claim: "email".to_string(),
            name_claim: "name".to_string(),
            groups_claim: Some("groups".to_string()),
            group_role_mappings: mappings,
            auto_create_users: true,
            default_role: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("admins"));
        assert!(json.contains("developers"));
    }

    // ========================================================================
    // LDAPConfig Tests
    // ========================================================================

    #[test]
    fn test_ldap_config_defaults() {
        assert_eq!(default_user_filter(), "(&(objectClass=person)(uid={0}))");
        assert_eq!(default_ldap_email_attr(), "mail");
        assert_eq!(default_ldap_name_attr(), "cn");
        assert_eq!(default_ldap_group_attr(), "cn");
        assert_eq!(default_ldap_timeout(), 10);
    }

    #[test]
    fn test_ldap_config_serialization() {
        let config = LDAPConfig {
            server_url: "ldaps://ldap.example.com:636".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "password123".to_string(),
            user_base_dn: "ou=users,dc=example,dc=com".to_string(),
            user_filter: default_user_filter(),
            user_attr_email: "mail".to_string(),
            user_attr_name: "cn".to_string(),
            group_base_dn: Some("ou=groups,dc=example,dc=com".to_string()),
            group_filter: Some("(&(objectClass=groupOfNames)(member={0}))".to_string()),
            group_attr_name: "cn".to_string(),
            group_role_mappings: HashMap::new(),
            use_ssl: true,
            skip_ssl_verify: false,
            timeout_seconds: 30,
        };

        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("ldaps://ldap.example.com:636"));
        assert!(json.contains("ou=users,dc=example,dc=com"));
        assert!(json.contains("\"use_ssl\":true"));
        assert!(json.contains("\"timeout_seconds\":30"));

        let deserialized: LDAPConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.server_url, deserialized.server_url);
        assert_eq!(config.use_ssl, deserialized.use_ssl);
    }

    #[test]
    fn test_ldap_config_minimal_deserialization() {
        let json = r#"{
            "server_url": "ldap://localhost:389",
            "bind_dn": "cn=admin,dc=test,dc=com",
            "bind_password": "secret",
            "user_base_dn": "ou=users,dc=test,dc=com"
        }"#;

        let config: LDAPConfig = serde_json::from_str(json).unwrap();

        // Check required fields
        assert_eq!(config.server_url, "ldap://localhost:389");
        assert_eq!(config.bind_dn, "cn=admin,dc=test,dc=com");

        // Check defaults are applied
        assert_eq!(config.user_filter, "(&(objectClass=person)(uid={0}))");
        assert_eq!(config.user_attr_email, "mail");
        assert!(config.use_ssl);
        assert!(!config.skip_ssl_verify);
        assert_eq!(config.timeout_seconds, 10);
    }

    // ========================================================================
    // SSOProviderType Tests
    // ========================================================================

    #[test]
    fn test_sso_provider_type_display() {
        assert_eq!(format!("{}", SSOProviderType::OIDC), "oidc");
        assert_eq!(format!("{}", SSOProviderType::LDAP), "ldap");
    }

    #[test]
    fn test_sso_provider_type_from_str() {
        assert_eq!("oidc".parse::<SSOProviderType>().unwrap(), SSOProviderType::OIDC);
        assert_eq!("OIDC".parse::<SSOProviderType>().unwrap(), SSOProviderType::OIDC);
        assert_eq!("Oidc".parse::<SSOProviderType>().unwrap(), SSOProviderType::OIDC);

        assert_eq!("ldap".parse::<SSOProviderType>().unwrap(), SSOProviderType::LDAP);
        assert_eq!("LDAP".parse::<SSOProviderType>().unwrap(), SSOProviderType::LDAP);
        assert_eq!("Ldap".parse::<SSOProviderType>().unwrap(), SSOProviderType::LDAP);
    }

    #[test]
    fn test_sso_provider_type_from_str_invalid() {
        let result = "invalid".parse::<SSOProviderType>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid SSO provider type"));

        let result = "saml".parse::<SSOProviderType>();
        assert!(result.is_err());
    }

    #[test]
    fn test_sso_provider_type_serialization() {
        let oidc = SSOProviderType::OIDC;
        let json = serde_json::to_string(&oidc).unwrap();
        assert_eq!(json, "\"oidc\"");

        let ldap = SSOProviderType::LDAP;
        let json = serde_json::to_string(&ldap).unwrap();
        assert_eq!(json, "\"ldap\"");

        // Deserialization
        let deserialized: SSOProviderType = serde_json::from_str("\"oidc\"").unwrap();
        assert_eq!(deserialized, SSOProviderType::OIDC);

        let deserialized: SSOProviderType = serde_json::from_str("\"ldap\"").unwrap();
        assert_eq!(deserialized, SSOProviderType::LDAP);
    }

    #[test]
    fn test_sso_provider_type_equality() {
        assert_eq!(SSOProviderType::OIDC, SSOProviderType::OIDC);
        assert_eq!(SSOProviderType::LDAP, SSOProviderType::LDAP);
        assert_ne!(SSOProviderType::OIDC, SSOProviderType::LDAP);
    }

    #[test]
    fn test_sso_provider_type_copy() {
        let provider = SSOProviderType::OIDC;
        let copied = provider; // Copy
        assert_eq!(provider, copied);
    }
}
