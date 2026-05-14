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
3. 异步执行所有处理程序
4. 收集结果并处理错误

### 配置管理设计

#### 配置文件格式

```toml
[hooks]
enabled = true

[[hooks.handlers]]
name = "commit-notifier"
type = "post-commit"
script_path = "~/.sourcesvn/hooks/commit-notifier.js"
enabled = true

[[hooks.handlers]]
name = "update-logger"
type = "post-update"
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

#### 错误通知

- Hook执行失败时，通过toast通知用户
- 错误详情记录到日志文件
- 不阻塞主流程

### 图形界面配置设计

#### 配置界面布局

- 左侧：Hook列表（显示所有已配置的hook）
- 右侧：Hook详情编辑区
- 底部：操作按钮（添加、删除、保存）

#### Hook列表

- 显示hook名称、类型、状态（启用/禁用）
- 支持拖拽排序
- 支持搜索过滤

#### Hook详情编辑区

- 名称输入框
- 类型下拉选择框
- 脚本路径选择（文件选择对话框）
- 启用/禁用开关
- 高级选项（超时时间、重试次数等）

#### 操作流程

- 添加hook：点击添加按钮，填写详情，保存
- 编辑hook：选择hook，修改详情，保存
- 删除hook：选择hook，确认删除
- 启用/禁用：点击开关切换状态

### 扩展性设计

#### Hook脚本支持

- 支持JavaScript/TypeScript脚本
- 支持外部可执行文件
- 支持内置Rust处理程序

#### 脚本执行环境

- 提供API访问SVN操作
- 提供API访问应用配置
- 提供API访问日志记录

### 性能设计

#### 异步执行

- 所有hook异步执行，不阻塞主流程
- UI显示加载状态
- 支持超时控制

#### 并发控制

- 同一仓库的hook顺序执行
- 不同仓库的hook并行执行
- 支持最大并发数限制

#### 资源管理

- 限制hook执行时间
- 限制hook内存使用
- 限制hook CPU使用

#### 缓存机制

- 缓存hook配置
- 缓存脚本执行环境
- 缓存日志记录

### 安全性设计

#### 脚本执行安全

- 沙箱环境执行脚本
- 限制文件系统访问
- 限制系统调用

#### 权限控制

- hook只能访问授权的资源
- hook不能修改系统文件
- hook不能访问敏感信息

#### 输入验证

- 验证hook配置参数
- 验证脚本路径
- 验证脚本内容

#### 错误隔离

- hook执行失败不影响主流程
- hook异常被捕获并记录
- hook超时被强制终止

## 实现计划

### 第一阶段：基础框架

1. 定义核心类型和接口
2. 实现事件总线基础架构
3. 实现配置管理
4. 实现日志记录

### 第二阶段：Hook点实现

1. 实现提交相关hook点
2. 实现更新相关hook点
3. 实现文件状态相关hook点
4. 实现其他hook点

### 第三阶段：脚本执行

1. 实现JavaScript/TypeScript脚本执行环境
2. 实现外部可执行文件执行
3. 实现内置Rust处理程序

### 第四阶段：图形界面

1. 实现hook配置界面
2. 实现hook管理界面
3. 实现hook调试界面

## 总结

本设计基于事件总线架构，支持异步执行、通知/日志记录和拦截/修改操作，为未来的plugin计划奠定基础。设计遵循简单扩展的原则，提供基本的扩展接口，便于后续开发。