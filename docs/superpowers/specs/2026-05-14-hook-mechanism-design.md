# Hook机制设计文档

## 概述

本文档描述SourceSvn的hook机制设计，基于事件总线架构，支持异步执行、通知/日志记录和拦截/修改操作，为未来的plugin计划奠定基础。

## 设计目标

1. **异步执行**：hook在后台异步执行，不阻塞主流程，UI显示加载状态
2. **双模功能**：支持通知/日志记录和拦截/修改操作
3. **图形界面配置**：通过图形界面配置hook，支持选择配置文件路径
4. **日志记录调试**：通过日志记录hook的执行情况
5. **错误通知**：hook失败时通知用户，但不阻塞主流程
6. **简单扩展**：提供基本的扩展接口，便于后续开发

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────┐
│                 事件总线                      │
│  ┌──────────────────────────────────────┐    │
│  │        事件分发器                    │    │
│  └──────────────────────────────────────┘    │
│                    │                         │
│  ┌─────────────────┼───────────────────┐    │
│  │           Hook处理程序               │    │
│  │                                      │    │
│  │  ┌──────────┐ ┌──────────┐ ┌───────┐│    │
│  │  │提交前hook│ │更新后hook│ │状态hook││    │
│  │  └──────────┘ └──────────┘ └───────┘│    │
│  └──────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

### Hook点设计

1. **提交相关Hook点**
   - `pre-commit`: 提交前触发，可以修改提交信息、添加文件、取消提交
   - `post-commit`: 提交后触发，用于通知、日志记录

2. **更新相关Hook点**
   - `pre-update`: 更新前触发，可以取消更新
   - `post-update`: 更新后触发，用于通知、日志记录

3. **文件状态相关Hook点**
   - `status-change`: 文件状态变更时触发
   - `conflict-detected`: 冲突发生时触发

4. **其他Hook点**
   - `pre-checkout`: 检出前触发
   - `post-checkout`: 检出后触发
   - `pre-merge`: 合并前触发
   - `post-merge`: 合并后触发

### Hook处理程序设计

#### Hook处理程序接口

```rust
#[async_trait]
pub trait HookHandler: Send + Sync {
    async fn execute(&self, context: &HookContext) -> HookResult;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
```

#### Hook上下文

```rust
pub struct HookContext {
    pub hook_type: HookType,
    pub repo_path: String,
    pub data: HashMap<String, Value>,
    pub timestamp: DateTime<Utc>,
}
```

#### Hook结果

```rust
pub enum HookResult {
    Continue,
    Cancel,
    Modify(HashMap<String, Value>),
}
```

#### Hook类型

```rust
pub enum HookType {
    PreCommit,
    PostCommit,
    PreUpdate,
    PostUpdate,
    StatusChange,
    ConflictDetected,
    PreCheckout,
    PostCheckout,
    PreMerge,
    PostMerge,
}
```

### 事件总线设计

#### 事件总线接口

```rust
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn emit(&self, event: HookEvent);
    fn subscribe(&self, hook_type: HookType, handler: Box<dyn HookHandler>);
    fn unsubscribe(&self, hook_type: HookType, handler_name: &str);
}
```

#### 事件

```rust
pub struct HookEvent {
    pub hook_type: HookType,
    pub context: HookContext,
    pub timestamp: DateTime<Utc>,
}
```

#### 事件总线实现

```rust
pub struct DefaultEventBus {
    handlers: HashMap<HookType, Vec<Box<dyn HookHandler>>>,
    logger: Arc<dyn Logger>,
}
```

#### 事件分发流程

1. 事件总线接收事件
2. 根据hook_type找到对应的处理程序列表
3. 按配置顺序异步执行所有处理程序
4. 收集结果并处理错误

### 配置管理设计

#### 配置文件格式

```toml
[hooks]
enabled = true

[[hooks.handlers]]
name = "commit-notifier"
type = "PostCommit"
script_path = "~/.sourcesvn/hooks/commit-notifier.js"
enabled = true

[[hooks.handlers]]
name = "update-logger"
type = "PostUpdate"
script_path = "~/.sourcesvn/hooks/update-logger.js"
enabled = true
```

