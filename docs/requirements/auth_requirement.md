# OpenObserve Visdata 认证与授权需求规格说明书

## 文档信息

| 项目 | 内容 |
|------|------|
| 文档版本 | v1.0 |
| 创建日期 | 2024-12-31 |
| 项目名称 | OpenObserve Visdata Edition |
| 模块名称 | 认证与授权模块 (Dex + OpenFGA) |
| 状态 | 初稿 |

## 修订历史

| 版本 | 日期 | 作者 | 修改内容 |
|------|------|------|----------|
| v1.0 | 2024-12-31 | | 初始版本，基于 OpenObserve 开源版和企业版分析 |

---

## 1. 概述

### 1.1 目的

本文档定义 OpenObserve Visdata 版本的认证与授权功能需求规格。Visdata 版本采用 **Dex** 作为身份认证服务和 **OpenFGA** 作为细粒度授权引擎，替代 OpenObserve 开源版的简单 RBAC 模型，提供企业级的身份管理和访问控制能力。

### 1.2 背景

#### 1.2.1 OpenObserve 开源版现状

OpenObserve 开源版提供基础的认证授权功能：

| 功能 | 实现方式 |
|------|----------|
| 认证方式 | Basic Auth (用户名:密码 Base64编码) |
| 密码存储 | Argon2d 哈希 (v=16, m=2048, t=4, p=2) |
| 会话管理 | JWT Token + 数据库/内存缓存 |
| 权限模型 | 简单 RBAC (6种固定角色) |
| 权限检查 | 总是允许 (无细粒度控制) |

**开源版角色定义** (`src/config/src/meta/user.rs`):
```rust
pub enum UserRole {
    Root = 0,           // 超级管理员
    Admin = 1,          // 组织管理员
    Editor = 2,         // 编辑者
    Viewer = 3,         // 只读查看者
    User = 4,           // 无访问权限用户
    ServiceAccount = 5, // 服务账户
}
```

#### 1.2.2 OpenObserve 企业版增强

企业版在开源版基础上增加了：

- **Dex 集成**: 支持 OIDC/SSO 单点登录
- **LDAP 集成**: 支持企业 LDAP/AD 群组映射
- **OpenFGA**: 细粒度权限控制 (基于 Google Zanzibar)
- **自定义角色**: 支持创建自定义角色和权限
- **资源级权限**: 支持对单个资源的访问控制

#### 1.2.3 Visdata 版本定位

Visdata 版本是 OpenObserve 的独立企业发行版，目标是：

1. **完全兼容**: 与 OpenObserve API 保持兼容
2. **独立部署**: 使用 Dex + OpenFGA 作为独立服务
3. **企业级安全**: 提供完整的 SSO、LDAP、细粒度权限支持
4. **可扩展性**: 支持自定义身份提供商和权限模型

### 1.3 范围

本文档覆盖以下功能范围：

**包含：**
- 基于 Dex 的身份认证功能
- 基于 OpenFGA 的授权功能
- 用户、角色、组管理
- 与 OpenObserve 核心功能的集成

**不包含：**
- OpenObserve 核心数据处理功能
- 前端 UI 实现细节
- 部署运维相关内容

### 1.4 术语与定义

| 术语 | 定义 |
|------|------|
| **Dex** | 基于 OpenID Connect 的身份认证服务，支持多种身份提供商 |
| **OpenFGA** | 细粒度授权引擎，基于 Google Zanzibar 论文设计，支持 ReBAC |
| **OIDC** | OpenID Connect，基于 OAuth 2.0 的身份认证协议 |
| **RBAC** | Role-Based Access Control，基于角色的访问控制 |
| **ReBAC** | Relationship-Based Access Control，基于关系的访问控制 |
| **IdP** | Identity Provider，身份提供商（如 LDAP、Google、GitHub） |
| **Tuple** | OpenFGA 中的权限三元组 (user, relation, object) |
| **Store** | OpenFGA 中的授权数据存储单元 |
| **Authorization Model** | OpenFGA 中定义资源类型和关系的模型 |
| **PKCE** | Proof Key for Code Exchange，OAuth 2.0 安全扩展 |
| **JWT** | JSON Web Token，用于安全传输声明的紧凑令牌格式 |
| **JWKS** | JSON Web Key Set，用于验证 JWT 签名的公钥集合 |

### 1.5 参考文档

| 文档 | 链接 |
|------|------|
| Dex 官方文档 | https://dexidp.io/docs/ |
| OpenFGA 官方文档 | https://openfga.dev/docs |
| OpenObserve 文档 | https://openobserve.ai/docs/ |
| Google Zanzibar 论文 | https://research.google/pubs/pub48190/ |
| OAuth 2.0 规范 | https://oauth.net/2/ |
| OpenID Connect 规范 | https://openid.net/connect/ |

---

## 2. 项目干系人

### 2.1 干系人列表

| 角色 | 职责 | 关注点 |
|------|------|--------|
| 产品负责人 | 定义产品需求和优先级 | 功能完整性、市场竞争力 |
| 架构师 | 设计系统架构 | 可扩展性、性能、安全性 |
| 开发团队 | 实现功能 | 技术可行性、代码质量 |
| 运维团队 | 部署和维护系统 | 部署便利性、监控、故障恢复 |
| 安全团队 | 审核安全设计 | 安全合规、漏洞防护 |
| 最终用户 | 使用系统 | 易用性、功能满足度 |
| 企业 IT 管理员 | 管理用户和权限 | 集成便利性、管理效率 |

### 2.2 用户角色定义

#### 2.2.1 系统预定义角色

| 用户角色 | 描述 | 权限概述 |
|----------|------|----------|
| **Root** | 超级管理员 | 所有组织、所有资源的完全控制权限，系统级配置管理 |
| **Admin** | 组织管理员 | 管理所属组织的用户、角色、资源，但不能跨组织操作 |
| **Editor** | 编辑者 | 创建、修改、删除组织内的数据流、仪表盘、告警等资源 |
| **Viewer** | 查看者 | 只读访问组织内的资源，不能创建或修改 |
| **User** | 普通用户 | 基础访问权限，需要通过自定义角色或组获得具体权限 |
| **ServiceAccount** | 服务账户 | 用于 API 集成，支持程序化访问 |

#### 2.2.2 自定义角色支持

Visdata 版本支持创建自定义角色，可以：
- 组合多种资源权限
- 绑定到特定资源实例
- 分配给用户或用户组

---

## 3. 功能需求

### 3.1 认证功能（Dex）

