# OpenObserve Visdata 认证与授权设计说明书

## 文档信息

| 项目 | 内容 |
|------|------|
| 文档版本 | v1.0 |
| 创建日期 | 2024-12-31 |
| 项目名称 | OpenObserve Visdata Edition |
| 模块名称 | 认证与授权模块 |
| 状态 | 初稿 |

## 修订历史

| 版本 | 日期 | 作者 | 修改内容 |
|------|------|------|----------|
| v1.0 | 2024-12-31 | | 初始版本 |

---

## 1. 引言

### 1.1 目的

本文档描述 OpenObserve Visdata 版本认证与授权模块的详细设计，包括系统架构、模块设计、接口设计、数据设计等内容，为开发团队提供实现指导。

### 1.2 范围

本设计文档覆盖：
- 基于 Dex 的身份认证模块设计
- 基于 OpenFGA 的细粒度授权模块设计
- 与 OpenObserve 核心系统的集成设计

### 1.3 读者对象

| 读者 | 关注内容 |
|------|----------|
| 架构师 | 整体架构、技术选型、接口设计 |
| 开发人员 | 详细设计、接口规格、数据模型 |
| 测试人员 | 测试策略、接口规格 |
| 运维人员 | 部署设计、配置管理 |

### 1.4 参考文档