#### 配置结构

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct HooksConfig {
    pub enabled: bool,
    pub handlers: Vec<HookHandlerConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HookHandlerConfig {
    pub name: String,
    pub hook_type: HookType,
    pub script_path: String,
    pub enabled: bool,
}
```

#### 配置管理接口

```rust
pub trait HookConfigManager {
    fn load_config(&self) -> Result<HooksConfig>;
    fn save_config(&self, config: &HooksConfig) -> Result<()>;
    fn add_handler(&self, config: HookHandlerConfig) -> Result<()>;
    fn remove_handler(&self, name: &str) -> Result<()>;
    fn update_handler(&self, name: &str, config: HookHandlerConfig) -> Result<()>;
}
```

配置文件路径：`~/.sourcesvn/hooks.toml`

### 错误处理和日志记录设计

#### 错误处理

```rust
pub enum HookError {
    ExecutionFailed(String),
    TimeoutExpired,
    InvalidConfiguration(String),
    ScriptNotFound(String),
}

impl HookError {
    pub fn user_message(&self) -> String {
        match self {
            HookError::ExecutionFailed(msg) => format!("Hook执行失败: {}", msg),
            HookError::TimeoutExpired => "Hook执行超时".to_string(),
            HookError::InvalidConfiguration(msg) => format!("Hook配置错误: {}", msg),
            HookError::ScriptNotFound(path) => format!("Hook脚本未找到: {}", path),
        }
    }
}
```

#### 日志记录

```rust
pub trait Logger {
    fn log_hook_start(&self, hook_type: HookType, handler_name: &str);
    fn log_hook_end(&self, hook_type: HookType, handler_name: &str, duration: Duration);
    fn log_hook_error(&self, hook_type: HookType, handler_name: &str, error: &HookError);
    fn log_hook_cancel(&self, hook_type: HookType, handler_name: &str, reason: &str);
}
```

日志记录到文件：`~/.sourcesvn/logs/hooks.log`

#### 错误通知

- Hook执行失败时，通过toast通知用户（显示错误摘要）
- 错误详情记录到日志文件（包含完整堆栈信息）
- 不阻塞主流程（用户可以继续操作）

### 图形界面配置设计

#### 配置界面布局

- 左侧：Hook列表（显示所有已配置的hook，支持搜索过滤）
- 右侧：Hook详情编辑区（显示选中hook的详细信息）
- 底部：操作按钮（添加、删除、保存、测试）

#### Hook列表

- 显示hook名称、类型、状态（启用/禁用）
- 支持拖拽排序（调整hook执行顺序）
- 支持搜索过滤（按名称或类型过滤）

#### Hook详情编辑区

- 名称输入框（唯一标识符）
- 类型下拉选择框（PreCommit、PostCommit等）
- 脚本路径选择（文件选择对话框，支持JS/TS/可执行文件）
- 启用/禁用开关
- 高级选项（超时时间、重试次数、并发控制等）

#### 操作流程

- 添加hook：点击添加按钮，填写详情，保存
- 编辑hook：选择hook，修改详情，保存
- 删除hook：选择hook，确认删除
- 启用/禁用：点击开关切换状态
- 测试hook：点击测试按钮，执行hook并显示结果

### 扩展性设计

#### Hook脚本支持

- 支持JavaScript/TypeScript脚本（通过内置JS引擎执行）
- 支持外部可执行文件（直接调用系统命令）
- 支持内置Rust处理程序（编译时注册）

#### 脚本执行环境

- 提供API访问SVN操作
- 提供API访问应用配置
- 提供API访问日志记录
- JavaScript/TypeScript脚本在沙箱环境中执行
- 外部可执行文件在普通环境中执行

### 性能设计

#### 异步执行

- 所有hook异步执行，不阻塞主流程
- UI显示加载状态（进度条或加载动画）
- 支持超时控制（默认30秒，可配置）

#### 并发控制

- 同一仓库的hook顺序执行（避免冲突）
- 不同仓库的hook并行执行（提高效率）
- 支持最大并发数限制（默认10，可配置）

#### 资源管理

- 限制hook执行时间（默认30秒，可配置）
- 限制hook内存使用
- 限制hook CPU使用

#### 缓存机制

- 缓存hook配置（内存缓存，配置变更时刷新）
- 缓存脚本执行环境（JavaScript引擎实例）
- 缓存日志记录（批量写入磁盘）

### 安全性设计

#### 脚本执行安全

- 沙箱环境执行脚本（JavaScript/TypeScript脚本）
- 限制文件系统访问（只能访问工作副本和配置目录）
- 限制系统调用（禁止危险操作）

#### 权限控制

- hook只能访问授权的资源（工作副本、配置文件等）
- hook不能修改系统文件
- hook不能访问敏感信息（API密钥、密码等）

#### 输入验证

- 验证hook配置参数（名称、类型、路径等）
- 验证脚本路径（文件存在、可执行权限）
- 验证脚本内容（语法检查）

#### 错误隔离

- hook执行失败不影响主流程
- hook异常被捕获并记录
- hook超时被强制终止
- hook执行结果通过toast通知用户

## 实现计划

### 第一阶段：基础框架

1. 定义核心类型和接口（HookType、HookContext、HookResult等）
2. 实现事件总线基础架构（EventBus trait和DefaultEventBus）
3. 实现配置管理（HooksConfig和HookConfigManager）
4. 实现日志记录（Logger trait和文件日志）

### 第二阶段：Hook点实现

1. 实现提交相关hook点（pre-commit、post-commit）
2. 实现更新相关hook点（pre-update、post-update）
3. 实现文件状态相关hook点（status-change、conflict-detected）
4. 实现其他hook点（pre-checkout、post-checkout、pre-merge、post-merge）

### 第三阶段：脚本执行

1. 实现JavaScript/TypeScript脚本执行环境（内置JS引擎）
2. 实现外部可执行文件执行（系统命令调用）
3. 实现内置Rust处理程序（编译时注册）

### 第四阶段：图形界面

1. 实现hook配置界面（设置页面中的hook配置区域）
2. 实现hook管理界面（hook列表、详情编辑、测试）
3. 实现hook调试界面（日志查看、执行历史）

## 总结

本设计基于事件总线架构，支持异步执行、通知/日志记录和拦截/修改操作，为未来的plugin计划奠定基础。设计遵循简单扩展的原则，提供基本的扩展接口，便于后续开发。

### 主要特点

1. **事件总线架构**：松耦合，易于扩展
2. **异步执行**：不阻塞主流程，用户体验好
3. **双模功能**：支持通知/日志记录和拦截/修改操作
4. **图形界面配置**：简单易用，便于管理
5. **安全性设计**：沙箱环境，权限控制，输入验证
6. **性能优化**：并发控制，缓存机制，资源管理

### 未来扩展

1. 支持更多脚本语言（Python、Ruby等）
2. 支持远程hook（网络调用）
3. 支持hook链式执行（多个hook串联）
4. 支持插件系统（第三方插件开发）