#### 3.1.1 用户登录

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTH-001 | 支持本地用户名/密码登录 | P0 | 通过 Dex 的 Local Connector 实现，密码使用 Argon2d 哈希存储 |
| AUTH-002 | 支持 OIDC 第三方登录 | P0 | 支持任何符合 OIDC 规范的身份提供商 |
| AUTH-003 | 支持 LDAP/AD 集成 | P1 | 支持 LDAP 群组到组织/角色的映射 |
| AUTH-004 | 支持 SAML 2.0 集成 | P1 | 企业 SSO 标准协议支持 |
| AUTH-005 | 支持登录页面定制 | P2 | 支持自定义 Logo、主题色等 |
| AUTH-006 | 支持 Root-only 登录模式 | P1 | 可配置仅允许 Root 用户通过本地方式登录 |
| AUTH-007 | 支持禁用本地登录 | P1 | 强制所有用户使用 SSO 登录 |

**登录流程：**

```
┌─────────┐      ┌─────────┐      ┌─────────┐      ┌─────────────┐
│  用户   │      │ Visdata │      │   Dex   │      │ IdP (LDAP/  │
│ 浏览器  │      │ Backend │      │         │      │ Google/...) │
└────┬────┘      └────┬────┘      └────┬────┘      └──────┬──────┘
     │                │                │                   │
     │ 1. 访问登录页   │                │                   │
     │───────────────>│                │                   │
     │                │                │                   │
     │ 2. 重定向到 Dex │                │                   │
     │<───────────────│                │                   │
     │                │                │                   │
     │ 3. 请求认证     │                │                   │
     │────────────────────────────────>│                   │
     │                │                │                   │
     │ 4. 选择 IdP     │                │                   │
     │<────────────────────────────────│                   │
     │                │                │                   │
     │ 5. 重定向到 IdP │                │                   │
     │─────────────────────────────────────────────────────>
     │                │                │                   │
     │ 6. 用户认证     │                │                   │
     │<─────────────────────────────────────────────────────
     │                │                │                   │
     │ 7. 返回授权码   │                │                   │
     │────────────────────────────────>│                   │
     │                │                │                   │
     │ 8. 回调 Visdata │                │                   │
     │<────────────────────────────────│                   │
     │                │                │                   │
     │ 9. 交换 Token   │                │                   │
     │───────────────>│                │                   │
     │                │ 10. 验证授权码  │                   │
     │                │───────────────>│                   │
     │                │                │                   │
     │                │ 11. 返回 JWT   │                   │
     │                │<───────────────│                   │
     │                │                │                   │
     │ 12. 设置 Cookie │                │                   │
     │<───────────────│                │                   │
     │                │                │                   │
     │ 13. 登录成功    │                │                   │
     │<───────────────│                │                   │
```

#### 3.1.2 用户登出

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTH-010 | 支持单点登出 | P0 | 清除本地会话和 Dex 会话 |
| AUTH-011 | 支持会话超时自动登出 | P0 | 可配置会话超时时间 |
| AUTH-012 | 支持强制登出指定用户 | P1 | 管理员可以强制登出任意用户 |
| AUTH-013 | 支持登出所有设备 | P2 | 用户可以登出自己在所有设备的会话 |

#### 3.1.3 Token 管理

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTH-020 | 支持 JWT Access Token | P0 | 用于 API 认证，包含用户身份和基本声明 |
| AUTH-021 | 支持 Refresh Token | P0 | 用于静默刷新 Access Token |
| AUTH-022 | 支持 Token 自动刷新 | P0 | Access Token 过期前自动使用 Refresh Token 刷新 |
| AUTH-023 | 支持 Token 撤销 | P1 | 支持主动撤销 Refresh Token |
| AUTH-024 | 支持 PKCE 安全扩展 | P0 | 防止授权码拦截攻击 |
| AUTH-025 | 支持 Token 有效期配置 | P1 | 可配置 Access Token 和 Refresh Token 的有效期 |

**Token 结构：**

```json
{
  "iss": "https://dex.example.com",
  "sub": "user-unique-id",
  "aud": "openobserve",
  "exp": 1735689600,
  "iat": 1735686000,
  "email": "user@example.com",
  "email_verified": true,
  "name": "User Name",
  "groups": ["org1-admin", "org2-viewer"]
}
```

#### 3.1.4 身份提供商集成

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTH-030 | 支持 GitHub 登录 | P1 | OAuth 2.0 集成 |
| AUTH-031 | 支持 Google 登录 | P1 | OIDC 集成 |
| AUTH-032 | 支持 Microsoft Azure AD | P1 | OIDC/SAML 集成 |
| AUTH-033 | 支持企业 LDAP/AD | P0 | 支持 LDAPS，群组同步 |
| AUTH-034 | 支持 Okta | P1 | OIDC/SAML 集成 |
| AUTH-035 | 支持 KeyCloak | P2 | OIDC 集成 |
| AUTH-036 | 支持自定义 OIDC 提供商 | P1 | 任何符合 OIDC 规范的 IdP |
| AUTH-037 | 支持多 IdP 同时启用 | P1 | 用户可选择登录方式 |

**LDAP 配置示例：**

```yaml
connectors:
  - type: ldap
    id: ldap
    name: LDAP
    config:
      host: ldap.example.com:636
      insecureNoSSL: false
      insecureSkipVerify: false
      bindDN: cn=admin,dc=example,dc=com
      bindPW: admin-password
      userSearch:
        baseDN: ou=users,dc=example,dc=com
        filter: "(objectClass=person)"
        username: uid
        emailAttr: mail
        nameAttr: cn
      groupSearch:
        baseDN: ou=groups,dc=example,dc=com
        filter: "(objectClass=groupOfNames)"
        userMatchers:
          - userAttr: DN
            groupAttr: member
        nameAttr: cn
```

#### 3.1.5 用户同步与映射

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTH-040 | 支持自动创建用户 | P0 | 首次 SSO 登录自动创建本地用户记录 |
| AUTH-041 | 支持群组到组织映射 | P1 | IdP 群组自动映射到 OpenObserve 组织 |
| AUTH-042 | 支持群组到角色映射 | P1 | IdP 群组自动映射到用户角色 |
| AUTH-043 | 支持用户属性同步 | P2 | 同步 IdP 中的用户属性（姓名、邮箱等） |
| AUTH-044 | 支持默认组织配置 | P1 | 新用户自动加入默认组织 |
| AUTH-045 | 支持默认角色配置 | P1 | 新用户自动分配默认角色 |

**群组映射配置：**

```rust
pub struct DexConfig {
    pub group_claim: String,           // OIDC 群组声明名称，默认 "groups"
    pub group_attribute: String,       // LDAP 群组属性，用于组织映射
    pub role_attribute: String,        // LDAP 角色属性，用于角色映射
    pub default_org: String,           // 默认组织
    pub default_role: String,          // 默认角色
}
```

