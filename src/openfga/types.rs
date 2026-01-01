// Copyright 2025 VisData Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! RBAC types compatible with existing API formats

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// OpenFGA Types
// ============================================================================

/// OpenFGA tuple key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TupleKey {
    pub user: String,
    pub relation: String,
    pub object: String,
}

impl TupleKey {
    pub fn new(user: impl Into<String>, relation: impl Into<String>, object: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            relation: relation.into(),
            object: object.into(),
        }
    }
}

/// OpenFGA tuple with optional condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tuple {
    pub key: TupleKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// OpenFGA check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRequest {
    pub tuple_key: TupleKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
}

/// OpenFGA check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResponse {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// OpenFGA write request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writes: Option<TupleKeys>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletes: Option<TupleKeys>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
}

/// Wrapper for tuple keys array
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TupleKeys {
    pub tuple_keys: Vec<TupleKey>,
}

/// OpenFGA list objects request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectsRequest {
    pub user: String,
    pub relation: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
}

/// OpenFGA list objects response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectsResponse {
    pub objects: Vec<String>,
}

/// OpenFGA read request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuple_key: Option<TupleKeyFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

/// Tuple key filter for read operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TupleKeyFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
}

/// OpenFGA read response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResponse {
    pub tuples: Vec<Tuple>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

/// OpenFGA store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Create store request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStoreRequest {
    pub name: String,
}

/// Create store response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStoreResponse {
    pub id: String,
    pub name: String,
}

/// List stores response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStoresResponse {
    pub stores: Vec<Store>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

// ============================================================================
// API Request/Response Types (Compatible with existing API)
// ============================================================================

/// Create role request (compatible with existing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub role: String,
}

/// Update role request (compatible with existing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    #[serde(default)]
    pub add: Option<Vec<PermissionEntry>>,
    #[serde(default)]
    pub remove: Option<Vec<PermissionEntry>>,
    #[serde(default)]
    pub add_users: Option<HashSet<String>>,
    #[serde(default)]
    pub remove_users: Option<HashSet<String>>,
}

/// Permission entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PermissionEntry {
    /// Resource object in format "resource:entity" (e.g., "logs:my_stream")
    pub object: String,
    /// Permission type: AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete
    pub permission: String,
}

/// Create group request (compatible with existing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Update group request (compatible with existing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    #[serde(default)]
    pub add_roles: Option<HashSet<String>>,
    #[serde(default)]
    pub remove_roles: Option<HashSet<String>>,
    #[serde(default)]
    pub add_users: Option<HashSet<String>>,
    #[serde(default)]
    pub remove_users: Option<HashSet<String>>,
}

/// Group response (compatible with existing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub roles: Vec<String>,
    pub users: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// User role option for dropdown (compatible with existing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleOption {
    pub label: String,
    pub value: String,
}

/// Role response (compatible with existing API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleResponse {
    pub name: String,
    pub label: String,
    pub users: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Resource definition (compatible with OFGA_MODELS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub key: String,
    /// Display name (serialized as display_name for frontend compatibility)
    #[serde(rename = "display_name")]
    pub label: String,
    #[serde(default)]
    pub parent: Option<String>,
    pub order: i32,
    pub visible: bool,
    /// Whether this is a top-level resource (no parent)
    #[serde(default)]
    pub top_level: bool,
    /// Whether this resource can have entity instances
    #[serde(default)]
    pub has_entities: bool,
}

// ============================================================================
// Permission Types
// ============================================================================

/// Permission type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    AllowAll,
    AllowList,
    AllowGet,
    AllowPost,
    AllowPut,
    AllowDelete,
}