| 文档 | 说明 |
|------|------|
| OpenObserve Visdata 认证与授权需求规格说明书 | 需求来源 |
| Dex 官方文档 (https://dexidp.io/docs/) | Dex 技术参考 |
| OpenFGA 官方文档 (https://openfga.dev/docs) | OpenFGA 技术参考 |
| Google Zanzibar 论文 | 授权模型理论基础 |

### 1.5 术语与缩写

| 术语 | 定义 |
|------|------|
| OIDC | OpenID Connect，基于 OAuth 2.0 的身份认证协议 |
| IdP | Identity Provider，身份提供商 |
| Tuple | OpenFGA 权限三元组 (user, relation, object) |
| Store | OpenFGA 授权数据存储单元 |
| Connector | Dex 中连接外部 IdP 的适配器 |
| PKCE | Proof Key for Code Exchange，OAuth 2.0 安全扩展 |

---

## 2. 系统概述

### 2.1 系统背景

#### 2.1.1 OpenObserve 开源版认证授权现状

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenObserve 开源版                        │
├─────────────────────────────────────────────────────────────┤
│  认证方式: Basic Auth (用户名:密码 Base64)                    │
│  密码存储: Argon2d 哈希                                       │
│  会话管理: JWT Token + 内存/数据库缓存                         │
│  权限模型: 简单 RBAC (6种固定角色)                            │
│  权限检查: 无 (总是允许)                                      │
└─────────────────────────────────────────────────────────────┘
```

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

#### 2.1.2 OpenObserve 企业版增强

企业版通过 `#[cfg(feature = "enterprise")]` 条件编译启用：

| 功能 | 实现 |
|------|------|
| SSO/OIDC | Dex 集成 (`o2_dex` 库) |
| LDAP | Dex LDAP Connector |
| 细粒度权限 | OpenFGA (`o2_openfga` 库) |
| 自定义角色 | OpenFGA 角色管理 |

### 2.2 设计目标

| 目标 | 描述 | 指标 |
|------|------|------|
| 安全性 | 企业级身份认证和访问控制 | 符合 OWASP 安全标准 |
| 性能 | 低延迟权限检查 | P95 < 50ms |
| 可扩展性 | 支持大规模用户和权限 | 100K+ 用户，1M+ 权限 |
| 兼容性 | 与 OpenObserve API 兼容 | 100% API 兼容 |
| 可维护性 | 清晰的模块边界 | 独立部署和升级 |

### 2.3 设计约束

| 约束 | 描述 |
|------|------|
| 语言约束 | 必须使用 Rust 实现 |
| 协议约束 | 必须支持 OIDC/OAuth 2.0 标准 |
| 部署约束 | Dex 和 OpenFGA 作为独立服务 |
| 兼容约束 | 保持与 OpenObserve 企业版 API 兼容 |

### 2.4 假设与依赖

| 依赖 | 版本 | 说明 |
|------|------|------|
| Dex | >= 2.37 | 身份认证服务 |
| OpenFGA | >= 1.3 | 授权服务 |
| PostgreSQL | >= 14 | 后端存储 |
| Rust | >= 1.75 | 开发语言 |

---

## 3. 架构设计

### 3.1 系统架构图

```
                                    ┌─────────────────┐
                                    │   External IdP  │
                                    │ (LDAP/Google/   │
                                    │  GitHub/Azure)  │
                                    └────────┬────────┘
                                             │
                                             ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Browser   │───▶│   Visdata   │───▶│     Dex     │
│   / Client  │    │   Backend   │    │  (AuthN)    │
└─────────────┘    └──────┬──────┘    └─────────────┘
                          │                  │
                          │                  ▼
                          │           ┌─────────────┐
                          │           │  Dex Store  │
                          │           │ (PostgreSQL)│
                          │           └─────────────┘
                          │
                          ▼
                   ┌─────────────┐    ┌─────────────┐
                   │   OpenFGA   │───▶│ FGA Store   │
                   │   (AuthZ)   │    │ (PostgreSQL)│
                   └─────────────┘    └─────────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │   Cache     │
                   │  (DashMap)  │
                   └─────────────┘
```

### 3.2 组件职责

#### 3.2.1 Visdata Backend

**职责：**
- 接收和处理 HTTP 请求
- 协调认证和授权流程
- 管理用户会话
- 提供 REST API

**关键模块** (`o2_visdata/src/`):
```
├── lib.rs                 # 模块入口，全局单例
├── dex/                   # Dex 认证模块
│   ├── client.rs          # Dex HTTP 客户端
│   ├── config.rs          # Dex 配置
│   └── service/token.rs   # Token 验证服务
├── openfga/               # OpenFGA 授权模块
│   ├── client.rs          # OpenFGA HTTP 客户端
│   ├── config.rs          # OpenFGA 配置
│   └── authorizer/        # 授权检查器
└── config/                # 配置管理
```

#### 3.2.2 Dex (认证服务)

**职责：**
- OIDC 身份认证
- 多 IdP 连接器管理
- JWT Token 签发
- 用户密码管理

**接口：**
- HTTP: `:5556` (OIDC 端点)
- gRPC: `:5557` (管理 API)

#### 3.2.3 OpenFGA (授权服务)

**职责：**
- 存储授权模型
- 管理权限元组
- 执行权限检查
- 支持关系查询

**接口：**
- HTTP: `:8080` (REST API)
- gRPC: `:8081` (可选)

#### 3.2.4 存储层

| 存储 | 用途 | 数据 |
|------|------|------|
| Dex PostgreSQL | Dex 后端 | 用户凭证、连接器配置、刷新令牌 |
| OpenFGA PostgreSQL | OpenFGA 后端 | 授权模型、权限元组 |
| Visdata 内存缓存 | 权限缓存 | 权限检查结果 |

### 3.3 部署架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Kubernetes Cluster                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Visdata    │  │   Visdata    │  │   Visdata    │          │
│  │   Pod #1     │  │   Pod #2     │  │   Pod #3     │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                    │
│         └────────────┬────┴────────────┬────┘                    │
│                      │                 │                         │
│                      ▼                 ▼                         │
│              ┌──────────────┐  ┌──────────────┐                 │
│              │     Dex      │  │   OpenFGA    │                 │
│              │   Service    │  │   Service    │                 │
│              │  (2 replicas)│  │  (2 replicas)│                 │
│              └──────┬───────┘  └──────┬───────┘                 │
│                     │                 │                          │
│                     └────────┬────────┘                          │
│                              │                                   │
│                              ▼                                   │
│                      ┌──────────────┐                           │
│                      │  PostgreSQL  │                           │
│                      │   (Primary)  │                           │
│                      └──────────────┘                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.4 技术选型

| 技术 | 选型 | 理由 |
|------|------|------|
| 认证服务 | Dex | 开源、支持多 IdP、OIDC 标准 |
| 授权服务 | OpenFGA | 细粒度、高性能、Google Zanzibar 实现 |
| HTTP 客户端 | reqwest | Rust 生态主流、异步支持 |
| 缓存 | DashMap | 高并发、无锁读 |
| 序列化 | serde_json | Rust 标准、性能好 |
| JWT | jsonwebtoken | Rust 主流 JWT 库 |

---

## 4. 认证模块设计 (Dex)

### 4.1 认证流程设计

#### 4.1.1 本地登录流程

```
┌────────┐     ┌─────────┐     ┌────────┐     ┌────────┐
│ Client │     │ Visdata │     │  Dex   │     │  DB    │
└───┬────┘     └────┬────┘     └───┬────┘     └───┬────┘
    │               │              │              │
    │ POST /auth/login             │              │
    │ {email, password}            │              │
    │──────────────▶│              │              │
    │               │              │              │
    │               │ VerifyPassword               │
    │               │─────────────▶│              │
    │               │              │              │
    │               │              │ Query User   │
    │               │              │─────────────▶│
    │               │              │              │
    │               │              │◀─────────────│
    │               │              │              │
    │               │◀─────────────│              │
    │               │ (verified)   │              │
    │               │              │              │
    │               │ Exchange Token               │
    │               │─────────────▶│              │
    │               │              │              │
    │               │◀─────────────│              │
    │               │ (JWT tokens) │              │
    │               │              │              │
    │ Set-Cookie    │              │              │
    │ {access_token,│              │              │
    │  refresh_token}              │              │
    │◀──────────────│              │              │
```

**实现代码** (`src/dex/handler/login.rs`):
```rust
pub async fn login(
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let dex_client = Visdata::global().dex();

    // 1. 验证密码
    let verified = dex_client
        .verify_password(&req.email, &req.password)
        .await?;

    if !verified {
        return Err(Error::InvalidCredentials);
    }

    // 2. 生成 Token
    let tokens = dex_service::exchange_code_for_tokens(
        &req.email,
    ).await?;

    // 3. 创建会话
    let session_id = create_session(&tokens.access_token).await?;

    // 4. 设置 Cookie
    Ok(HttpResponse::Ok()
        .cookie(Cookie::build("auth_tokens", &tokens.access_token)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .finish())
        .json(LoginResponse {
            status: "success",
            expires_in: tokens.expires_in,
        }))
}
```

#### 4.1.2 SSO/OIDC 登录流程

```
┌────────┐     ┌─────────┐     ┌────────┐     ┌────────┐
│ Client │     │ Visdata │     │  Dex   │     │  IdP   │
└───┬────┘     └────┬────┘     └───┬────┘     └───┬────┘
    │               │              │              │
    │ GET /auth/login              │              │
    │──────────────▶│              │              │
    │               │              │              │
    │ 302 Redirect  │              │              │
    │ to Dex /auth  │              │              │
    │◀──────────────│              │              │
    │               │              │              │
    │ GET /auth?client_id=...      │              │
    │─────────────────────────────▶│              │
    │               │              │              │
    │ 302 Redirect to IdP          │              │
    │◀─────────────────────────────│              │
    │               │              │              │
    │ Authenticate with IdP        │              │
    │─────────────────────────────────────────────▶
    │               │              │              │
    │◀─────────────────────────────────────────────
    │ (auth code)   │              │              │
    │               │              │              │
    │ Callback to Dex              │              │
    │─────────────────────────────▶│              │
    │               │              │              │
    │ 302 Redirect to Visdata      │              │
    │ /config/redirect?code=...    │              │
    │◀─────────────────────────────│              │
    │               │              │              │
    │ GET /config/redirect         │              │
    │──────────────▶│              │              │
    │               │              │              │
    │               │ POST /token  │              │
    │               │─────────────▶│              │
    │               │              │              │
    │               │◀─────────────│              │
    │               │ (JWT tokens) │              │
    │               │              │              │
    │ Set-Cookie + Redirect        │              │
    │◀──────────────│              │              │
```

**PKCE 支持** (`src/dex/service/token.rs`):
```rust
pub async fn pre_login() -> Result<PreLoginResponse, Error> {
    // 生成 PKCE code_verifier
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    // 构建授权 URL
    let auth_url = format!(
        "{}/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256",
        dex_cfg.issuer_url,
        dex_cfg.client_id,
        dex_cfg.redirect_uri,
        dex_cfg.scopes.join("+"),
        code_challenge,
    );

    Ok(PreLoginResponse {
        auth_url,
        code_verifier, // 存储在 session 中
    })
}
```

#### 4.1.3 LDAP 登录流程

LDAP 登录通过 Dex LDAP Connector 实现，流程与 SSO 相同，区别在于：

1. Dex 配置 LDAP Connector
2. 用户在 Dex 登录页选择 LDAP
3. Dex 转发认证请求到 LDAP 服务器
4. LDAP 返回用户信息和群组

**群组映射** (`src/dex/service/token.rs`):
```rust
pub async fn process_ldap_groups(
    claims: &JwtClaims,
    dex_cfg: &DexConfig,
) -> Result<(String, UserRole), Error> {
    let groups = claims.get(&dex_cfg.group_claim)
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![]);

    let mut org_id = dex_cfg.default_org.clone();
    let mut role = UserRole::from_str(&dex_cfg.default_role)?;

    for group in groups {
        let group_str = group.as_str().unwrap_or("");

        // 解析 LDAP DN 格式: "cn=admin,ou=groups,dc=example,dc=com"
        for part in group_str.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() == 2 {
                if kv[0] == dex_cfg.group_attribute && org_id.is_empty() {
                    org_id = kv[1].to_string();
                }
                if kv[0] == dex_cfg.role_attribute && role == UserRole::User {
                    role = UserRole::from_str(kv[1])?;
                }
            }
        }
    }

    Ok((org_id, role))
}
```

#### 4.1.4 Token 刷新流程

```
┌────────┐     ┌─────────┐     ┌────────┐
│ Client │     │ Visdata │     │  Dex   │
└───┬────┘     └────┬────┘     └───┬────┘
    │               │              │
    │ POST /auth/refresh           │
    │ {refresh_token}              │
    │──────────────▶│              │
    │               │              │
    │               │ POST /token  │
    │               │ grant_type=  │
    │               │ refresh_token│
    │               │─────────────▶│
    │               │              │
    │               │◀─────────────│
    │               │ (new tokens) │
    │               │              │
    │ Set-Cookie    │              │
    │ (new tokens)  │              │
    │◀──────────────│              │
```

**实现** (`src/dex/service/token.rs`):
```rust
pub async fn refresh_token(
    refresh_token: &str,
) -> Result<TokenResponse, Error> {
    let dex_cfg = get_dex_config();

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", &dex_cfg.client_id),
        ("client_secret", &dex_cfg.client_secret),
    ];

    let response = HTTP_CLIENT
        .post(&format!("{}/token", dex_cfg.issuer_url))
        .form(&params)
        .send()
        .await?;

    let tokens: TokenResponse = response.json().await?;
    Ok(tokens)
}
```

#### 4.1.5 登出流程

```rust
pub async fn logout(session_id: &str) -> Result<(), Error> {
    // 1. 删除本地会话
    delete_session(session_id).await?;

    // 2. 撤销 Dex Refresh Token
    if let Some(refresh_token) = get_refresh_token(session_id).await? {
        dex_client.revoke_refresh_token(&refresh_token).await?;
    }

    // 3. 清除缓存
    clear_user_cache(session_id).await?;

    Ok(())
}
```

### 4.2 Dex 集成设计

#### 4.2.1 Dex 客户端实现

**客户端结构** (`src/dex/client.rs`):
```rust
pub struct DexClient {
    http_client: reqwest::Client,
    config: DexConfig,
}

impl DexClient {
    pub async fn new(config: DexConfig) -> Result<Self, Error> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self { http_client, config })
    }

    // 密码验证
    pub async fn verify_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<bool, Error>;

    // 创建用户密码
    pub async fn create_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), Error>;

    // 更新密码
    pub async fn update_password(
        &self,
        email: &str,
        new_password: &str,
    ) -> Result<(), Error>;

    // 删除密码
    pub async fn delete_password(
        &self,
        email: &str,
    ) -> Result<(), Error>;

    // 连接器管理
    pub async fn create_connector(
        &self,
        connector: &ConnectorConfig,
    ) -> Result<(), Error>;

    pub async fn list_connectors(&self) -> Result<Vec<Connector>, Error>;

    // 刷新令牌管理
    pub async fn list_refresh_tokens(
        &self,
        user_id: &str,
    ) -> Result<Vec<RefreshToken>, Error>;

    pub async fn revoke_refresh_token(
        &self,
        token_id: &str,
    ) -> Result<(), Error>;

    // 健康检查
    pub async fn is_healthy(&self) -> bool;
    pub async fn get_version(&self) -> Result<String, Error>;
}
```

#### 4.2.2 连接器配置

**支持的连接器类型：**

| 类型 | 说明 | 配置示例 |
|------|------|----------|
| `ldap` | LDAP/AD | 见下方 |
| `oidc` | 通用 OIDC | Google, Azure AD |
| `github` | GitHub OAuth | |
| `saml` | SAML 2.0 | |
| `local` | 本地密码 | 默认启用 |

**LDAP 连接器配置：**
```yaml
type: ldap
id: ldap
name: "Enterprise LDAP"
config:
  host: ldap.example.com:636
  insecureNoSSL: false
  insecureSkipVerify: false
  bindDN: cn=service,dc=example,dc=com
  bindPW: ${LDAP_PASSWORD}
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

#### 4.2.3 回调处理

**回调端点** (`src/dex/handler/login.rs`):
```rust
pub async fn sso_callback(
    query: web::Query<CallbackQuery>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let code = &query.code;
    let state = &query.state;

    // 验证 state 防止 CSRF
    let expected_state = session.get::<String>("oauth_state")?;
    if Some(state) != expected_state.as_ref() {
        return Err(Error::InvalidState);
    }

    // 获取 PKCE code_verifier
    let code_verifier = session.get::<String>("code_verifier")?
        .ok_or(Error::MissingCodeVerifier)?;

    // 交换 Token
    let tokens = exchange_code(code, &code_verifier).await?;

    // 验证并解析 JWT
    let claims = verify_token(&tokens.id_token).await?;

    // 自动创建/更新用户
    let user = sync_user_from_claims(&claims).await?;

    // 创建会话
    let session_id = create_session(&user, &tokens).await?;

    // 重定向到首页
    Ok(HttpResponse::Found()
        .cookie(build_auth_cookie(&tokens))
        .append_header(("Location", "/"))
        .finish())
}
```

### 4.3 Token 管理设计

#### 4.3.1 JWT 结构设计

**Access Token Claims：**
```json
{
  "iss": "https://dex.example.com",
  "sub": "CiQ1MjM0NTY3OC1hYmNkLTEyMzQtNTY3OC1hYmNkZWYxMjM0NTYSB2xkYXA",
  "aud": "openobserve",
  "exp": 1735689600,
  "iat": 1735686000,
  "nonce": "random-nonce",
  "at_hash": "abc123",
  "email": "user@example.com",
  "email_verified": true,
  "name": "User Name",
  "groups": ["org1-admin", "org2-viewer"]
}
```

**Token 验证** (`src/dex/service/token.rs`):
```rust
pub async fn verify_token(token: &str) -> Result<JwtClaims, Error> {
    let dex_cfg = get_dex_config();

    // 获取 JWKS
    let jwks = get_jwks(&dex_cfg.issuer_url).await?;

    // 解析 Token Header 获取 kid
    let header = jsonwebtoken::decode_header(token)?;
    let kid = header.kid.ok_or(Error::MissingKid)?;

    // 查找对应的 JWK
    let jwk = jwks.keys.iter()
        .find(|k| k.kid == Some(kid.clone()))
        .ok_or(Error::KeyNotFound)?;

    // 验证签名
    let validation = Validation::new(Algorithm::RS256);
    let token_data = jsonwebtoken::decode::<JwtClaims>(
        token,
        &DecodingKey::from_jwk(jwk)?,
        &validation,
    )?;

    // 验证 issuer 和 audience
    if token_data.claims.iss != dex_cfg.issuer_url {
        return Err(Error::InvalidIssuer);
    }
    if token_data.claims.aud != dex_cfg.client_id {
        return Err(Error::InvalidAudience);
    }

    Ok(token_data.claims)
}
```

#### 4.3.2 Token 生命周期

| Token 类型 | 默认有效期 | 存储位置 | 刷新方式 |
|------------|------------|----------|----------|
| Access Token | 1 小时 | HttpOnly Cookie | 自动刷新 |
| Refresh Token | 7 天 | HttpOnly Cookie | 重新登录 |
| ID Token | 1 小时 | 不存储 | 随 Access Token |

#### 4.3.3 JWKS 密钥管理

**JWKS 缓存：**
```rust
static JWKS_CACHE: Lazy<RwLock<Option<CachedJwks>>> = Lazy::new(Default::default);

struct CachedJwks {
    jwks: Jwks,
    cached_at: Instant,
}

pub async fn get_jwks(issuer_url: &str) -> Result<Jwks, Error> {
    // 检查缓存
    {
        let cache = JWKS_CACHE.read().await;
        if let Some(cached) = cache.as_ref() {
            if cached.cached_at.elapsed() < Duration::from_secs(300) {
                return Ok(cached.jwks.clone());
            }
        }
    }

    // 获取新的 JWKS
    let jwks_url = format!("{}/.well-known/jwks.json", issuer_url);
    let jwks: Jwks = HTTP_CLIENT.get(&jwks_url).send().await?.json().await?;

    // 更新缓存
    {
        let mut cache = JWKS_CACHE.write().await;
        *cache = Some(CachedJwks {
            jwks: jwks.clone(),
            cached_at: Instant::now(),
        });
    }

    Ok(jwks)
}
```

### 4.4 会话管理设计

#### 4.4.1 会话存储

**会话数据结构：**
```rust
pub struct Session {
    pub session_id: String,      // KSUID
    pub user_id: String,         // 用户邮箱
    pub access_token: String,    // JWT Access Token
    pub refresh_token: String,   // Refresh Token
    pub created_at: i64,         // 创建时间戳
    pub expires_at: i64,         // 过期时间戳
}
```

**存储层次：**
```
┌─────────────────┐
│   内存缓存       │  ← 热数据，快速访问
│   (DashMap)     │
└────────┬────────┘
         │ 未命中
         ▼
┌─────────────────┐
│   数据库         │  ← 持久化存储
│  (PostgreSQL)   │
└─────────────────┘
```

#### 4.4.2 会话同步 (集群)

在集群部署中，会话需要跨节点同步：

```rust
pub async fn set_session(session: &Session) -> Result<(), Error> {
    // 1. 写入本地缓存
    SESSIONS.insert(session.session_id.clone(), session.clone());

    // 2. 写入数据库
    db::sessions::set(session).await?;

    // 3. 通知其他节点 (通过 coordinator)
    if let Some(coordinator) = get_coordinator() {
        coordinator.broadcast_session_update(&session.session_id).await?;
    }

    Ok(())
}

pub async fn watch_session_updates() {
    let coordinator = get_coordinator();
    let mut rx = coordinator.subscribe_session_updates();

    while let Some(session_id) = rx.recv().await {
        // 从数据库重新加载会话
        if let Ok(session) = db::sessions::get(&session_id).await {
            SESSIONS.insert(session_id, session);
        } else {
            SESSIONS.remove(&session_id);
        }
    }
}
```

#### 4.4.3 会话失效策略

| 策略 | 触发条件 | 处理方式 |
|------|----------|----------|
| 超时失效 | Access Token 过期 | 自动刷新或重新登录 |
| 主动登出 | 用户点击登出 | 删除会话和 Token |
| 强制失效 | 管理员强制登出 | 广播删除消息 |
| 密码变更 | 用户修改密码 | 失效所有会话 |

### 4.5 用户同步与映射

#### 4.5.1 自动用户创建

首次 SSO 登录时自动创建本地用户：

```rust
pub async fn sync_user_from_claims(
    claims: &JwtClaims,
) -> Result<User, Error> {
    let email = claims.email.to_lowercase();

    // 检查用户是否存在
    if let Ok(user) = db::users::get(&email).await {
        // 更新用户信息
        let updated_user = User {
            email: email.clone(),
            first_name: claims.given_name.clone().unwrap_or_default(),
            last_name: claims.family_name.clone().unwrap_or_default(),
            ..user
        };
        db::users::update(&updated_user).await?;
        return Ok(updated_user);
    }

    // 创建新用户
    let dex_cfg = get_dex_config();
    let (org_id, role) = process_groups(claims, &dex_cfg).await?;

    let new_user = User {
        id: email.clone(),
        email,
        first_name: claims.given_name.clone().unwrap_or_default(),
        last_name: claims.family_name.clone().unwrap_or_default(),
        is_root: false,
        user_type: UserType::External,
        ..Default::default()
    };

    db::users::create(&new_user).await?;

    // 添加到组织
    db::org_users::add(&org_id, &new_user.email, role).await?;

    // 在 OpenFGA 创建权限
    openfga::add_user_to_org(&new_user.email, &org_id, &role).await?;

    Ok(new_user)
}
```

#### 4.5.2 群组到组织映射

**映射规则配置：**
```toml
[dex]
group_claim = "groups"           # OIDC claim 名称
group_attribute = "ou"           # LDAP 属性名
default_org = "default"          # 默认组织

# 群组映射规则 (可选)
[dex.group_mappings]
"cn=developers,ou=groups,dc=example,dc=com" = "dev-org"
"cn=ops,ou=groups,dc=example,dc=com" = "ops-org"
```

#### 4.5.3 群组到角色映射

**映射规则配置：**
```toml
[dex]
role_attribute = "role"          # LDAP 属性名
default_role = "viewer"          # 默认角色

# 角色映射规则 (可选)
[dex.role_mappings]
"cn=admins,ou=groups,dc=example,dc=com" = "admin"
"cn=editors,ou=groups,dc=example,dc=com" = "editor"
"cn=viewers,ou=groups,dc=example,dc=com" = "viewer"
```

---

## 5. 授权模块设计 (OpenFGA)

### 5.1 授权模型设计

#### 5.1.1 类型定义 (Types)

Visdata 授权模型支持 30+ 资源类型：

```dsl
model
  schema 1.1

# 基础类型
type user
type group
type role
type org

# 数据类型
type stream
type logs
type metrics
type traces
type metadata
type index

# 可视化类型
type dashboard
type dfolder
type savedviews
type template

# 告警类型
type alert
type afolder
type destination

# 报告类型
type report
type rfolder

# 功能类型
type function
type pipeline
type enrichment_table
type kv
```

#### 5.1.2 关系定义 (Relations)

**用户组关系：**
```dsl
type group
  relations
    define member: [user, group#member]
```

**角色关系：**
```dsl
type role
  relations
    define assignee: [user, group#member]
```

**组织关系：**
```dsl
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
```

**资源关系 (以 dashboard 为例)：**
```dsl
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
```

#### 5.1.3 权限继承设计

```
                    ┌─────────┐
                    │   org   │
                    │ owner   │
                    └────┬────┘
                         │ inherits
                         ▼
                    ┌─────────┐
                    │   org   │
                    │  admin  │
                    └────┬────┘
                         │ inherits
            ┌────────────┼────────────┐
            ▼            ▼            ▼
       ┌─────────┐  ┌─────────┐  ┌─────────┐
       │ dfolder │  │ stream  │  │ afolder │
       │  owner  │  │  owner  │  │  owner  │
       └────┬────┘  └─────────┘  └─────────┘
            │
            ▼
       ┌─────────┐
       │dashboard│
       │  owner  │
       └─────────┘
```

### 5.2 OpenFGA 集成设计

#### 5.2.1 OpenFGA 客户端实现

**客户端结构** (`src/openfga/client.rs`):
```rust
pub struct OpenFGAClient {
    http_client: reqwest::Client,
    config: OpenFGAConfig,
    store_id: String,
    model_id: String,
}

impl OpenFGAClient {
    pub async fn new(config: OpenFGAConfig) -> Result<Self, Error> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        let mut client = Self {
            http_client,
            config,
            store_id: String::new(),
            model_id: String::new(),
        };

        // 初始化 Store 和 Model
        client.initialize().await?;

        Ok(client)
    }

    async fn initialize(&mut self) -> Result<(), Error> {
        // 查找或创建 Store
        self.store_id = self.get_or_create_store().await?;

        // 写入授权模型
        self.model_id = self.write_authorization_model().await?;

        Ok(())
    }

    // 权限检查
    pub async fn check(&self, tuple: &TupleKey) -> Result<bool, Error>;

    // 批量检查
    pub async fn batch_check(&self, tuples: &[TupleKey]) -> Result<Vec<bool>, Error>;

    // 写入元组
    pub async fn write(&self, writes: Vec<TupleKey>) -> Result<(), Error>;

    // 删除元组
    pub async fn delete(&self, deletes: Vec<TupleKey>) -> Result<(), Error>;

    // 读取元组
    pub async fn read(&self, tuple: &TupleKey) -> Result<Vec<Tuple>, Error>;

    // 列出用户可访问的对象
    pub async fn list_objects(
        &self,
        user: &str,
        relation: &str,
        object_type: &str,
    ) -> Result<Vec<String>, Error>;

    // 展开关系
    pub async fn expand(
        &self,
        tuple: &TupleKey,
    ) -> Result<ExpandResponse, Error>;
}
```

#### 5.2.2 Store 初始化

```rust
async fn get_or_create_store(&self) -> Result<String, Error> {
    // 列出现有 Store
    let stores = self.list_stores().await?;

    // 查找同名 Store
    if let Some(store) = stores.iter().find(|s| s.name == self.config.store_name) {
        return Ok(store.id.clone());
    }

    // 创建新 Store
    let response = self.http_client
        .post(&format!("{}/stores", self.config.api_url))
        .json(&json!({ "name": self.config.store_name }))
        .send()
        .await?;

    let store: Store = response.json().await?;
    Ok(store.id)
}
```

#### 5.2.3 Model 管理

**授权模型加载** (`src/openfga/model/schema.rs`):
```rust
const AUTHORIZATION_MODEL: &str = include_str!("authorization_model.json");

pub async fn write_authorization_model(client: &OpenFGAClient) -> Result<String, Error> {
    let model: AuthorizationModel = serde_json::from_str(AUTHORIZATION_MODEL)?;

    let response = client.http_client
        .post(&format!(
            "{}/stores/{}/authorization-models",
            client.config.api_url,
            client.store_id
        ))
        .json(&model)
        .send()
        .await?;

    let result: WriteModelResponse = response.json().await?;
    Ok(result.authorization_model_id)
}
```

### 5.3 权限检查设计

#### 5.3.1 检查流程

```rust
pub async fn is_allowed(
    org_id: &str,
    user_id: &str,
    method: &str,
    object: &str,
    parent_id: &str,
    role: &str,
) -> Result<bool, Error> {
    // 1. Root 用户跳过检查
    if is_root_user(user_id) {
        return Ok(true);
    }

    // 2. 检查 OpenFGA 是否启用
    let config = get_openfga_config();
    if !config.enabled {
        return Ok(true);
    }

    // 3. 构建缓存 Key
    let cache_key = format!("{}:{}:{}:{}", user_id, method, object, org_id);

    // 4. 检查缓存
    if let Some(result) = PERMISSION_CACHE.get(&cache_key) {
        return Ok(*result);
    }

    // 5. 调用 OpenFGA 检查
    let client = Visdata::global().openfga();
    let tuple = TupleKey {
        user: format!("user:{}", user_id),
        relation: method_to_relation(method),
        object: object.to_string(),
    };

    let allowed = client.check(&tuple).await?;

    // 6. 更新缓存
    PERMISSION_CACHE.insert(cache_key, allowed, get_cache_ttl());

    Ok(allowed)
}

fn method_to_relation(method: &str) -> &str {
    match method {
        "GET" | "LIST" => "can_read",
        "POST" | "PUT" | "PATCH" => "can_write",
        "DELETE" => "can_delete",
        _ => "can_read",
    }
}
```

#### 5.3.2 缓存策略

**缓存配置：**
```rust
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,    // 默认 300 秒
    pub max_entries: usize,  // 默认 10000
}
```

**缓存实现：**
```rust
static PERMISSION_CACHE: Lazy<DashMap<String, CachedPermission>> =
    Lazy::new(DashMap::new);

struct CachedPermission {
    allowed: bool,
    cached_at: Instant,
}

impl PERMISSION_CACHE {
    fn get(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|entry| {
            let config = get_cache_config();
            if entry.cached_at.elapsed() < Duration::from_secs(config.ttl_seconds) {
                Some(entry.allowed)
            } else {
                None
            }
        })
    }

    fn insert(&self, key: String, allowed: bool) {
        let config = get_cache_config();

        // 检查容量
        if self.len() >= config.max_entries {
            self.evict_oldest();
        }

        self.insert(key, CachedPermission {
            allowed,
            cached_at: Instant::now(),
        });
    }
}
```

**缓存失效：**
```rust
pub fn invalidate_user_cache(user_id: &str) {
    PERMISSION_CACHE.retain(|k, _| !k.starts_with(&format!("{}:", user_id)));
}

pub fn invalidate_object_cache(object: &str) {
    PERMISSION_CACHE.retain(|k, _| !k.contains(&format!(":{}:", object)));
}

pub fn invalidate_all_cache() {
    PERMISSION_CACHE.clear();
}
```

#### 5.3.3 批量检查

```rust
pub async fn batch_check_permissions(
    user_id: &str,
    checks: Vec<(String, String)>,  // (method, object)
) -> Result<Vec<bool>, Error> {
    let client = Visdata::global().openfga();

    let tuples: Vec<TupleKey> = checks.iter()
        .map(|(method, object)| TupleKey {
            user: format!("user:{}", user_id),
            relation: method_to_relation(method).to_string(),
            object: object.clone(),
        })
        .collect();

    client.batch_check(&tuples).await
}
```

### 5.4 权限管理设计

#### 5.4.1 元组写入

```rust
pub async fn grant_permission(
    user_id: &str,
    relation: &str,
    object: &str,
) -> Result<(), Error> {
    let client = Visdata::global().openfga();

    let tuple = TupleKey {
        user: format!("user:{}", user_id),
        relation: relation.to_string(),
        object: object.to_string(),
    };

    client.write(vec![tuple]).await?;

    // 失效缓存
    invalidate_user_cache(user_id);

    Ok(())
}
```

#### 5.4.2 元组删除

```rust
pub async fn revoke_permission(
    user_id: &str,
    relation: &str,
    object: &str,
) -> Result<(), Error> {
    let client = Visdata::global().openfga();

    let tuple = TupleKey {
        user: format!("user:{}", user_id),
        relation: relation.to_string(),
        object: object.to_string(),
    };

    client.delete(vec![tuple]).await?;

    // 失效缓存
    invalidate_user_cache(user_id);

    Ok(())
}
```

#### 5.4.3 权限查询

```rust
pub async fn list_user_permissions(
    user_id: &str,
    object_type: &str,
) -> Result<Vec<String>, Error> {
    let client = Visdata::global().openfga();

    client.list_objects(
        &format!("user:{}", user_id),
        "can_read",
        object_type,
    ).await
}
```

### 5.5 角色管理设计

#### 5.5.1 预定义角色

| 角色 | 权限 | 说明 |
|------|------|------|
| Root | 全部 | 超级管理员，跳过所有权限检查 |
| Admin | org#admin | 组织管理员 |
| Editor | org#editor | 编辑者 |
| Viewer | org#viewer | 只读查看者 |
| User | 无 | 基础用户，需单独授权 |

#### 5.5.2 自定义角色

**创建角色：**
```rust
pub async fn create_role(
    org_id: &str,
    role_name: &str,
    permissions: Vec<O2EntityAuthorization>,
) -> Result<(), Error> {
    let client = Visdata::global().openfga();

    // 创建角色对象
    let role_object = format!("role:{}_{}", org_id, role_name);

    // 写入角色权限
    let tuples: Vec<TupleKey> = permissions.iter()
        .map(|p| TupleKey {
            user: role_object.clone(),
            relation: permission_to_relation(&p.permission),
            object: p.object.clone(),
        })
        .collect();

    client.write(tuples).await?;

    Ok(())
}
```

#### 5.5.3 角色分配

```rust
pub async fn assign_role(
    user_id: &str,
    role_name: &str,
    org_id: &str,
) -> Result<(), Error> {
    let client = Visdata::global().openfga();

    let tuple = TupleKey {
        user: format!("user:{}", user_id),
        relation: "assignee".to_string(),
        object: format!("role:{}_{}", org_id, role_name),
    };

    client.write(vec![tuple]).await?;
    invalidate_user_cache(user_id);

    Ok(())
}
```

### 5.6 用户组设计

#### 5.6.1 组结构

```rust
pub struct Group {
    pub id: String,
    pub name: String,
    pub org_id: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

#### 5.6.2 组成员管理

```rust
pub async fn add_user_to_group(
    user_id: &str,
    group_id: &str,
) -> Result<(), Error> {
    let client = Visdata::global().openfga();

    let tuple = TupleKey {
        user: format!("user:{}", user_id),
        relation: "member".to_string(),
        object: format!("group:{}", group_id),
    };

    client.write(vec![tuple]).await?;
    invalidate_user_cache(user_id);

    Ok(())
}

pub async fn remove_user_from_group(
    user_id: &str,
    group_id: &str,
) -> Result<(), Error> {
    let client = Visdata::global().openfga();

    let tuple = TupleKey {
        user: format!("user:{}", user_id),
        relation: "member".to_string(),
        object: format!("group:{}", group_id),
    };

    client.delete(vec![tuple]).await?;
    invalidate_user_cache(user_id);

    Ok(())
}
```

#### 5.6.3 组权限继承

通过 OpenFGA 模型实现组权限继承：

```dsl
type dashboard
  relations
    # 组成员可以通过组获得权限
    define viewer: [user, group#member] or viewer from org
```

当用户加入组后，自动继承组的所有权限。

---

## 6. 接口设计

### 6.1 内部接口

#### 6.1.1 Dex gRPC/HTTP 接口

| 接口 | 方法 | 路径 | 说明 |
|------|------|------|------|
| 授权 | GET | `/auth` | OIDC 授权端点 |
| Token | POST | `/token` | Token 交换 |
| 用户信息 | GET | `/userinfo` | 获取用户信息 |
| JWKS | GET | `/keys` | 获取签名公钥 |
| 发现 | GET | `/.well-known/openid-configuration` | OIDC 发现 |

#### 6.1.2 OpenFGA HTTP 接口

| 接口 | 方法 | 路径 | 说明 |
|------|------|------|------|
| 检查 | POST | `/stores/{store_id}/check` | 权限检查 |
| 写入 | POST | `/stores/{store_id}/write` | 写入元组 |
| 读取 | POST | `/stores/{store_id}/read` | 读取元组 |
| 列对象 | POST | `/stores/{store_id}/list-objects` | 列出可访问对象 |
| 展开 | POST | `/stores/{store_id}/expand` | 展开关系 |

### 6.2 外部 API 设计

#### 6.2.1 认证 API

| 端点 | 方法 | 说明 | 请求体 |
|------|------|------|--------|
| `/auth/login` | GET | 获取登录状态/发起 SSO | - |
| `/auth/login` | POST | 本地登录 | `{email, password}` |
| `/auth/refresh` | POST | 刷新 Token | `{refresh_token}` |
| `/auth/logout` | POST | 登出 | - |
| `/config/redirect` | GET | SSO 回调 | - |

**登录请求/响应：**
```json
// POST /auth/login
// Request
{
  "email": "user@example.com",
  "password": "password123"
}

// Response
{
  "status": "success",
  "expires_in": 3600
}
// Set-Cookie: auth_tokens=<jwt>; HttpOnly; Secure; SameSite=Strict
```

#### 6.2.2 用户管理 API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/{org_id}/users` | GET | 列出组织用户 |
| `/api/{org_id}/users` | POST | 创建用户 |
| `/api/{org_id}/users/{email}` | GET | 获取用户详情 |
| `/api/{org_id}/users/{email}` | PUT | 更新用户 |
| `/api/{org_id}/users/{email}` | DELETE | 删除用户 |

#### 6.2.3 角色管理 API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/{org_id}/roles` | GET | 列出角色 |
| `/api/{org_id}/roles` | POST | 创建角色 |
| `/api/{org_id}/roles/{name}` | GET | 获取角色详情 |
| `/api/{org_id}/roles/{name}` | PUT | 更新角色 |
| `/api/{org_id}/roles/{name}` | DELETE | 删除角色 |

**创建角色请求：**
```json
// POST /api/{org_id}/roles
{
  "name": "dashboard-editor",
  "permissions": [
    {"object": "dashboard:*", "permission": "AllowAll"},
    {"object": "dfolder:*", "permission": "AllowAll"},
    {"object": "stream:*", "permission": "AllowGet"}
  ]
}
```

#### 6.2.4 用户组管理 API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/{org_id}/groups` | GET | 列出用户组 |
| `/api/{org_id}/groups` | POST | 创建用户组 |
| `/api/{org_id}/groups/{name}` | GET | 获取用户组详情 |
| `/api/{org_id}/groups/{name}` | PUT | 更新用户组 |
| `/api/{org_id}/groups/{name}` | DELETE | 删除用户组 |

### 6.3 中间件设计

#### 6.3.1 认证中间件

```rust
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;

    fn new_transform(&self, service: S) -> Self::Transform {
        AuthMiddlewareService { service }
    }
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S> {
    async fn call(&self, req: ServiceRequest) -> Result<Self::Response, Self::Error> {
        // 1. 提取认证信息
        let auth_info = extract_auth_info(&req)?;

        // 2. 验证 Token
        let claims = verify_token(&auth_info.token).await?;

        // 3. 加载用户信息
        let user = load_user(&claims.email).await?;

        // 4. 注入用户上下文
        req.extensions_mut().insert(user);

        // 5. 继续处理请求
        self.service.call(req).await
    }
}
```

#### 6.3.2 授权中间件

```rust
pub struct AuthzMiddleware;

impl<S, B> Service<ServiceRequest> for AuthzMiddlewareService<S> {
    async fn call(&self, req: ServiceRequest) -> Result<Self::Response, Self::Error> {
        // 1. 获取用户上下文
        let user = req.extensions().get::<User>()
            .ok_or(Error::Unauthorized)?;

        // 2. 提取授权信息
        let auth_extractor = AuthExtractor::from_request(&req)?;

        // 3. 检查权限
        let allowed = check_permissions(
            &user.email,
            &auth_extractor.org_id,
            &auth_extractor.method,
            &auth_extractor.o2_type,
            &user.role.to_string(),
        ).await;

        if !allowed {
            return Err(Error::Forbidden);
        }

        // 4. 继续处理请求
        self.service.call(req).await
    }
}
```

---

## 7. 数据设计

### 7.1 数据模型

#### 7.1.1 用户数据模型

```rust
// 用户表
pub struct User {
    pub id: String,           // email
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,     // Argon2d hash
    pub salt: String,
    pub is_root: bool,
    pub password_ext: Option<String>,
    pub user_type: UserType,
    pub created_at: i64,
    pub updated_at: i64,
}

// 组织用户关联
pub struct OrgUser {
    pub id: String,
    pub email: String,
    pub org_id: String,
    pub role: UserRole,
    pub token: String,
    pub rum_token: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

#### 7.1.2 会话数据模型

```rust
pub struct Session {
    pub session_id: String,   // KSUID
    pub access_token: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

#### 7.1.3 OpenFGA 授权模型

见 5.1 节详细说明。

### 7.2 数据存储设计

#### 7.2.1 Dex 存储

**PostgreSQL Schema：**
```sql
-- 密码表
CREATE TABLE passwords (
    email VARCHAR(255) PRIMARY KEY,
    hash BYTEA NOT NULL,
    username VARCHAR(255),
    user_id VARCHAR(255)
);

-- 连接器表
CREATE TABLE connectors (
    id VARCHAR(255) PRIMARY KEY,
    type VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    resource_version VARCHAR(255),
    config JSONB
);

-- 刷新令牌表
CREATE TABLE refresh_tokens (
    id VARCHAR(255) PRIMARY KEY,
    client_id VARCHAR(255),
    scopes TEXT[],
    nonce VARCHAR(255),
    claims_user_id VARCHAR(255),
    claims_username VARCHAR(255),
    claims_email VARCHAR(255),
    connector_id VARCHAR(255),
    connector_data BYTEA,
    token VARCHAR(255),
    created_at TIMESTAMP,
    last_used TIMESTAMP
);
```

#### 7.2.2 OpenFGA 存储

**PostgreSQL Schema：**
```sql
-- Store 表
CREATE TABLE stores (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

-- 授权模型表
CREATE TABLE authorization_models (
    id VARCHAR(255) PRIMARY KEY,
    store_id VARCHAR(255) REFERENCES stores(id),
    schema_version VARCHAR(255),
    type_definitions JSONB,
    created_at TIMESTAMP
);

-- 元组表
CREATE TABLE tuples (
    store_id VARCHAR(255),
    object_type VARCHAR(255),
    object_id VARCHAR(255),
    relation VARCHAR(255),
    user_type VARCHAR(255),
    user_id VARCHAR(255),
    user_relation VARCHAR(255),
    created_at TIMESTAMP,
    PRIMARY KEY (store_id, object_type, object_id, relation, user_type, user_id, user_relation)
);
```

#### 7.2.3 缓存设计

**缓存层次：**
```
┌─────────────────────────────────────────┐
│           L1: 本地内存缓存               │
│           (DashMap, TTL=300s)           │
├─────────────────────────────────────────┤
│           L2: 数据库                     │
│           (PostgreSQL)                  │
└─────────────────────────────────────────┘
```

**缓存 Key 设计：**
| 缓存类型 | Key 格式 | 示例 |
|----------|----------|------|
| 权限检查 | `{user}:{method}:{object}:{org}` | `alice@ex.com:GET:dashboard:123:org1` |
| 用户信息 | `user:{email}` | `user:alice@example.com` |
| 会话 | `session:{session_id}` | `session:2GnQmXyz...` |
| JWKS | `jwks:{issuer}` | `jwks:https://dex.example.com` |

### 7.3 数据迁移设计

**迁移脚本结构：**
```
migrations/
├── 001_create_users.sql
├── 002_create_org_users.sql
├── 003_create_sessions.sql
├── 004_migrate_from_oss.sql
└── 005_init_openfga_permissions.sql
```

**从开源版迁移：**
```sql
-- 004_migrate_from_oss.sql
-- 迁移用户数据到 Dex
INSERT INTO dex.passwords (email, hash, username, user_id)
SELECT email, password, first_name || ' ' || last_name, id
FROM openobserve.users;

-- 初始化 OpenFGA 权限
-- 通过应用代码执行
```

---

## 8. 安全设计

### 8.1 认证安全

#### 8.1.1 密码安全

| 措施 | 实现 |
|------|------|
| 哈希算法 | Argon2d (v=16, m=2048, t=4, p=2) |
| 盐值 | 随机生成，每用户唯一 |
| 密码强度 | 最小 8 字符，包含数字和字母 |
| 登录限制 | 5 次失败后锁定 15 分钟 |

#### 8.1.2 Token 安全

| 措施 | 实现 |
|------|------|
| 签名算法 | RS256 |
| 存储方式 | HttpOnly + Secure Cookie |
| 传输方式 | HTTPS only |
| 防重放 | nonce + exp 验证 |
| 防 CSRF | SameSite=Strict + PKCE |

#### 8.1.3 传输安全

| 措施 | 实现 |
|------|------|
| 协议 | TLS 1.2+ |
| 证书 | Let's Encrypt 或企业 CA |
| HSTS | max-age=31536000 |

### 8.2 授权安全

#### 8.2.1 最小权限原则

- 新用户默认无权限
- 权限按需分配
- 定期权限审计

#### 8.2.2 权限边界

- 组织隔离
- 资源级别控制
- 操作级别控制

### 8.3 审计日志设计

```rust
pub struct AuditLog {
    pub id: String,
    pub timestamp: i64,
    pub user_id: String,
    pub action: String,       // LOGIN, LOGOUT, GRANT, REVOKE, etc.
    pub resource: String,
    pub result: String,       // SUCCESS, FAILURE
    pub details: serde_json::Value,
    pub ip_address: String,
    pub user_agent: String,
}
```

### 8.4 敏感数据保护

**加密配置** (`src/config/types.rs`):
```rust
pub struct VisdataConfig {
    pub encryption_key: Option<String>,  // AES-256 密钥
}
```

**加密实现：**
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};

pub fn encrypt(plaintext: &str, key: &str) -> Result<String, Error> {
    let key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce"); // 生产环境使用随机 nonce

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
    Ok(base64::encode(ciphertext))
}

pub fn decrypt(ciphertext: &str, key: &str) -> Result<String, Error> {
    let key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");

    let ciphertext = base64::decode(ciphertext)?;
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
    Ok(String::from_utf8(plaintext)?)
}
```

---

## 9. 性能设计

### 9.1 性能目标

| 指标 | 目标 | 说明 |
|------|------|------|
| 认证延迟 | P95 < 500ms | 包括 Dex 交互 |
| 权限检查延迟 | P95 < 50ms | 缓存命中 |
| 权限检查延迟 | P95 < 200ms | 缓存未命中 |
| 并发认证 | > 1000 QPS | |
| 并发权限检查 | > 10000 QPS | |
| 缓存命中率 | > 90% | |

### 9.2 缓存策略

| 缓存类型 | TTL | 最大条目 | 淘汰策略 |
|----------|-----|----------|----------|
| 权限检查 | 300s | 10000 | LRU |
| 用户信息 | 300s | 1000 | LRU |
| JWKS | 300s | 10 | 过期刷新 |

### 9.3 并发处理

```rust
// 使用 tokio 异步运行时
#[tokio::main]
async fn main() {
    // 配置线程池
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .build()
        .unwrap();
}

// 使用连接池
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .pool_max_idle_per_host(100)
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .unwrap()
}
```

### 9.4 性能优化措施

| 优化 | 措施 |
|------|------|
| 批量检查 | 合并多个权限检查请求 |
| 预加载 | 启动时预热缓存 |
| 连接复用 | HTTP 连接池 |
| 异步 IO | tokio 异步运行时 |

---

## 10. 可靠性设计

### 10.1 高可用设计

```
┌─────────────────────────────────────────┐
│              Load Balancer              │
└────────────────┬────────────────────────┘
                 │
       ┌─────────┴─────────┐
       ▼                   ▼
┌─────────────┐     ┌─────────────┐
│  Visdata 1  │     │  Visdata 2  │
└──────┬──────┘     └──────┬──────┘
       │                   │
       └─────────┬─────────┘
                 │
       ┌─────────┴─────────┐
       ▼                   ▼
┌─────────────┐     ┌─────────────┐
│    Dex 1    │     │    Dex 2    │
└─────────────┘     └─────────────┘
       │                   │
       └─────────┬─────────┘
                 │
       ┌─────────┴─────────┐
       ▼                   ▼
┌─────────────┐     ┌─────────────┐
│  OpenFGA 1  │     │  OpenFGA 2  │
└─────────────┘     └─────────────┘
```

### 10.2 故障处理

| 故障场景 | 处理方式 |
|----------|----------|
| Dex 不可用 | 使用缓存的 JWKS 验证现有 Token |
| OpenFGA 不可用 | 根据配置拒绝或允许请求 |
| 数据库不可用 | 使用内存缓存，记录审计日志待恢复后补写 |

### 10.3 降级策略

```rust
pub async fn check_permission_with_fallback(
    user_id: &str,
    method: &str,
    object: &str,
) -> bool {
    match check_permission(user_id, method, object).await {
        Ok(allowed) => allowed,
        Err(e) => {
            log::warn!("Permission check failed: {}, using fallback", e);

            let config = get_config();
            match config.authz_fallback_policy {
                FallbackPolicy::Deny => false,
                FallbackPolicy::Allow => true,
                FallbackPolicy::AllowRead => method == "GET" || method == "LIST",
            }
        }
    }
}
```

### 10.4 数据一致性

| 场景 | 一致性保证 |
|------|------------|
| 权限变更 | 最终一致性 (缓存 TTL 内) |
| 用户创建 | 强一致性 |
| 会话管理 | 最终一致性 (集群同步) |

---

## 11. 配置设计

### 11.1 Dex 配置项

```toml
[dex]
grpc_url = "http://localhost:5557"
issuer_url = "https://dex.example.com"
client_id = "openobserve"
client_secret = "${DEX_CLIENT_SECRET}"
redirect_uri = "https://openobserve.example.com/config/redirect"
default_org = "default"
default_role = "viewer"
native_login_enabled = true
root_only_login = false
group_claim = "groups"
group_attribute = "ou"
role_attribute = "role"
scopes = ["openid", "email", "profile", "groups", "offline_access"]
timeout_seconds = 30
```

### 11.2 OpenFGA 配置项

```toml
[openfga]
api_url = "http://localhost:8080"
store_name = "openobserve"
enabled = true
list_only_permitted = true
timeout_seconds = 10
```

### 11.3 Visdata 配置项

```toml
[visdata]
rbac_enabled = true
sso_enabled = true
encryption_key = "${ENCRYPTION_KEY}"

[visdata.cache]
enabled = true
ttl_seconds = 300
max_entries = 10000
```

### 11.4 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `VISDATA_DEX_CLIENT_SECRET` | Dex 客户端密钥 | - |
| `VISDATA_OPENFGA_URL` | OpenFGA API URL | http://localhost:8080 |
| `VISDATA_ENCRYPTION_KEY` | 加密密钥 | - |
| `VISDATA_CACHE_TTL` | 缓存 TTL | 300 |

---

## 12. 错误处理设计

### 12.1 错误码定义

**Dex 错误** (`src/dex/error.rs`):
```rust
pub enum Error {
    InvalidCredentials(String),    // 401
    InvalidToken(String),          // 401
    TokenExpired,                  // 401
    UserNotFound(String),          // 404
    ConnectorNotFound(String),     // 404
    ConnectorExists(String),       // 409
    InvalidConnector(String),      // 400
    GrpcError(String),             // 500
    HttpError(String),             // 500
    ConfigError(String),           // 500
    Internal(String),              // 500
}
```

**OpenFGA 错误** (`src/openfga/error.rs`):
```rust
pub enum Error {
    NotInitialized(String),        // 500
    OpenFGA(String),               // 500
    Http(reqwest::Error),          // 500
    StoreNotFound,                 // 404
    ModelNotFound,                 // 404
    RoleNotFound(String),          // 404
    GroupNotFound(String),         // 404
    UserNotFound(String),          // 404
    PermissionDenied(String),      // 403
    InvalidPermission(String),     // 400
    InvalidResourceType(String),   // 400
    DuplicateEntry(String),        // 409
    Validation(String),            // 400
    Serialization(String),         // 500
    Config(String),                // 500
    Internal(String),              // 500
}
```

### 12.2 错误响应格式

```json
{
  "status": "error",
  "code": "PERMISSION_DENIED",
  "message": "You do not have permission to access this resource",
  "details": {
    "resource": "dashboard:123",
    "required_permission": "can_read"
  }
}
```

### 12.3 异常处理策略

```rust
pub async fn handle_request(req: Request) -> Result<Response, Error> {
    match process_request(req).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // 记录错误日志
            log::error!("Request failed: {:?}", e);

            // 记录审计日志
            audit_log::record_failure(&req, &e).await;

            // 返回适当的 HTTP 响应
            Ok(e.into_response())
        }
    }
}
```

---

## 13. 测试设计

### 13.1 单元测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_token_success() {
        let token = create_test_token();
        let result = verify_token(&token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_token_expired() {
        let token = create_expired_token();
        let result = verify_token(&token).await;
        assert!(matches!(result, Err(Error::TokenExpired)));
    }

    #[tokio::test]
    async fn test_permission_check_root_user() {
        let result = is_allowed("root@example.com", "DELETE", "org:*").await;
        assert!(result.unwrap());
    }
}
```