### 3.2 授权功能（OpenFGA）

#### 3.2.1 权限模型

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTHZ-001 | 支持组织级别权限控制 | P0 | 用户只能访问所属组织的资源 |
| AUTHZ-002 | 支持资源类型级别权限 | P0 | 按资源类型（流、仪表盘、告警等）控制访问 |
| AUTHZ-003 | 支持资源实例级别权限 | P0 | 控制对单个资源实例的访问 |
| AUTHZ-004 | 支持权限继承 | P0 | 组织权限自动继承到下级资源 |
| AUTHZ-005 | 支持权限否定 | P2 | 显式拒绝某些权限 |

**支持的资源类型（30+种）：**

| 资源类型 | 描述 | 支持的权限 |
|----------|------|------------|
| `org` | 组织 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `user` | 用户 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `group` | 用户组 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `role` | 角色 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `stream` | 数据流 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `logs` | 日志流 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `metrics` | 指标流 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `traces` | 追踪流 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `dashboard` | 仪表盘 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `dfolder` | 仪表盘文件夹 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `alert` | 告警规则 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `afolder` | 告警文件夹 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `destination` | 告警目标 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `template` | 告警模板 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `report` | 报告 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `rfolder` | 报告文件夹 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `function` | 函数 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `pipeline` | 数据管道 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `enrichment_table` | 富化表 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `savedviews` | 保存的视图 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `index` | 索引 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `metadata` | 元数据 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |
| `kv` | 键值存储 | AllowAll, AllowList, AllowGet, AllowPost, AllowPut, AllowDelete |

**权限类型定义：**

```rust
pub enum Permission {
    AllowAll,       // 全部权限
    AllowList,      // 列表权限 (GET 列表)
    AllowGet,       // 读权限 (GET 单个)
    AllowPost,      // 创建权限
    AllowPut,       // 更新权限
    AllowDelete,    // 删除权限
}
```

#### 3.2.2 角色管理

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTHZ-010 | 支持系统预定义角色 | P0 | Root, Admin, Editor, Viewer, User, ServiceAccount |
| AUTHZ-011 | 支持创建自定义角色 | P0 | 组织管理员可创建自定义角色 |
| AUTHZ-012 | 支持角色权限组合 | P0 | 自定义角色可组合多种资源权限 |
| AUTHZ-013 | 支持角色继承 | P1 | 角色可继承其他角色的权限 |
| AUTHZ-014 | 支持角色绑定到资源 | P0 | 角色可绑定到特定资源实例 |
| AUTHZ-015 | 支持查看角色权限列表 | P0 | 查看角色拥有的所有权限 |
| AUTHZ-016 | 支持角色使用情况统计 | P2 | 统计角色被分配的用户数 |

**自定义角色示例：**

```json
{
  "name": "dashboard-editor",
  "org_id": "org1",
  "permissions": [
    {"object": "dashboard:*", "permission": "AllowAll"},
    {"object": "dfolder:*", "permission": "AllowAll"},
    {"object": "stream:*", "permission": "AllowGet"},
    {"object": "stream:*", "permission": "AllowList"}
  ]
}
```

#### 3.2.3 用户组管理

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTHZ-020 | 支持创建用户组 | P0 | 组织管理员可创建用户组 |
| AUTHZ-021 | 支持用户组添加/移除成员 | P0 | 管理用户组成员 |
| AUTHZ-022 | 支持用户组分配角色 | P0 | 将角色分配给用户组 |
| AUTHZ-023 | 支持用户组权限继承 | P0 | 组成员自动获得组的权限 |
| AUTHZ-024 | 支持嵌套用户组 | P2 | 用户组可包含其他用户组 |
| AUTHZ-025 | 支持 IdP 群组同步 | P1 | 自动同步 LDAP/OIDC 群组 |

**用户组操作接口：**

```rust
pub struct GroupRequest {
    pub add_users: Option<HashSet<String>>,      // 添加用户
    pub remove_users: Option<HashSet<String>>,   // 移除用户
    pub add_roles: Option<HashSet<String>>,      // 添加角色
    pub remove_roles: Option<HashSet<String>>,   // 移除角色
}
```

#### 3.2.4 权限检查

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTHZ-030 | 支持实时权限检查 | P0 | 每次 API 请求检查权限 |
| AUTHZ-031 | 支持批量权限检查 | P1 | 一次请求检查多个权限 |
| AUTHZ-032 | 支持列出用户权限 | P0 | 列出用户对某类资源的所有权限 |
| AUTHZ-033 | 支持列出资源的授权用户 | P1 | 列出对某资源有访问权限的所有用户 |
| AUTHZ-034 | 支持权限检查缓存 | P0 | 缓存权限检查结果，提高性能 |
| AUTHZ-035 | 支持仅返回有权限的资源 | P1 | 列表 API 仅返回用户有权限访问的资源 |

**权限检查流程：**

```
┌─────────┐      ┌─────────────┐      ┌──────────┐      ┌──────────┐
│  用户   │      │   Visdata   │      │  Cache   │      │ OpenFGA  │
│ 请求    │      │   Backend   │      │          │      │          │
└────┬────┘      └──────┬──────┘      └────┬─────┘      └────┬─────┘
     │                  │                  │                 │
     │ 1. API 请求      │                  │                 │
     │─────────────────>│                  │                 │
     │                  │                  │                 │
     │                  │ 2. 检查缓存       │                 │
     │                  │─────────────────>│                 │
     │                  │                  │                 │
     │                  │ 3a. 缓存命中      │                 │
     │                  │<─────────────────│                 │
     │                  │                  │                 │
     │                  │ 3b. 缓存未命中，查询 OpenFGA        │
     │                  │─────────────────────────────────────>
     │                  │                  │                 │
     │                  │ 4. 返回权限结果   │                 │
     │                  │<─────────────────────────────────────
     │                  │                  │                 │
     │                  │ 5. 更新缓存       │                 │
     │                  │─────────────────>│                 │
     │                  │                  │                 │
     │ 6. 返回响应      │                  │                 │
     │<─────────────────│                  │                 │
```

**权限检查代码示例：**

```rust
pub async fn is_allowed(
    org_id: &str,
    user_id: &str,
    method: &str,        // GET, POST, PUT, DELETE, LIST
    object: &str,        // 格式: "resource_type:entity_id"
    parent_id: &str,
    role: &str,
) -> Result<bool> {
    // 1. Root 用户跳过检查
    if is_root_user(user_id) {
        return Ok(true);
    }

    // 2. 检查缓存
    if let Some(result) = cache.get(&cache_key) {
        return Ok(result);
    }

    // 3. 调用 OpenFGA 检查
    let result = openfga_client.check(TupleKey {
        user: format!("user:{}", user_id),
        relation: method_to_relation(method),
        object: object.to_string(),
    }).await?;

    // 4. 更新缓存
    cache.insert(cache_key, result, ttl);

    Ok(result)
}
```

