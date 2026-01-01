# VisData - OpenObserve 企业版扩展模块

## 概述

VisData 是 OpenObserve 的企业版扩展模块，提供与官方企业版 API 兼容的 SSO 和 RBAC 功能：

- **OpenFGA 集成**: 基于 [OpenFGA](https://openfga.dev/) 的细粒度权限控制
- **Dex 集成**: 基于 [Dex](https://dexidp.io/) 的 SSO 单点登录

## 特性

| 功能 | 描述 |
|------|------|
| RBAC | 基于角色的访问控制，支持自定义角色和权限 |
| SSO | 支持 OIDC/LDAP 等多种认证协议 |
| API 兼容 | 与 OpenObserve 官方企业版 API 完全兼容 |
| 前端兼容 | 可直接使用官方企业版前端界面 |

## 架构

```
visdata/
├── lib.rs                    # 模块入口，全局单例 Visdata
├── config/                   # 配置模块
│   ├── mod.rs
│   ├── types.rs              # VisdataConfig, OIDCConfig, LDAPConfig
│   └── get_config.rs         # 从环境变量读取配置
│
├── openfga/                  # OpenFGA 授权模块 (兼容 o2_openfga)
│   ├── mod.rs
│   ├── config.rs             # OpenFGAConfig
│   ├── client.rs             # OpenFGAClient (HTTP)
│   ├── error.rs              # 错误类型
│   ├── types.rs              # TupleKey, TupleKeyFilter 等
│   ├── authorizer/           # 权限检查 API
│   │   ├── mod.rs
│   │   ├── authz.rs          # is_allowed, add_user_to_org 等
│   │   ├── roles.rs          # 角色管理 API
│   │   └── groups.rs         # 用户组管理 API
│   ├── service/              # 内部服务层
│   │   ├── mod.rs
│   │   ├── checker.rs        # 权限检查逻辑
│   │   ├── tuples.rs         # Tuple 操作
│   │   ├── roles.rs          # 角色服务
│   │   └── groups.rs         # 用户组服务
│   ├── model/                # FGA 模型定义
│   │   ├── mod.rs
│   │   ├── schema.rs         # 类型名称生成函数
│   │   └── resources.rs      # 资源类型定义
│   └── meta/                 # 元数据映射
│       ├── mod.rs
│       └── mapping.rs        # OFGA_MODELS 资源映射表
│
├── dex/                      # Dex 认证模块 (兼容 o2_dex)
│   ├── mod.rs
│   ├── config.rs             # DexConfig
│   ├── client.rs             # DexClient (HTTP/gRPC)
│   ├── error.rs              # 错误类型
│   ├── types.rs              # 请求/响应类型
│   ├── handler/              # HTTP 处理器
│   │   ├── mod.rs
│   │   ├── login.rs          # 登录处理
│   │   └── connectors.rs     # 连接器管理
│   ├── service/              # 服务层
│   │   ├── mod.rs
│   │   ├── token.rs          # Token 验证
│   │   └── connector.rs      # 连接器服务
│   └── meta/                 # 元数据类型
│       ├── mod.rs
│       └── auth.rs           # Permission, RoleRequest 等
│
├── enterprise/               # 企业版兼容层
│   └── common/
│       ├── mod.rs
│       └── config.rs         # 企业版配置工具
│
└── common/                   # 公共工具
    ├── mod.rs
    └── id.rs                 # KSUID 生成
```

## 快速开始

### 依赖配置

在 OpenObserve 的 `Cargo.toml` 中添加：

```toml
[dependencies]
visdata = { git = "https://github.com/visdata-com/o2_visdata.git", branch = "main" }

# 或使用本地路径开发
# visdata = { path = "../o2_visdata" }
```

### 环境变量

```bash
# ========== OpenFGA 配置 ==========
ZO_OPENFGA_URL=http://localhost:8080
ZO_OPENFGA_STORE_NAME=openobserve

# ========== Dex 配置 ==========
VISDATA_DEX_GRPC_URL=http://localhost:5557
VISDATA_DEX_ISSUER_URL=http://localhost:5556
VISDATA_DEX_CLIENT_ID=openobserve
VISDATA_DEX_CLIENT_SECRET=your-secret
VISDATA_DEX_REDIRECT_URI=http://localhost:5080/config/redirect

# ========== 功能开关 ==========
VISDATA_SSO_ENABLED=true
VISDATA_RBAC_ENABLED=true
```

### 初始化

```rust
use visdata::{Visdata, VisdataConfig};

// 从环境变量读取配置并初始化
let config = visdata::config::get_config();
Visdata::init_enterprise(config).await?;

// 使用全局实例
let visdata = Visdata::global();
let openfga = visdata.openfga();
let dex = visdata.dex();
```

## OpenFGA 权限模型

### 权限类型

| 权限 | 描述 | HTTP 方法 |
|------|------|-----------|
| AllowAll | 全部权限 | * |
| AllowList | 列表权限 | GET (list) |
| AllowGet | 读取权限 | GET |
| AllowPost | 创建权限 | POST |
| AllowPut | 更新权限 | PUT |
| AllowDelete | 删除权限 | DELETE |

### 资源类型

```rust
// 数据流
logs, metrics, traces, index, metadata

// 仪表板
dfolder (文件夹), dashboard, template, savedviews

// 告警
afolder (文件夹), alert, destination

// 报表
rfolder (文件夹), report

// 其他
function, pipeline, settings, kv, enrichment_table
passcode, rumtoken, service_accounts, cipher_keys
```

### 对象命名规则

```
格式: {resource_type}:{entity_id}

示例:
- logs:my_stream           # 具体日志流
- logs:_all_default        # 所有日志流 (default 组织)
- dashboard:my_dashboard   # 具体仪表板
- dfolder:_all_default     # 所有仪表板文件夹
```

### 组织隔离

组织隔离通过角色名称实现：

```
role:{org_id}_{role_name}

示例:
- role:default_admin       # default 组织的 admin 角色
- role:prod_viewer         # prod 组织的 viewer 角色
```

## API 兼容性

### authorizer 模块

与 `o2_openfga::authorizer` 兼容：

```rust
use visdata::openfga::authorizer::{authz, roles, groups};

// 权限检查
authz::is_allowed(org_id, user_id, permission, object_type, object_id, role).await?;

// 角色管理
roles::create_role(org_id, role_name).await?;
roles::update_role(org_id, role_name, add_perms, remove_perms, add_users, remove_users).await?;
roles::get_role_permissions(org_id, role_name, resource_type).await?;

// 用户组管理
groups::create_group(org_id, group_name).await?;
groups::get_user_roles(org_id, user_email).await?;
```

### meta 模块

与 `o2_openfga::meta` 兼容：

```rust
use visdata::openfga::meta::mapping::{OFGA_MODELS, NON_CLOUD_RESOURCE_KEYS, Resource};

// 获取资源定义
let logs = OFGA_MODELS.get("logs");

// 检查是否为云端资源
let is_non_cloud = NON_CLOUD_RESOURCE_KEYS.contains("license");
```

### auth 模块

与 `o2_dex` 兼容：

```rust
use visdata::dex::meta::auth::{Permission, RoleRequest, O2EntityAuthorization};

// 或使用别名
use visdata::auth::meta::auth::{Permission, RoleRequest};
```

## 系统角色

以下为系统内置角色，不可删除：

| 角色 | 描述 |
|------|------|
| admin | 管理员，拥有所有权限 |
| editor | 编辑者，可创建和修改资源 |
| viewer | 查看者，只读权限 |

## 前端配置

在 OpenObserve Web 前端启用企业版功能：

```bash
# web/.env
VITE_OPENOBSERVE_ENTERPRISE=true
```

## 依赖服务

### OpenFGA

```bash
# Docker 部署
docker run -d --name openfga \
  -p 8080:8080 \
  -p 3000:3000 \
  openfga/openfga run
```

### Dex

```bash
# Docker 部署
docker run -d --name dex \
  -p 5556:5556 \
  -p 5557:5557 \
  -v /path/to/config.yaml:/etc/dex/config.yaml \
  ghcr.io/dexidp/dex:latest serve /etc/dex/config.yaml
```

## 开发

### 本地开发

```bash
# 切换到本地依赖
# 在 openobserve/Cargo.toml 中:
visdata = { path = "../o2_visdata" }

# 编译
cargo build

# 运行测试
cargo test
```

### 日志调试

```bash
# .env
RUST_LOG=info,openobserve=debug,visdata=debug
```

## License

GNU Affero General Public License v3.0 (AGPL-3.0)

Copyright 2025 VisData Inc.