impl Permission {
    /// Convert from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "allowall" | "allow_all" => Some(Permission::AllowAll),
            "allowlist" | "allow_list" => Some(Permission::AllowList),
            "allowget" | "allow_get" => Some(Permission::AllowGet),
            "allowpost" | "allow_post" => Some(Permission::AllowPost),
            "allowput" | "allow_put" => Some(Permission::AllowPut),
            "allowdelete" | "allow_delete" => Some(Permission::AllowDelete),
            _ => None,
        }
    }

    /// Convert to OpenFGA relation name
    pub fn to_relation(&self) -> &'static str {
        match self {
            Permission::AllowAll => "admin",
            Permission::AllowList => "can_list",
            Permission::AllowGet => "can_read",
            Permission::AllowPost => "can_create",
            Permission::AllowPut => "can_update",
            Permission::AllowDelete => "can_delete",
        }
    }

    /// Convert from HTTP method
    pub fn from_method(method: &str, is_list: bool) -> Self {
        match method.to_uppercase().as_str() {
            "GET" if is_list => Permission::AllowList,
            "GET" => Permission::AllowGet,
            "POST" => Permission::AllowPost,
            "PUT" | "PATCH" => Permission::AllowPut,
            "DELETE" => Permission::AllowDelete,
            _ => Permission::AllowGet,
        }
    }

    /// Check if this permission implies another permission
    pub fn implies(&self, other: &Permission) -> bool {
        match self {
            Permission::AllowAll => true,
            _ => self == other,
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::AllowAll => write!(f, "AllowAll"),
            Permission::AllowList => write!(f, "AllowList"),
            Permission::AllowGet => write!(f, "AllowGet"),
            Permission::AllowPost => write!(f, "AllowPost"),
            Permission::AllowPut => write!(f, "AllowPut"),
            Permission::AllowDelete => write!(f, "AllowDelete"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TupleKey Tests
    // ========================================================================

    #[test]
    fn test_tuple_key_new() {
        let key = TupleKey::new("user:alice", "viewer", "document:1");
        assert_eq!(key.user, "user:alice");
        assert_eq!(key.relation, "viewer");
        assert_eq!(key.object, "document:1");
    }

    #[test]
    fn test_tuple_key_new_with_string() {
        let key = TupleKey::new(
            String::from("user:bob"),
            String::from("editor"),
            String::from("folder:root"),
        );
        assert_eq!(key.user, "user:bob");
        assert_eq!(key.relation, "editor");
        assert_eq!(key.object, "folder:root");
    }

    #[test]
    fn test_tuple_key_equality() {
        let key1 = TupleKey::new("user:alice", "viewer", "doc:1");
        let key2 = TupleKey::new("user:alice", "viewer", "doc:1");
        let key3 = TupleKey::new("user:bob", "viewer", "doc:1");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_tuple_key_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(TupleKey::new("user:alice", "viewer", "doc:1"));
        set.insert(TupleKey::new("user:alice", "viewer", "doc:1")); // duplicate

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_tuple_key_serialization() {
        let key = TupleKey::new("user:alice", "viewer", "doc:1");
        let json = serde_json::to_string(&key).unwrap();
        let deserialized: TupleKey = serde_json::from_str(&json).unwrap();

        assert_eq!(key, deserialized);
    }

    // ========================================================================
    // Permission Tests
    // ========================================================================

    #[test]
    fn test_permission_from_str_camel_case() {
        assert_eq!(Permission::from_str("AllowAll"), Some(Permission::AllowAll));
        assert_eq!(Permission::from_str("AllowList"), Some(Permission::AllowList));
        assert_eq!(Permission::from_str("AllowGet"), Some(Permission::AllowGet));
        assert_eq!(Permission::from_str("AllowPost"), Some(Permission::AllowPost));
        assert_eq!(Permission::from_str("AllowPut"), Some(Permission::AllowPut));
        assert_eq!(Permission::from_str("AllowDelete"), Some(Permission::AllowDelete));
    }

    #[test]
    fn test_permission_from_str_snake_case() {
        assert_eq!(Permission::from_str("allow_all"), Some(Permission::AllowAll));
        assert_eq!(Permission::from_str("allow_list"), Some(Permission::AllowList));
        assert_eq!(Permission::from_str("allow_get"), Some(Permission::AllowGet));
        assert_eq!(Permission::from_str("allow_post"), Some(Permission::AllowPost));
        assert_eq!(Permission::from_str("allow_put"), Some(Permission::AllowPut));
        assert_eq!(Permission::from_str("allow_delete"), Some(Permission::AllowDelete));
    }

    #[test]
    fn test_permission_from_str_lowercase() {
        assert_eq!(Permission::from_str("allowall"), Some(Permission::AllowAll));
        assert_eq!(Permission::from_str("allowget"), Some(Permission::AllowGet));
    }

    #[test]
    fn test_permission_from_str_invalid() {
        assert_eq!(Permission::from_str("invalid"), None);
        assert_eq!(Permission::from_str(""), None);
        assert_eq!(Permission::from_str("read"), None);
        assert_eq!(Permission::from_str("write"), None);
    }

    #[test]
    fn test_permission_to_relation() {
        assert_eq!(Permission::AllowAll.to_relation(), "admin");
        assert_eq!(Permission::AllowList.to_relation(), "can_list");
        assert_eq!(Permission::AllowGet.to_relation(), "can_read");
        assert_eq!(Permission::AllowPost.to_relation(), "can_create");
        assert_eq!(Permission::AllowPut.to_relation(), "can_update");
        assert_eq!(Permission::AllowDelete.to_relation(), "can_delete");
    }

    #[test]
    fn test_permission_from_method_get() {
        assert_eq!(Permission::from_method("GET", false), Permission::AllowGet);
        assert_eq!(Permission::from_method("get", false), Permission::AllowGet);
    }

    #[test]
    fn test_permission_from_method_get_list() {
        assert_eq!(Permission::from_method("GET", true), Permission::AllowList);
    }

    #[test]
    fn test_permission_from_method_post() {
        assert_eq!(Permission::from_method("POST", false), Permission::AllowPost);
        assert_eq!(Permission::from_method("post", false), Permission::AllowPost);
    }

    #[test]
    fn test_permission_from_method_put_patch() {
        assert_eq!(Permission::from_method("PUT", false), Permission::AllowPut);
        assert_eq!(Permission::from_method("PATCH", false), Permission::AllowPut);
    }

    #[test]
    fn test_permission_from_method_delete() {
        assert_eq!(Permission::from_method("DELETE", false), Permission::AllowDelete);
    }

    #[test]
    fn test_permission_from_method_unknown() {
        assert_eq!(Permission::from_method("OPTIONS", false), Permission::AllowGet);
        assert_eq!(Permission::from_method("HEAD", false), Permission::AllowGet);
        assert_eq!(Permission::from_method("UNKNOWN", false), Permission::AllowGet);
    }

    #[test]
    fn test_permission_implies_allow_all() {
        // AllowAll implies all permissions
        assert!(Permission::AllowAll.implies(&Permission::AllowAll));
        assert!(Permission::AllowAll.implies(&Permission::AllowList));
        assert!(Permission::AllowAll.implies(&Permission::AllowGet));
        assert!(Permission::AllowAll.implies(&Permission::AllowPost));
        assert!(Permission::AllowAll.implies(&Permission::AllowPut));
        assert!(Permission::AllowAll.implies(&Permission::AllowDelete));
    }

    #[test]
    fn test_permission_implies_self() {
        assert!(Permission::AllowGet.implies(&Permission::AllowGet));
        assert!(Permission::AllowList.implies(&Permission::AllowList));
        assert!(Permission::AllowPost.implies(&Permission::AllowPost));
        assert!(Permission::AllowPut.implies(&Permission::AllowPut));
        assert!(Permission::AllowDelete.implies(&Permission::AllowDelete));
    }

    #[test]
    fn test_permission_does_not_imply_others() {
        assert!(!Permission::AllowGet.implies(&Permission::AllowPost));
        assert!(!Permission::AllowGet.implies(&Permission::AllowDelete));
        assert!(!Permission::AllowList.implies(&Permission::AllowPut));
        assert!(!Permission::AllowPost.implies(&Permission::AllowGet));
    }

    #[test]
    fn test_permission_display() {
        assert_eq!(format!("{}", Permission::AllowAll), "AllowAll");
        assert_eq!(format!("{}", Permission::AllowList), "AllowList");
        assert_eq!(format!("{}", Permission::AllowGet), "AllowGet");
        assert_eq!(format!("{}", Permission::AllowPost), "AllowPost");
        assert_eq!(format!("{}", Permission::AllowPut), "AllowPut");
        assert_eq!(format!("{}", Permission::AllowDelete), "AllowDelete");
    }

    #[test]
    fn test_permission_serialization() {
        let perm = Permission::AllowGet;
        let json = serde_json::to_string(&perm).unwrap();
        assert_eq!(json, "\"AllowGet\"");

        let deserialized: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(perm, deserialized);
    }

    // ========================================================================
    // PermissionEntry Tests
    // ========================================================================

    #[test]
    fn test_permission_entry_equality() {
        let entry1 = PermissionEntry {
            object: "logs:my_stream".to_string(),
            permission: "AllowGet".to_string(),
        };
        let entry2 = PermissionEntry {
            object: "logs:my_stream".to_string(),
            permission: "AllowGet".to_string(),
        };
        let entry3 = PermissionEntry {
            object: "logs:other_stream".to_string(),
            permission: "AllowGet".to_string(),
        };

        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }

    #[test]
    fn test_permission_entry_hash() {
        let mut set = HashSet::new();
        set.insert(PermissionEntry {
            object: "logs:stream1".to_string(),
            permission: "AllowGet".to_string(),
        });
        set.insert(PermissionEntry {
            object: "logs:stream1".to_string(),
            permission: "AllowGet".to_string(),
        }); // duplicate

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_permission_entry_serialization() {
        let entry = PermissionEntry {
            object: "dashboard:my_dash".to_string(),
            permission: "AllowAll".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"object\":\"dashboard:my_dash\""));
        assert!(json.contains("\"permission\":\"AllowAll\""));

        let deserialized: PermissionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    // ========================================================================
    // UpdateRoleRequest Tests
    // ========================================================================

    #[test]
    fn test_update_role_request_empty() {
        let req = UpdateRoleRequest {
            add: None,
            remove: None,
            add_users: None,
            remove_users: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: UpdateRoleRequest = serde_json::from_str(&json).unwrap();

        assert!(deserialized.add.is_none());
        assert!(deserialized.remove.is_none());
    }

    #[test]
    fn test_update_role_request_with_permissions() {
        let mut add_users = HashSet::new();
        add_users.insert("user@example.com".to_string());

        let req = UpdateRoleRequest {
            add: Some(vec![PermissionEntry {
                object: "logs:_all_default".to_string(),
                permission: "AllowGet".to_string(),
            }]),
            remove: None,
            add_users: Some(add_users),
            remove_users: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("logs:_all_default"));
        assert!(json.contains("user@example.com"));
    }

    #[test]
    fn test_update_role_request_deserialization_from_api() {
        let json = r#"{
            "add": [{"object": "logs:stream1", "permission": "AllowGet"}],
            "remove": [{"object": "logs:stream2", "permission": "AllowPost"}],
            "add_users": ["alice@example.com"],
            "remove_users": ["bob@example.com"]
        }"#;

        let req: UpdateRoleRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.add.as_ref().unwrap().len(), 1);
        assert_eq!(req.remove.as_ref().unwrap().len(), 1);
        assert!(req.add_users.as_ref().unwrap().contains("alice@example.com"));
        assert!(req.remove_users.as_ref().unwrap().contains("bob@example.com"));
    }

    // ========================================================================
    // TupleKeyFilter Tests
    // ========================================================================

    #[test]
    fn test_tuple_key_filter_default() {
        let filter = TupleKeyFilter::default();
        assert!(filter.user.is_none());
        assert!(filter.relation.is_none());
        assert!(filter.object.is_none());
    }

    #[test]
    fn test_tuple_key_filter_partial() {
        let filter = TupleKeyFilter {
            user: Some("user:alice".to_string()),
            relation: None,
            object: Some("doc:1".to_string()),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("user:alice"));
        assert!(json.contains("doc:1"));
        assert!(!json.contains("relation")); // skip_serializing_if = None
    }

    // ========================================================================
    // CreateGroupRequest Tests
    // ========================================================================

    #[test]
    fn test_create_group_request_minimal() {
        let req = CreateGroupRequest {
            name: "developers".to_string(),
            display_name: None,
            description: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("developers"));
    }

    #[test]
    fn test_create_group_request_full() {
        let req = CreateGroupRequest {
            name: "developers".to_string(),
            display_name: Some("Development Team".to_string()),
            description: Some("All developers".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Development Team"));
        assert!(json.contains("All developers"));
    }

    // ========================================================================
    // GroupResponse Tests
    // ========================================================================

    #[test]
    fn test_group_response_serialization() {
        let resp = GroupResponse {
            id: "group123".to_string(),
            name: "admins".to_string(),
            display_name: Some("Administrators".to_string()),
            description: None,
            roles: vec!["admin".to_string(), "editor".to_string()],
            users: vec!["alice@example.com".to_string()],
            created_at: 1704067200000000,
            updated_at: 1704067200000000,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("group123"));
        assert!(json.contains("admins"));
        assert!(json.contains("Administrators"));
    }

    // ========================================================================
    // UserRoleOption Tests
    // ========================================================================

    #[test]
    fn test_user_role_option() {
        let option = UserRoleOption {
            label: "Administrator".to_string(),
            value: "admin".to_string(),
        };

        let json = serde_json::to_string(&option).unwrap();
        assert!(json.contains("Administrator"));
        assert!(json.contains("admin"));
    }

    // ========================================================================
    // Resource Tests
    // ========================================================================

    #[test]
    fn test_resource_serialization_display_name() {
        let resource = Resource {
            key: "logs".to_string(),
            label: "Logs".to_string(),
            parent: Some("stream".to_string()),
            order: 11,
            visible: true,
            top_level: false,
            has_entities: true,
        };

        let json = serde_json::to_string(&resource).unwrap();
        // label should be serialized as display_name
        assert!(json.contains("\"display_name\":\"Logs\""));
        assert!(!json.contains("\"label\""));
    }
}