### 13.2 集成测试策略

```rust
#[cfg(test)]
mod integration_tests {
    use testcontainers::{Docker, images};

    #[tokio::test]
    async fn test_full_login_flow() {
        // 启动测试容器
        let docker = Docker::new();
        let dex = docker.run(images::Dex::default());
        let openfga = docker.run(images::OpenFGA::default());

        // 初始化 Visdata
        let config = create_test_config(&dex, &openfga);
        Visdata::init_enterprise(config).await.unwrap();

        // 测试登录流程
        let response = login("test@example.com", "password").await;
        assert!(response.is_ok());

        // 验证权限
        let allowed = check_permission("test@example.com", "GET", "dashboard:1").await;
        assert!(allowed);
    }
}
```

### 13.3 性能测试策略

```rust
// 使用 criterion 进行基准测试
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_permission_check(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("permission_check_cached", |b| {
        b.iter(|| {
            rt.block_on(async {
                check_permission("user@example.com", "GET", "dashboard:1").await
            })
        })
    });
}

criterion_group!(benches, bench_permission_check);
criterion_main!(benches);
```

### 13.4 安全测试策略

| 测试类型 | 工具 | 覆盖范围 |
|----------|------|----------|
| 漏洞扫描 | OWASP ZAP | API 端点 |
| 依赖审计 | cargo-audit | Rust 依赖 |
| 密码强度 | 自定义规则 | 用户密码 |
| Token 安全 | JWT 测试套件 | Token 生成和验证 |