#### 3.2.5 权限管理

| 需求ID | 需求描述 | 优先级 | 详细说明 |
|--------|----------|--------|----------|
| AUTHZ-040 | 支持授予用户权限 | P0 | 直接授予用户对资源的权限 |
| AUTHZ-041 | 支持撤销用户权限 | P0 | 撤销用户对资源的权限 |
| AUTHZ-042 | 支持授予用户组权限 | P0 | 授予用户组对资源的权限 |
| AUTHZ-043 | 支持撤销用户组权限 | P0 | 撤销用户组对资源的权限 |
| AUTHZ-044 | 支持批量权限操作 | P1 | 批量授予/撤销权限 |
| AUTHZ-045 | 支持权限变更审计 | P0 | 记录所有权限变更操作 |

**权限操作接口：**

```rust
pub struct RoleRequest {
    pub add: Vec<O2EntityAuthorization>,      // 添加权限
    pub remove: Vec<O2EntityAuthorization>,   // 移除权限
    pub add_users: Option<HashSet<String>>,   // 添加用户
    pub remove_users: Option<HashSet<String>>, // 移除用户
}

pub struct O2EntityAuthorization {
    pub object: String,        // 格式: "resource_type:entity_id"
    pub permission: Permission,
}
```

---

## 4. 非功能需求

### 4.1 性能需求

| 需求ID | 需求描述 | 指标 | 测量方法 |
|--------|----------|------|----------|
| NFR-001 | 认证响应时间 | P95 < 500ms | 压力测试 |
| NFR-002 | Token 刷新响应时间 | P95 < 200ms | 压力测试 |
| NFR-003 | 权限检查响应时间 | P95 < 50ms | 压力测试 |
| NFR-004 | 权限检查缓存命中率 | > 90% | 监控统计 |
| NFR-005 | 并发认证请求 | > 1000 QPS | 压力测试 |
| NFR-006 | 并发权限检查请求 | > 10000 QPS | 压力测试 |

**缓存配置：**

```rust
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,    // 默认 300 (5分钟)
    pub max_entries: usize,  // 默认 10000
}
```

### 4.2 安全需求

| 需求ID | 需求描述 | 优先级 | 实现方式 |
|--------|----------|--------|----------|
| NFR-010 | 密码安全存储 | P0 | Argon2d 哈希，独立 salt |
| NFR-011 | Token 安全传输 | P0 | HTTPS/TLS 1.2+ |
| NFR-012 | Token 安全存储 | P0 | HttpOnly + Secure Cookie |
| NFR-013 | 防止暴力破解 | P0 | 登录失败限速，账户锁定 |
| NFR-014 | 防止 CSRF 攻击 | P0 | PKCE，SameSite Cookie |
| NFR-015 | 防止 XSS 攻击 | P0 | Token 不存储在 localStorage |
| NFR-016 | 审计日志 | P0 | 记录所有认证和授权操作 |
| NFR-017 | 敏感数据加密 | P1 | AES-GCM 加密敏感配置 |
| NFR-018 | 密钥轮换 | P2 | 支持 JWT 签名密钥轮换 |

**密码哈希参数：**

```rust
// Argon2d 配置
// v=16, m=2048, t=4, p=2
let hash = argon2d::hash(password, salt, &Config {
    variant: Variant::Argon2d,
    version: Version::Version16,
    mem_cost: 2048,
    time_cost: 4,
    lanes: 2,
    ..Default::default()
});
```

### 4.3 可用性需求

| 需求ID | 需求描述 | 指标 | 说明 |
|--------|----------|------|------|
| NFR-020 | 系统可用性 | 99.9% | 年停机时间 < 8.76 小时 |
| NFR-021 | Dex 高可用 | 支持多实例部署 | 无单点故障 |
| NFR-022 | OpenFGA 高可用 | 支持多实例部署 | 无单点故障 |
| NFR-023 | 优雅降级 | 认证/授权服务不可用时的处理策略 | 可配置 |
| NFR-024 | 故障恢复时间 | < 5 分钟 | 自动恢复 |

### 4.4 可扩展性需求

| 需求ID | 需求描述 | 优先级 | 说明 |
|--------|----------|--------|------|
| NFR-030 | 水平扩展 | P0 | Dex 和 OpenFGA 支持多实例 |
| NFR-031 | 多租户支持 | P0 | 支持多组织隔离 |
| NFR-032 | 用户规模 | P0 | 支持 100,000+ 用户 |
| NFR-033 | 权限规模 | P0 | 支持 1,000,000+ 权限元组 |
| NFR-034 | 资源类型扩展 | P1 | 支持添加新的资源类型 |

### 4.5 兼容性需求

| 需求ID | 需求描述 | 优先级 | 说明 |
|--------|----------|--------|------|
| NFR-040 | OpenObserve API 兼容 | P0 | 保持与开源版 API 完全兼容 |
| NFR-041 | 开源版迁移 | P1 | 支持从开源版平滑迁移 |
| NFR-042 | 企业版兼容 | P1 | 与官方企业版 API 兼容 |
| NFR-043 | 浏览器兼容 | P0 | Chrome, Firefox, Safari, Edge 最新版 |

---

## 5. 外部接口需求

### 5.1 Dex 接口

#### 5.1.1 OIDC 标准端点

| 端点 | 方法 | 描述 | 协议 |
|------|------|------|------|
| `/.well-known/openid-configuration` | GET | OIDC 发现端点 | OIDC |
| `/auth` | GET | 授权端点，发起认证流程 | OAuth 2.0 |
| `/token` | POST | Token 端点，交换授权码 | OAuth 2.0 |
| `/userinfo` | GET | 用户信息端点 | OIDC |
| `/keys` | GET | JWKS 端点，获取签名公钥 | OIDC |

#### 5.1.2 Dex 管理接口 (gRPC/HTTP)

