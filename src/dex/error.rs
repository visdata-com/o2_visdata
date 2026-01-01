// Copyright 2025 VisData Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Authentication error types

use actix_web::{HttpResponse, ResponseError};
use std::fmt;

/// Result type for auth operations
pub type Result<T> = std::result::Result<T, Error>;

/// Auth-specific error types
#[derive(Debug)]
pub enum Error {
    /// Invalid credentials
    InvalidCredentials(String),
    /// Token validation failed
    InvalidToken(String),
    /// Token expired
    TokenExpired,
    /// User not found
    UserNotFound(String),
    /// Connector not found
    ConnectorNotFound(String),
    /// Connector already exists
    ConnectorExists(String),
    /// Invalid connector configuration
    InvalidConnector(String),
    /// gRPC communication error
    GrpcError(String),
    /// HTTP communication error
    HttpError(String),
    /// Configuration error
    ConfigError(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCredentials(msg) => write!(f, "Invalid credentials: {}", msg),
            Error::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            Error::TokenExpired => write!(f, "Token has expired"),
            Error::UserNotFound(user) => write!(f, "User not found: {}", user),
            Error::ConnectorNotFound(id) => write!(f, "Connector not found: {}", id),
            Error::ConnectorExists(id) => write!(f, "Connector already exists: {}", id),
            Error::InvalidConnector(msg) => write!(f, "Invalid connector configuration: {}", msg),
            Error::GrpcError(msg) => write!(f, "gRPC error: {}", msg),
            Error::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let (status, code) = match self {
            Error::InvalidCredentials(_) => (actix_web::http::StatusCode::UNAUTHORIZED, 401),
            Error::InvalidToken(_) => (actix_web::http::StatusCode::UNAUTHORIZED, 401),
            Error::TokenExpired => (actix_web::http::StatusCode::UNAUTHORIZED, 401),
            Error::UserNotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, 404),
            Error::ConnectorNotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, 404),
            Error::ConnectorExists(_) => (actix_web::http::StatusCode::CONFLICT, 409),
            Error::InvalidConnector(_) => (actix_web::http::StatusCode::BAD_REQUEST, 400),
            Error::GrpcError(_) => (actix_web::http::StatusCode::BAD_GATEWAY, 502),
            Error::HttpError(_) => (actix_web::http::StatusCode::BAD_GATEWAY, 502),
            Error::ConfigError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, 500),
            Error::Internal(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, 500),
        };

        HttpResponse::build(status).json(serde_json::json!({
            "code": code,
            "message": self.to_string()
        }))
    }
}

impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        Error::GrpcError(status.message().to_string())
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Error::GrpcError(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::HttpError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Error::InvalidToken(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    // ========================================================================
    // Error Display Tests
    // ========================================================================

    #[test]
    fn test_error_display_invalid_credentials() {
        let err = Error::InvalidCredentials("wrong password".to_string());
        assert_eq!(format!("{}", err), "Invalid credentials: wrong password");
    }

    #[test]
    fn test_error_display_invalid_token() {
        let err = Error::InvalidToken("malformed JWT".to_string());
        assert_eq!(format!("{}", err), "Invalid token: malformed JWT");
    }

    #[test]
    fn test_error_display_token_expired() {
        let err = Error::TokenExpired;
        assert_eq!(format!("{}", err), "Token has expired");
    }

    #[test]
    fn test_error_display_user_not_found() {
        let err = Error::UserNotFound("user@example.com".to_string());
        assert_eq!(format!("{}", err), "User not found: user@example.com");
    }

    #[test]
    fn test_error_display_connector_not_found() {
        let err = Error::ConnectorNotFound("ldap-prod".to_string());
        assert_eq!(format!("{}", err), "Connector not found: ldap-prod");
    }

    #[test]
    fn test_error_display_connector_exists() {
        let err = Error::ConnectorExists("oidc-dev".to_string());
        assert_eq!(format!("{}", err), "Connector already exists: oidc-dev");
    }

    #[test]
    fn test_error_display_invalid_connector() {
        let err = Error::InvalidConnector("missing client_id".to_string());
        assert_eq!(
            format!("{}", err),
            "Invalid connector configuration: missing client_id"
        );
    }

    #[test]
    fn test_error_display_grpc_error() {
        let err = Error::GrpcError("connection refused".to_string());
        assert_eq!(format!("{}", err), "gRPC error: connection refused");
    }

    #[test]
    fn test_error_display_http_error() {
        let err = Error::HttpError("timeout".to_string());
        assert_eq!(format!("{}", err), "HTTP error: timeout");
    }

    #[test]
    fn test_error_display_config_error() {
        let err = Error::ConfigError("missing DEX_URL".to_string());
        assert_eq!(format!("{}", err), "Configuration error: missing DEX_URL");
    }

    #[test]
    fn test_error_display_internal() {
        let err = Error::Internal("unexpected state".to_string());
        assert_eq!(format!("{}", err), "Internal error: unexpected state");
    }

    // ========================================================================
    // Error Response Status Code Tests
    // ========================================================================

    #[test]
    fn test_error_response_invalid_credentials() {
        let err = Error::InvalidCredentials("bad password".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_error_response_invalid_token() {
        let err = Error::InvalidToken("bad token".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_error_response_token_expired() {
        let err = Error::TokenExpired;
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_error_response_user_not_found() {
        let err = Error::UserNotFound("unknown@test.com".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_connector_not_found() {
        let err = Error::ConnectorNotFound("missing-connector".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_connector_exists() {
        let err = Error::ConnectorExists("duplicate".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_error_response_invalid_connector() {
        let err = Error::InvalidConnector("bad config".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_response_grpc_error() {
        let err = Error::GrpcError("unavailable".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_error_response_http_error() {
        let err = Error::HttpError("network error".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_error_response_config_error() {
        let err = Error::ConfigError("invalid config".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_internal() {
        let err = Error::Internal("panic".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ========================================================================
    // From Trait Tests
    // ========================================================================

    #[test]
    fn test_from_tonic_status() {
        let status = tonic::Status::not_found("resource not found");
        let err: Error = status.into();

        match err {
            Error::GrpcError(msg) => {
                assert_eq!(msg, "resource not found");
            }
            _ => panic!("Expected GrpcError"),
        }
    }

    #[test]
    fn test_from_tonic_status_various_codes() {
        // Test different gRPC status codes
        let codes = vec![
            tonic::Status::invalid_argument("bad arg"),
            tonic::Status::unauthenticated("no auth"),
            tonic::Status::permission_denied("forbidden"),
            tonic::Status::internal("server error"),
        ];

        for status in codes {
            let err: Error = status.into();
            match err {
                Error::GrpcError(_) => {}
                _ => panic!("Expected GrpcError"),
            }
        }
    }

    #[test]
    fn test_from_jsonwebtoken_error() {
        // Create a JWT error by trying to decode an invalid token
        let result = jsonwebtoken::decode::<serde_json::Value>(
            "invalid.token.here",
            &jsonwebtoken::DecodingKey::from_secret(b"secret"),
            &jsonwebtoken::Validation::default(),
        );

        let jwt_err = result.unwrap_err();
        let err: Error = jwt_err.into();

        match err {
            Error::InvalidToken(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected InvalidToken error"),
        }
    }

    // ========================================================================
    // std::error::Error Trait Tests
    // ========================================================================

    #[test]
    fn test_error_is_std_error() {
        let err = Error::Internal("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_error_debug() {
        let err = Error::InvalidCredentials("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidCredentials"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_error_debug_token_expired() {
        let err = Error::TokenExpired;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("TokenExpired"));
    }

    // ========================================================================
    // Result Type Alias Tests
    // ========================================================================

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(Error::Internal("fail".to_string()));
        assert!(result.is_err());
    }

    // ========================================================================
    // Error Message Preservation Tests
    // ========================================================================

    #[test]
    fn test_error_preserves_empty_message() {
        let err = Error::Internal("".to_string());
        assert_eq!(format!("{}", err), "Internal error: ");
    }

    #[test]
    fn test_error_preserves_special_characters() {
        let err = Error::InvalidCredentials("user: 'test' with \"quotes\"".to_string());
        assert!(format!("{}", err).contains("user: 'test' with \"quotes\""));
    }

    #[test]
    fn test_error_preserves_unicode() {
        let err = Error::UserNotFound("用户@example.com".to_string());
        assert_eq!(format!("{}", err), "User not found: 用户@example.com");
    }

    #[test]
    fn test_error_preserves_newlines() {
        let err = Error::ConfigError("line1\nline2".to_string());
        assert!(format!("{}", err).contains("line1\nline2"));
    }
}