---

## 14. 部署设计

### 14.1 部署拓扑

见 3.3 节部署架构图。

### 14.2 容器化部署

**Dockerfile：**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features visdata

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/openobserve /usr/local/bin/
ENTRYPOINT ["openobserve"]
```

**docker-compose.yml：**
```yaml
version: '3.8'
services:
  visdata:
    image: visdata/openobserve:latest
    ports:
      - "5080:5080"
    environment:
      - VISDATA_DEX_CLIENT_SECRET=${DEX_CLIENT_SECRET}
      - VISDATA_OPENFGA_URL=http://openfga:8080
    depends_on:
      - dex
      - openfga
      - postgres

  dex:
    image: dexidp/dex:v2.37.0
    ports:
      - "5556:5556"
      - "5557:5557"
    volumes:
      - ./dex-config.yaml:/etc/dex/config.yaml

  openfga:
    image: openfga/openfga:v1.3.0
    ports:
      - "8080:8080"
    environment:
      - OPENFGA_DATASTORE_ENGINE=postgres
      - OPENFGA_DATASTORE_URI=postgres://postgres:password@postgres:5432/openfga

  postgres:
    image: postgres:14
    environment:
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### 14.3 配置管理

**Kubernetes ConfigMap：**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: visdata-config
data:
  config.toml: |
    [dex]
    issuer_url = "https://dex.example.com"
    client_id = "openobserve"

    [openfga]
    api_url = "http://openfga:8080"
    enabled = true