| 接口 | 描述 | 用途 |
|------|------|------|
| `CreatePassword` | 创建本地用户密码 | 用户注册 |
| `UpdatePassword` | 更新用户密码 | 密码修改 |
| `DeletePassword` | 删除用户密码 | 用户删除 |
| `VerifyPassword` | 验证用户密码 | 本地登录验证 |
| `ListPasswords` | 列出所有密码 | 用户管理 |
| `CreateConnector` | 创建 IdP 连接器 | IdP 配置 |
| `UpdateConnector` | 更新 IdP 连接器 | IdP 配置 |
| `DeleteConnector` | 删除 IdP 连接器 | IdP 配置 |
| `ListConnectors` | 列出所有连接器 | IdP 管理 |
| `ListRefresh` | 列出刷新令牌 | 会话管理 |
| `RevokeRefresh` | 撤销刷新令牌 | 会话管理 |
| `GetVersion` | 获取版本信息 | 健康检查 |

**Dex 配置：**

```rust
pub struct DexConfig {
    pub grpc_url: String,              // gRPC API URL (默认: http://localhost:5557)
    pub client_id: String,              // OAuth2 Client ID
    pub client_secret: String,          // OAuth2 Client Secret
    pub issuer_url: String,             // OIDC Issuer URL (默认: http://localhost:5556)
    pub redirect_uri: String,           // OAuth2 Redirect URI
    pub default_org: String,            // 默认组织
    pub default_role: String,           // 默认角色
    pub native_login_enabled: bool,     // 启用本地登录
    pub root_only_login: bool,          // 仅允许 Root 用户登录
    pub group_claim: String,            // OIDC 群组声明名称
    pub group_attribute: String,        // LDAP 群组属性
    pub role_attribute: String,         // LDAP 角色属性
    pub scopes: Vec<String>,            // OIDC scopes
    pub timeout_seconds: u64,           // 连接超时
}
```

### 5.2 OpenFGA 接口

#### 5.2.1 核心 API

| 端点 | 方法 | 描述 | 用途 |
|------|------|------|------|
| `/stores` | POST | 创建 Store | 初始化 |
| `/stores` | GET | 列出 Store | 管理 |
| `/stores/{store_id}` | GET | 获取 Store | 管理 |
| `/stores/{store_id}` | DELETE | 删除 Store | 管理 |
| `/stores/{store_id}/authorization-models` | POST | 写入授权模型 | 初始化 |
| `/stores/{store_id}/authorization-models` | GET | 列出授权模型 | 管理 |
| `/stores/{store_id}/check` | POST | 权限检查 | 核心 |
| `/stores/{store_id}/write` | POST | 写入元组 | 权限管理 |
| `/stores/{store_id}/read` | POST | 读取元组 | 权限查询 |
| `/stores/{store_id}/expand` | POST | 展开关系 | 调试 |
| `/stores/{store_id}/list-objects` | POST | 列出可访问对象 | 资源过滤 |

**OpenFGA 配置：**

```rust
pub struct OpenFGAConfig {
    pub api_url: String,               // HTTP API URL (默认: http://localhost:8080)
    pub store_id: String,              // Store ID (自动创建)
    pub model_id: Option<String>,      // Authorization Model ID
    pub store_name: String,            // Store name (默认: openobserve)
    pub enabled: bool,                 // 启用权限检查
    pub list_only_permitted: bool,     // 列表仅返回有权限的资源
    pub timeout_seconds: u64,          // 请求超时时间
}
```

#### 5.2.2 权限检查请求示例

```json
// POST /stores/{store_id}/check
{
  "tuple_key": {
    "user": "user:alice@example.com",
    "relation": "viewer",
    "object": "dashboard:dashboard-123"
  }
}

// 响应
{
  "allowed": true
}
```

#### 5.2.3 写入权限元组示例

```json
// POST /stores/{store_id}/write
{
  "writes": {
    "tuple_keys": [
      {
        "user": "user:alice@example.com",
        "relation": "editor",
        "object": "dashboard:dashboard-123"
      },
      {
        "user": "group:data-team#member",
        "relation": "viewer",
        "object": "stream:logs-production"
      }
    ]
  }
}
```

### 5.3 Visdata 认证授权 API

#### 5.3.1 认证 API

| 端点 | 方法 | 描述 | 请求体 |
|------|------|------|--------|
| `/auth/login` | GET | 获取登录状态/发起 SSO | - |
| `/auth/login` | POST | 本地登录 | `{email, password}` |
| `/auth/refresh` | POST | 刷新 Token | `{refresh_token}` |
| `/auth/logout` | POST | 登出 | - |
| `/config/redirect` | GET | SSO 回调 | - |

#### 5.3.2 用户管理 API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/{org_id}/users` | GET | 列出组织用户 |
| `/api/{org_id}/users` | POST | 创建用户 |
| `/api/{org_id}/users/{user_id}` | GET | 获取用户详情 |
| `/api/{org_id}/users/{user_id}` | PUT | 更新用户 |
| `/api/{org_id}/users/{user_id}` | DELETE | 删除用户 |

#### 5.3.3 角色管理 API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/{org_id}/roles` | GET | 列出角色 |
| `/api/{org_id}/roles` | POST | 创建角色 |
| `/api/{org_id}/roles/{role_name}` | GET | 获取角色详情 |
| `/api/{org_id}/roles/{role_name}` | PUT | 更新角色 |
| `/api/{org_id}/roles/{role_name}` | DELETE | 删除角色 |

#### 5.3.4 用户组管理 API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/{org_id}/groups` | GET | 列出用户组 |
| `/api/{org_id}/groups` | POST | 创建用户组 |
| `/api/{org_id}/groups/{group_name}` | GET | 获取用户组详情 |
| `/api/{org_id}/groups/{group_name}` | PUT | 更新用户组 |
| `/api/{org_id}/groups/{group_name}` | DELETE | 删除用户组 |

---

## 6. 数据需求

### 6.1 数据模型

#### 6.1.1 用户数据

**用户表 (users)：**

| 字段 | 类型 | 描述 | 约束 |
|------|------|------|------|
| id | String | 用户 ID (邮箱) | 主键 |
| email | String | 邮箱地址 | 唯一 |
| first_name | String | 名 | |
| last_name | String | 姓 | |
| password | String | 密码哈希 | |
| salt | String | 密码盐值 | |
| is_root | Boolean | 是否为超级管理员 | |
| password_ext | String | 外部认证密码 | 可空 |
| user_type | Integer | 用户类型 | |
| created_at | Timestamp | 创建时间 | |
| updated_at | Timestamp | 更新时间 | |

**组织用户关联表 (org_users)：**

| 字段 | 类型 | 描述 | 约束 |
|------|------|------|------|
| id | String | 记录 ID | 主键 |
| email | String | 用户邮箱 | 外键 |
| org_id | String | 组织 ID | |
| role | Integer | 用户角色 | |
| token | String | 组织级 Token | |
| rum_token | String | RUM Token | 可空 |
| created_at | Timestamp | 创建时间 | |
| updated_at | Timestamp | 更新时间 | |

