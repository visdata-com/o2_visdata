// Copyright 2025 VisData Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! RBAC module error types

use actix_web::{HttpResponse, ResponseError};
use std::fmt;

/// Result type alias for RBAC operations
pub type Result<T> = std::result::Result<T, Error>;

/// RBAC error types
#[derive(Debug)]
pub enum Error {
    /// Not initialized
    NotInitialized(String),

    /// OpenFGA API error
    OpenFGA(String),

    /// HTTP client error
    Http(reqwest::Error),

    /// Store not found or not created
    StoreNotFound,

    /// Model not found or not created
    ModelNotFound,

    /// Role not found
    RoleNotFound(String),

    /// Group not found
    GroupNotFound(String),

    /// User not found
    UserNotFound(String),

    /// Permission denied
    PermissionDenied(String),

    /// Invalid permission type
    InvalidPermission(String),

    /// Invalid resource type
    InvalidResourceType(String),

    /// Duplicate entry
    DuplicateEntry(String),

    /// Validation error
    Validation(String),

    /// Serialization error
    Serialization(String),

    /// Configuration error
    Config(String),

    /// Internal error
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotInitialized(msg) => write!(f, "Not initialized: {}", msg),
            Error::OpenFGA(msg) => write!(f, "OpenFGA error: {}", msg),
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::StoreNotFound => write!(f, "OpenFGA store not found"),
            Error::ModelNotFound => write!(f, "OpenFGA authorization model not found"),
            Error::RoleNotFound(name) => write!(f, "Role not found: {}", name),
            Error::GroupNotFound(name) => write!(f, "Group not found: {}", name),
            Error::UserNotFound(email) => write!(f, "User not found: {}", email),
            Error::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Error::InvalidPermission(perm) => write!(f, "Invalid permission: {}", perm),
            Error::InvalidResourceType(rt) => write!(f, "Invalid resource type: {}", rt),
            Error::DuplicateEntry(msg) => write!(f, "Duplicate entry: {}", msg),
            Error::Validation(msg) => write!(f, "Validation error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::NotInitialized(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "message": self.to_string()
            })),
            Error::RoleNotFound(_) | Error::GroupNotFound(_) | Error::UserNotFound(_) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "message": self.to_string()
                }))
            }
            Error::PermissionDenied(_) => HttpResponse::Forbidden().json(serde_json::json!({
                "message": self.to_string()
            })),
            Error::DuplicateEntry(_) => HttpResponse::Conflict().json(serde_json::json!({
                "message": self.to_string()
            })),
            Error::InvalidPermission(_) | Error::InvalidResourceType(_) | Error::Validation(_) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "message": self.to_string()
                }))
            }
            _ => HttpResponse::InternalServerError().json(serde_json::json!({
                "message": self.to_string()
            })),
        }
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
    fn test_error_display_not_initialized() {
        let err = Error::NotInitialized("OpenFGA client".to_string());
        assert_eq!(format!("{}", err), "Not initialized: OpenFGA client");
    }

    #[test]
    fn test_error_display_openfga() {
        let err = Error::OpenFGA("connection failed".to_string());
        assert_eq!(format!("{}", err), "OpenFGA error: connection failed");
    }

    #[test]
    fn test_error_display_store_not_found() {
        let err = Error::StoreNotFound;
        assert_eq!(format!("{}", err), "OpenFGA store not found");
    }

    #[test]
    fn test_error_display_model_not_found() {
        let err = Error::ModelNotFound;
        assert_eq!(format!("{}", err), "OpenFGA authorization model not found");
    }

    #[test]
    fn test_error_display_role_not_found() {
        let err = Error::RoleNotFound("custom_role".to_string());
        assert_eq!(format!("{}", err), "Role not found: custom_role");
    }

    #[test]
    fn test_error_display_group_not_found() {
        let err = Error::GroupNotFound("developers".to_string());
        assert_eq!(format!("{}", err), "Group not found: developers");
    }

    #[test]
    fn test_error_display_user_not_found() {
        let err = Error::UserNotFound("user@example.com".to_string());
        assert_eq!(format!("{}", err), "User not found: user@example.com");
    }

    #[test]
    fn test_error_display_permission_denied() {
        let err = Error::PermissionDenied("insufficient privileges".to_string());
        assert_eq!(format!("{}", err), "Permission denied: insufficient privileges");
    }

    #[test]
    fn test_error_display_invalid_permission() {
        let err = Error::InvalidPermission("ReadWrite".to_string());
        assert_eq!(format!("{}", err), "Invalid permission: ReadWrite");
    }

    #[test]
    fn test_error_display_invalid_resource_type() {
        let err = Error::InvalidResourceType("unknown_type".to_string());
        assert_eq!(format!("{}", err), "Invalid resource type: unknown_type");
    }

    #[test]
    fn test_error_display_duplicate_entry() {
        let err = Error::DuplicateEntry("role already exists".to_string());
        assert_eq!(format!("{}", err), "Duplicate entry: role already exists");
    }

    #[test]
    fn test_error_display_validation() {
        let err = Error::Validation("role name cannot be empty".to_string());
        assert_eq!(format!("{}", err), "Validation error: role name cannot be empty");
    }

    #[test]
    fn test_error_display_serialization() {
        let err = Error::Serialization("invalid JSON".to_string());
        assert_eq!(format!("{}", err), "Serialization error: invalid JSON");
    }

    #[test]
    fn test_error_display_config() {
        let err = Error::Config("missing OpenFGA URL".to_string());
        assert_eq!(format!("{}", err), "Configuration error: missing OpenFGA URL");
    }

    #[test]
    fn test_error_display_internal() {
        let err = Error::Internal("unexpected error".to_string());
        assert_eq!(format!("{}", err), "Internal error: unexpected error");
    }

    // ========================================================================
    // Error Response Status Code Tests
    // ========================================================================

    #[test]
    fn test_error_response_not_initialized() {
        let err = Error::NotInitialized("test".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_error_response_role_not_found() {
        let err = Error::RoleNotFound("admin".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_group_not_found() {
        let err = Error::GroupNotFound("devs".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_user_not_found() {
        let err = Error::UserNotFound("user@test.com".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_permission_denied() {
        let err = Error::PermissionDenied("access denied".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_error_response_duplicate_entry() {
        let err = Error::DuplicateEntry("exists".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_error_response_invalid_permission() {
        let err = Error::InvalidPermission("bad".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_response_invalid_resource_type() {
        let err = Error::InvalidResourceType("bad".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_response_validation() {
        let err = Error::Validation("invalid".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_response_openfga() {
        let err = Error::OpenFGA("api error".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_store_not_found() {
        let err = Error::StoreNotFound;
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_model_not_found() {
        let err = Error::ModelNotFound;
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_serialization() {
        let err = Error::Serialization("json error".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_config() {
        let err = Error::Config("bad config".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_internal() {
        let err = Error::Internal("unexpected".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ========================================================================
    // From Trait Tests
    // ========================================================================

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<String>("invalid json").unwrap_err();
        let err: Error = json_err.into();

        match err {
            Error::Serialization(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected Serialization error"),
        }
    }

    // ========================================================================
    // std::error::Error Trait Tests
    // ========================================================================

    #[test]
    fn test_error_source_none() {
        use std::error::Error as StdError;
        let err = Error::RoleNotFound("test".to_string());
        assert!(err.source().is_none());
    }

    #[test]
    fn test_error_debug() {
        let err = Error::RoleNotFound("custom".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("RoleNotFound"));
        assert!(debug_str.contains("custom"));
    }
}