```

**Kubernetes Secret：**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: visdata-secrets
type: Opaque
stringData:
  dex-client-secret: "your-secret-here"
  encryption-key: "your-32-byte-key-here"
```

### 14.4 监控与告警

**Prometheus 指标：**
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref AUTH_REQUESTS: Counter = register_counter!(
        "visdata_auth_requests_total",
        "Total number of authentication requests"
    ).unwrap();

    static ref AUTH_FAILURES: Counter = register_counter!(
        "visdata_auth_failures_total",
        "Total number of authentication failures"
    ).unwrap();

    static ref AUTHZ_LATENCY: Histogram = register_histogram!(
        "visdata_authz_latency_seconds",
        "Authorization check latency in seconds"
    ).unwrap();
}
```

**Grafana 告警规则：**
```yaml
groups:
  - name: visdata-auth
    rules:
      - alert: HighAuthFailureRate
        expr: rate(visdata_auth_failures_total[5m]) / rate(visdata_auth_requests_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High authentication failure rate

      - alert: SlowAuthzChecks
        expr: histogram_quantile(0.95, rate(visdata_authz_latency_seconds_bucket[5m])) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: Authorization checks are slow
```

---

## 15. 附录

### 15.1 OpenFGA 完整授权模型

见需求文档附录 10.1。

### 15.2 时序图

#### 登录时序图

```
┌──────┐          ┌─────────┐          ┌─────┐          ┌───────┐
│Client│          │ Visdata │          │ Dex │          │OpenFGA│
└──┬───┘          └────┬────┘          └──┬──┘          └───┬───┘
   │  POST /auth/login │                  │                 │
   │──────────────────▶│                  │                 │
   │                   │ VerifyPassword   │                 │
   │                   │─────────────────▶│                 │
   │                   │                  │                 │
   │                   │◀─────────────────│                 │
   │                   │ (verified)       │                 │
   │                   │                  │                 │
   │                   │ GetUserPermissions                 │
   │                   │─────────────────────────────────────▶
   │                   │                  │                 │
   │                   │◀─────────────────────────────────────
   │                   │ (permissions)    │                 │
   │                   │                  │                 │
   │ 200 OK + Cookie   │                  │                 │
   │◀──────────────────│                  │                 │
```

### 15.3 状态图

#### 会话状态

```
                    ┌─────────┐
                    │  Init   │
                    └────┬────┘
                         │ login
                         ▼
                    ┌─────────┐
         refresh    │ Active  │◀───────┐
         ┌─────────▶│         │────────┘
         │          └────┬────┘  access
         │               │
         │               │ logout / expire
         │               ▼
         │          ┌─────────┐
         └──────────│ Expired │
                    └─────────┘
```

### 15.4 配置示例

见第 11 节。

### 15.5 API 详细规格

见需求文档附录 10.3。

---

## 文档结束