**会话表 (sessions)：**

| 字段 | 类型 | 描述 | 约束 |
|------|------|------|------|
| session_id | String | 会话 ID | 主键 |
| access_token | String | JWT Token | |
| created_at | Timestamp | 创建时间 | |
| updated_at | Timestamp | 更新时间 | |

#### 6.1.2 OpenFGA 授权模型

**授权模型定义 (authorization_model.json)：**

```dsl
model
  schema 1.1

# 用户类型
type user

# 用户组类型
type group
  relations
    define member: [user, group#member]

# 角色类型
type role
  relations
    define assignee: [user, group#member]

# 组织类型
type org
  relations
    define owner: [user]
    define admin: [user, group#member] or owner
    define member: [user, group#member] or admin
    define viewer: [user, group#member] or member

    # 权限定义
    define can_delete: owner
    define can_write: admin
    define can_read: viewer

# 数据流类型
type stream
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member] or admin from org
    define viewer: [user, group#member] or member from org

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

# 日志流类型
type logs
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member] or admin from org
    define viewer: [user, group#member] or member from org

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

# 仪表盘类型
type dashboard
  relations
    define org: [org]
    define folder: [dfolder]
    define owner: [user] or owner from org or owner from folder
    define editor: [user, group#member] or admin from org or editor from folder
    define viewer: [user, group#member] or member from org or viewer from folder

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

# 仪表盘文件夹类型
type dfolder
  relations
    define org: [org]
    define parent: [dfolder]
    define owner: [user] or owner from org or owner from parent
    define editor: [user, group#member] or admin from org or editor from parent
    define viewer: [user, group#member] or member from org or viewer from parent

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

# 告警类型
type alert
  relations
    define org: [org]
    define folder: [afolder]
    define owner: [user] or owner from org or owner from folder
    define editor: [user, group#member] or admin from org or editor from folder
    define viewer: [user, group#member] or member from org or viewer from folder

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

# ... 其他 30+ 资源类型定义
```

### 6.2 数据存储

| 数据类型 | 存储位置 | 备注 |
|----------|----------|------|
| 用户凭证 | Dex 后端存储 (PostgreSQL/MySQL/SQLite) | 密码使用 bcrypt 哈希 |
| 用户信息 | OpenObserve 数据库 | 用户详细信息 |
| 会话数据 | OpenObserve 数据库 + 内存缓存 | 支持集群同步 |
| 授权模型 | OpenFGA 存储 (PostgreSQL/MySQL) | 启动时自动初始化 |
| 授权元组 | OpenFGA 存储 | 权限关系数据 |
| 权限缓存 | 内存 (DashMap) | TTL 5分钟 |

### 6.3 数据迁移

#### 6.3.1 从开源版迁移

**用户数据迁移：**

1. 导出现有用户数据
2. 在 Dex 中创建对应的本地用户
3. 迁移用户到组织的关联关系
4. 在 OpenFGA 中创建初始权限元组

**权限数据迁移：**

| 开源版角色 | OpenFGA 权限 |
|------------|--------------|
| Root | `org:*#owner` |
| Admin | `org:{org_id}#admin` |
| Editor | `org:{org_id}#editor` |
| Viewer | `org:{org_id}#viewer` |
| User | 无默认权限，需单独授权 |

**迁移脚本示例：**

```rust
async fn migrate_user(user: &OldUser) -> Result<()> {
    // 1. 在 Dex 创建用户
    dex_client.create_password(&user.email, &user.password_hash).await?;

    // 2. 在 OpenFGA 创建权限
    let tuple = TupleKey {
        user: format!("user:{}", user.email),
        relation: role_to_relation(&user.role),
        object: format!("org:{}", user.org_id),
    };
    openfga_client.write(vec![tuple]).await?;

    Ok(())
}
```

---

## 7. 用例说明

### 7.1 用户认证用例

#### UC-001: 本地用户登录

- **参与者**: 用户
- **前置条件**:
  - 用户已在系统中注册
  - 本地登录已启用 (`native_login_enabled = true`)
- **主流程**:
  1. 用户访问登录页面
  2. 用户输入邮箱和密码
  3. 系统调用 Dex 验证密码
  4. Dex 验证成功，返回授权码
  5. 系统使用授权码交换 JWT Token
  6. 系统设置 Cookie，返回登录成功
- **异常流程**:
  - E1: 邮箱或密码错误 → 返回认证失败
  - E2: 账户被锁定 → 返回账户锁定提示
  - E3: 本地登录被禁用 → 重定向到 SSO
- **后置条件**: 用户获得有效会话，可访问系统资源

#### UC-002: SSO 登录 (OIDC)

- **参与者**: 用户
- **前置条件**:
  - 身份提供商已配置
  - 用户在 IdP 中存在
- **主流程**:
  1. 用户访问登录页面
  2. 用户选择 SSO 登录方式
  3. 系统重定向到 Dex 授权端点
  4. Dex 重定向到选择的 IdP
  5. 用户在 IdP 完成认证
  6. IdP 回调 Dex，返回用户信息
  7. Dex 回调 Visdata，返回授权码
  8. 系统交换 Token，解析用户信息
  9. 系统自动创建/更新本地用户
  10. 系统根据群组映射分配组织和角色
  11. 系统设置 Cookie，返回登录成功
- **异常流程**:
  - E1: IdP 认证失败 → 返回认证失败
  - E2: 用户不在允许的群组 → 返回访问被拒绝
  - E3: 群组映射失败 → 使用默认组织和角色
- **后置条件**: 用户获得有效会话，组织和角色已自动分配

#### UC-003: Token 刷新

- **参与者**: 系统 (自动)
- **前置条件**: 用户持有有效 Refresh Token
- **主流程**:
  1. Access Token 即将过期
  2. 前端检测到 Token 过期
  3. 前端使用 Refresh Token 请求新 Token
  4. 系统调用 Dex Token 端点
  5. Dex 验证 Refresh Token
  6. Dex 签发新的 Access Token 和 Refresh Token
  7. 系统更新 Cookie
- **异常流程**:
  - E1: Refresh Token 过期 → 重定向到登录页面
  - E2: Refresh Token 被撤销 → 重定向到登录页面
- **后置条件**: 用户获得新的有效 Token

#### UC-004: 用户登出

- **参与者**: 用户
- **前置条件**: 用户已登录
- **主流程**:
  1. 用户点击登出按钮
  2. 系统清除本地 Cookie
  3. 系统删除会话记录
  4. 系统调用 Dex 撤销 Refresh Token
  5. 系统重定向到登录页面
- **异常流程**:
  - E1: Dex 服务不可用 → 仅清除本地会话
- **后置条件**: 用户会话已结束

### 7.2 权限管理用例

#### UC-010: 权限检查

- **参与者**: 系统 (自动)
- **前置条件**: 用户已认证
- **主流程**:
  1. 用户发起 API 请求
  2. 系统提取用户身份和请求的资源
  3. 系统检查内存缓存是否有权限记录
  4. 缓存未命中，调用 OpenFGA Check API
  5. OpenFGA 返回权限检查结果
  6. 系统缓存结果
  7. 系统根据结果允许或拒绝请求
- **异常流程**:
  - E1: OpenFGA 服务不可用 → 根据配置拒绝或允许
  - E2: 用户是 Root → 跳过检查，直接允许
- **后置条件**: 请求被允许或拒绝

#### UC-011: 创建自定义角色

- **参与者**: 组织管理员
- **前置条件**:
  - 管理员已认证
  - 管理员有角色管理权限
- **主流程**:
  1. 管理员访问角色管理页面
  2. 管理员点击创建角色
  3. 管理员输入角色名称
  4. 管理员选择权限（资源类型 + 操作）
  5. 系统在 OpenFGA 创建角色
  6. 系统返回创建成功
- **异常流程**:
  - E1: 角色名已存在 → 返回重复错误
  - E2: 权限组合无效 → 返回验证错误
- **后置条件**: 新角色已创建，可分配给用户

#### UC-012: 授予用户权限

- **参与者**: 组织管理员
- **前置条件**:
  - 管理员已认证
  - 目标用户存在于组织中
- **主流程**:
  1. 管理员访问用户管理页面
  2. 管理员选择目标用户
  3. 管理员选择要授予的角色或权限
  4. 系统在 OpenFGA 写入权限元组
  5. 系统清除相关缓存
  6. 系统返回授权成功
- **异常流程**:
  - E1: 权限已存在 → 返回重复提示
  - E2: OpenFGA 写入失败 → 返回系统错误
- **后置条件**: 用户获得新权限，立即生效

#### UC-013: 列出有权限的资源

- **参与者**: 用户
- **前置条件**: 用户已认证
- **主流程**:
  1. 用户请求资源列表（如仪表盘列表）
  2. 系统调用 OpenFGA ListObjects API
  3. OpenFGA 返回用户有权限访问的资源 ID 列表
  4. 系统根据 ID 列表查询资源详情
  5. 系统返回过滤后的资源列表
- **异常流程**:
  - E1: `list_only_permitted = false` → 返回所有资源（不过滤）
- **后置条件**: 用户只看到有权限的资源

---

## 8. 验收标准

### 8.1 认证功能验收标准

| 标准ID | 验收条件 | 验证方法 | 优先级 |
|--------|----------|----------|--------|
| AC-001 | 用户可通过本地用户名/密码成功登录 | 功能测试 | P0 |
| AC-002 | 用户可通过 OIDC 第三方 IdP 登录 | 功能测试 | P0 |
| AC-003 | 用户可通过 LDAP 登录 | 功能测试 | P1 |
| AC-004 | Token 过期后自动刷新，用户无感知 | 功能测试 | P0 |
| AC-005 | 用户登出后无法使用旧 Token 访问 | 安全测试 | P0 |
| AC-006 | LDAP 群组正确映射到组织和角色 | 功能测试 | P1 |
| AC-007 | 首次 SSO 登录自动创建用户 | 功能测试 | P0 |
| AC-008 | 禁用本地登录后，只能通过 SSO 登录 | 功能测试 | P1 |
| AC-009 | 登录响应时间 P95 < 500ms | 性能测试 | P0 |
| AC-010 | 支持 1000+ 并发登录请求 | 压力测试 | P0 |

### 8.2 授权功能验收标准

| 标准ID | 验收条件 | 验证方法 | 优先级 |
|--------|----------|----------|--------|
| AC-020 | 权限检查响应时间 P95 < 50ms | 性能测试 | P0 |
| AC-021 | 无权限用户无法访问受保护资源 | 安全测试 | P0 |
| AC-022 | 权限变更实时生效（缓存刷新后） | 功能测试 | P0 |
| AC-023 | Root 用户可访问所有资源 | 功能测试 | P0 |
| AC-024 | 自定义角色权限正确生效 | 功能测试 | P0 |
| AC-025 | 用户组权限正确继承 | 功能测试 | P0 |
| AC-026 | 资源列表仅显示有权限的资源 | 功能测试 | P1 |
| AC-027 | 支持 10000+ 并发权限检查 | 压力测试 | P0 |
| AC-028 | 权限缓存命中率 > 90% | 性能测试 | P0 |
| AC-029 | 支持 100 万+ 权限元组 | 容量测试 | P0 |

### 8.3 集成验收标准

| 标准ID | 验收条件 | 验证方法 | 优先级 |
|--------|----------|----------|--------|
| AC-030 | 与 OpenObserve 开源版 API 100% 兼容 | 兼容性测试 | P0 |
| AC-031 | 从开源版迁移用户和权限无数据丢失 | 迁移测试 | P1 |
| AC-032 | Dex 和 OpenFGA 支持高可用部署 | 部署测试 | P0 |
| AC-033 | 系统可用性达到 99.9% | 可用性测试 | P0 |

---

## 9. 约束与假设

### 9.1 约束条件

| 约束 | 描述 | 影响 |
|------|------|------|
| 技术栈约束 | 必须使用 Rust 实现，与 OpenObserve 保持一致 | 开发语言限制 |
| 协议约束 | 必须支持 OIDC 和 OAuth 2.0 标准协议 | 认证协议限制 |
| 兼容性约束 | 必须与 OpenObserve API 保持兼容 | 接口设计限制 |
| 性能约束 | 权限检查不能显著增加 API 延迟 | 缓存设计要求 |
| 部署约束 | Dex 和 OpenFGA 作为独立服务部署 | 架构设计限制 |

### 9.2 假设条件

| 假设 | 描述 | 风险 |
|------|------|------|
| 网络假设 | Visdata、Dex、OpenFGA 之间网络延迟 < 10ms | 高延迟影响性能 |
| 规模假设 | 初期用户规模 < 10,000 | 超规模需重新评估 |
| IdP 假设 | 企业 IdP 支持标准 OIDC/LDAP 协议 | 非标准 IdP 需额外适配 |
| 数据假设 | 权限关系数据量 < 1,000,000 | 超规模需分片 |

### 9.3 依赖关系

| 依赖 | 版本 | 用途 | 备注 |
|------|------|------|------|
| Dex | >= 2.37 | 身份认证 | 支持 gRPC API |
| OpenFGA | >= 1.3 | 细粒度授权 | 支持 HTTP API |
| PostgreSQL | >= 14 | 数据存储 | Dex 和 OpenFGA 后端 |
| OpenObserve | >= 0.9 | 核心系统 | 集成基础 |

---

## 10. 附录

### 10.1 OpenFGA 授权模型完整示例

```dsl
model
  schema 1.1

type user

type group
  relations
    define member: [user, group#member]

type role
  relations
    define assignee: [user, group#member]

type org
  relations
    define owner: [user]
    define admin: [user, group#member, role#assignee] or owner
    define editor: [user, group#member, role#assignee] or admin
    define viewer: [user, group#member, role#assignee] or editor

    define can_manage_users: admin
    define can_manage_roles: owner
    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type stream
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer
    define can_ingest: editor

type logs
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer
    define can_query: viewer

type metrics
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type traces
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type dashboard
  relations
    define org: [org]
    define folder: [dfolder]
    define owner: [user] or owner from org or owner from folder
    define editor: [user, group#member, role#assignee] or admin from org or editor from folder or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or viewer from folder or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type dfolder
  relations
    define org: [org]
    define parent: [dfolder]
    define owner: [user] or owner from org or owner from parent
    define editor: [user, group#member, role#assignee] or admin from org or editor from parent or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or viewer from parent or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type alert
  relations
    define org: [org]
    define folder: [afolder]
    define owner: [user] or owner from org or owner from folder
    define editor: [user, group#member, role#assignee] or admin from org or editor from folder or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or viewer from folder or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer
    define can_trigger: editor

type afolder
  relations
    define org: [org]
    define parent: [afolder]
    define owner: [user] or owner from org or owner from parent
    define editor: [user, group#member, role#assignee] or admin from org or editor from parent or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or viewer from parent or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type destination
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type template
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type function
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer
    define can_execute: viewer

type pipeline
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer

type enrichment_table
  relations
    define org: [org]
    define owner: [user] or owner from org
    define editor: [user, group#member, role#assignee] or admin from org or owner
    define viewer: [user, group#member, role#assignee] or viewer from org or editor

    define can_delete: owner
    define can_write: editor
    define can_read: viewer
```

### 10.2 配置示例

#### Dex 配置 (dex-config.yaml)

```yaml
issuer: https://dex.example.com

storage:
  type: postgres
  config:
    host: postgres
    port: 5432
    database: dex
    user: dex
    password: ${DEX_DB_PASSWORD}
    ssl:
      mode: disable

web:
  http: 0.0.0.0:5556

grpc:
  addr: 0.0.0.0:5557

oauth2:
  skipApprovalScreen: true

staticClients:
  - id: openobserve
    redirectURIs:
      - 'https://openobserve.example.com/config/redirect'
    name: 'OpenObserve'
    secret: ${DEX_CLIENT_SECRET}

connectors:
  # LDAP 连接器
  - type: ldap
    id: ldap
    name: LDAP
    config:
      host: ldap.example.com:636
      insecureNoSSL: false
      bindDN: cn=admin,dc=example,dc=com
      bindPW: ${LDAP_BIND_PASSWORD}
      userSearch:
        baseDN: ou=users,dc=example,dc=com
        filter: "(objectClass=person)"
        username: uid
        emailAttr: mail
        nameAttr: cn
      groupSearch:
        baseDN: ou=groups,dc=example,dc=com
        filter: "(objectClass=groupOfNames)"
        userMatchers:
          - userAttr: DN
            groupAttr: member
        nameAttr: cn

  # GitHub 连接器
  - type: github
    id: github
    name: GitHub
    config:
      clientID: ${GITHUB_CLIENT_ID}
      clientSecret: ${GITHUB_CLIENT_SECRET}
      redirectURI: https://dex.example.com/callback
      orgs:
        - name: my-org

  # Google 连接器
  - type: oidc
    id: google
    name: Google
    config:
      issuer: https://accounts.google.com
      clientID: ${GOOGLE_CLIENT_ID}
      clientSecret: ${GOOGLE_CLIENT_SECRET}
      redirectURI: https://dex.example.com/callback
```

#### Visdata 配置 (visdata-config.toml)

```toml
# RBAC 和 SSO 开关
rbac_enabled = true
sso_enabled = true

# 加密密钥
encryption_key = "your-32-byte-encryption-key-here"

# OpenFGA 配置
openfga_url = "http://openfga:8080"
openfga_store_name = "openobserve"

# Dex 配置
dex_grpc_url = "http://dex:5557"
dex_issuer_url = "https://dex.example.com"
dex_client_id = "openobserve"
dex_client_secret = "your-client-secret"
dex_redirect_uri = "https://openobserve.example.com/config/redirect"

# 默认组织和角色
dex_default_org = "default"
dex_default_role = "viewer"

# 本地登录控制
dex_native_login_enabled = true
dex_root_only_login = false

# 群组映射
dex_group_claim = "groups"
dex_group_attribute = "ou"
dex_role_attribute = "role"

# 缓存配置
[cache]
enabled = true
ttl_seconds = 300
max_entries = 10000
```

### 10.3 API 示例

#### 本地登录

```bash
# POST /auth/login
curl -X POST https://openobserve.example.com/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'

# 响应
{
  "status": "success",
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "dex_refresh_token_...",
  "expires_in": 3600
}
```

#### 创建自定义角色

```bash
# POST /api/{org_id}/roles
curl -X POST https://openobserve.example.com/api/org1/roles \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "dashboard-editor",
    "permissions": [
      {"object": "dashboard:*", "permission": "AllowAll"},
      {"object": "dfolder:*", "permission": "AllowAll"},
      {"object": "stream:*", "permission": "AllowGet"}
    ]
  }'

# 响应
{
  "status": "success",
  "message": "Role created successfully"
}
```

#### 授予用户角色

```bash
# PUT /api/{org_id}/roles/{role_name}
curl -X PUT https://openobserve.example.com/api/org1/roles/dashboard-editor \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "add_users": ["alice@example.com", "bob@example.com"]
  }'

# 响应
{
  "status": "success",
  "message": "Role updated successfully"
}
```

### 10.4 待定事项

| 事项 | 描述 | 负责人 | 状态 |
|------|------|--------|------|
| SAML 2.0 支持 | 企业 SAML SSO 集成 | | 待定 |
| 多因素认证 (MFA) | 支持 TOTP/WebAuthn | | 待定 |
| 审计日志 | 认证授权操作审计 | | 待定 |
| 权限模型版本控制 | 支持授权模型版本管理 | | 待定 |
| 权限模拟 | 管理员模拟用户权限测试 | | 待定 |

---

## 文档结束
