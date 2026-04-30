# SourceSvn 架构设计文档（第三版）

## 一、整体架构

```
┌─────────────────────────────────────────────┐
│                 前端 (Vue 3)                 │
│  ┌──────────┐ ┌──────────┐ ┌─────────────┐  │
│  │ 视图层   │ │ 路由/状态 │ │ 组件库      │  │
│  └─────┬────┘ └─────┬────┘ └──────┬──────┘  │
│        │           │              │         │
│        └───────────┴──────────────┘         │
│                    │ invoke / listen        │
└────────────────────┼───────────────────────┘
                     │
┌────────────────────┼───────────────────────┐
│                 Tauri 桥接层                │
│  ┌─────────────────┴───────────────────┐    │
│  │       命令路由、事件转发            │    │
│  └─────────────────┬───────────────────┘    │
│                    │                        │
│  ┌─────────────────┼───────────────────┐    │
│  │           后端核心 (Rust)            │    │
│  │                                      │    │
│  │  ┌──────────┐ ┌──────────┐ ┌───────┐│    │
│  │  │SVN 服务  │ │AI 服务   │ │配置   ││    │
│  │  │模块      │ │模块      │ │模块   ││    │
│  │  └────┬─────┘ └────┬─────┘ └───┬───┘│    │
│  │       │            │           │     │    │
│  │  ┌────┴─────┐ ┌────┴─────┐ ┌───┴───┐│    │
│  │  │Shelve    │ │HTTP 客户端│ │存储   ││    │
│  │  │管理      │ │(reqwest) │ │(confy)││    │
│  │  └──────────┘ └──────────┘ └───────┘│    │
│  └──────────────────────────────────────┘    │
│                    │                         │
│  ┌─────────────────┴───────────────────┐    │
│  │       系统调用 (svn 命令)            │    │
│  └──────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

**配置生命周期说明**  
- 启动时：Rust 在 `tauri::Builder::setup` 中加载配置，应用窗口几何与主题，并将全量配置通过 `get_config` 命令返回给前端。  
- 关闭时：Rust 通过 Tauri v2 的 `window.on_window_event` 捕获 `WindowEvent::CloseRequested`，保存窗口尺寸、位置、最大化状态。同时前端监听 `tauri://close-requested` 事件，调用 `set_config` 更新 `open_tabs` 与 `active_tab_index`，确保页签状态被持久化。

---

## 二、后端模块设计

后端采用**无状态设计**：每个命令均要求传入工作副本路径，不持有任何仓库级上下文。切换仓库时前端传入不同路径即可，无需后端切换。

### 2.1 SVN 服务模块

**职责**：封装所有 `svn` 命令行调用，将原始输出（XML/文本）解析为结构化数据。

**内部接口**：

- `run_svn(args: &[&str], timeout: Duration) -> Result<Output>` — 底层命令执行，统一处理超时、错误。
- `parse_status_xml(xml: &str) -> Result<Vec<FileStatus>>`
- `parse_log_xml(xml: &str) -> Result<Vec<LogEntry>>`
- `parse_list_xml(xml: &str) -> Result<Vec<DirEntry>>`
- `parse_info_xml(xml: &str) -> Result<RepoInfo>`

### 2.2 AI 服务模块

**职责**：管理与 AI 提供商的通信，构建 prompt，支持流式/非流式响应。

**扩展设计**：定义 `AiProvider` trait，支持多后端可插拔切换。MVP 阶段实现 OpenAI 兼容 API Provider，后续可通过配置切换 provider。

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_message(&self, diff: &str, config: &AIConfig) -> Result<String>;
    async fn review_changes(&self, diff: &str, config: &AIConfig, emitter: Emitter) -> Result<()>;
    fn provider_type(&self) -> ProviderType;
}

pub enum ProviderType { OpenAI, Local, Claude, Custom }
```

**接口**：

- `async fn generate_message(diff: &str, config: &AIConfig) -> Result<String>`
- `async fn review_changes(diff: &str, config: &AIConfig, app_handle: tauri::AppHandle) -> Result<()>` — 内部通过 `app_handle.emit("review_chunk", payload)` 推送流式分块，命令返回 `Ok(())` 表示审查启动完成。
- 依赖 `reqwest` 异步 HTTP 客户端，支持代理、超时、重试。
- 配置项 `ai_provider` 选择当前使用的 provider。

### 2.3 Shelve 管理模块

**职责**：在全局存储位置管理补丁文件，避免污染工作副本。

**存储路径**：`~/.sourcesvn/shelves/<repo_hash>/`，`repo_hash` 基于工作副本绝对路径计算（如 SHA256 前 12 位）。

**接口**：

- `save_shelve(repo_path: &Path, name: &str) -> Result<()>` — 保存整个工作副本当前所有未提交的修改（`svn diff` 全量）。
- `list_shelves(repo_path: &Path) -> Result<Vec<ShelveInfo>>`
- `apply_shelve(repo_path: &Path, name: &str) -> Result<()>` — 从全局 shelve 目录恢复补丁。
- `delete_shelve(repo_path: &Path, name: &str) -> Result<()>`

### 2.4 配置管理模块

**职责**：使用 `confy` 库持久化应用配置（基于 `serde` + `toml`）。

**接口**：

- `load_config() -> AppConfig`
- `save_config(config: &AppConfig) -> Result<()>`

配置文件存储于系统标准配置目录。

**完整配置项**：

| 分类 | 配置项 | 类型 | 默认值 | 说明 |
|------|--------|------|--------|------|
|  | `config_version` | `u32` | `1` | 配置版本号，用于迁移 |
| **窗口** | `window_width` | `u32` | `1200` | 窗口宽度（px） |
| | `window_height` | `u32` | `800` | 窗口高度（px） |
| | `window_x` | `Option<i32>` | `None`（居中） | 窗口 X 坐标 |
| | `window_y` | `Option<i32>` | `None`（居中） | 窗口 Y 坐标 |
| | `window_maximized` | `bool` | `false` | 启动时是否最大化 |
| **外观** | `theme` | `String` | `"light"` | 主题：`"light"` / `"dark"` |
| | `ui_font_family` | `String` | 系统默认 sans-serif | UI 字体（菜单、标签等） |
| | `ui_font_size` | `u32` | `14` | UI 字体大小（px） |
| | `code_font_family` | `String` | `"monospace"` | 代码/差异字体 |
| | `code_font_size` | `u32` | `13` | 代码字体大小（px） |
| | `icon_size` | `u32` | `20` | 图标大小（px） |
| **会话** | `open_tabs` | `Vec<TabInfo>` | `[]` | 上次打开的仓库页签，启动时自动恢复（路径无效则跳过并提示） |
| | `active_tab_index` | `usize` | `0` | 当前选中页签索引 |
| | `recent_repos` | `Vec<RepoEntry>` | `[]` | 仓库历史记录（上限 N 条，FIFO。路径失效的条目打开时提示允许删除） |
| | `max_recent_repos` | `usize` | `10` | 仓库历史记录上限 |
| **SVN** | `svn_executable` | `Option<String>` | `None` | 自定义 SVN 可执行文件路径 |
| **AI** | `ai_endpoint` | `String` | `"https://api.openai.com/v1"` | AI API 端点 |
| | `ai_api_key` | `String` | `""` | API 密钥，空字符串禁用 AI 功能 |
| | `ai_model` | `String` | `"gpt-4o-mini"` | 模型名称 |
| | `ai_timeout_secs` | `u64` | `30` | AI 请求超时（秒） |
| **差异** | `diff_context_lines` | `u32` | `3` | 差异上下文行数 |
| | `diff_ignore_whitespace` | `bool` | `false` | 忽略空白变更 |
| | `diff_view_mode` | `String` | `"unified"` | 差异视图模式：`"unified"` / `"side_by_side"` |
| **日志** | `log_fetch_limit` | `u32` | `100` | 单次拉取日志条数上限 |
| | `log_show_changed_paths` | `bool` | `true` | 日志列表是否默认展开变更路径 |
| **提交** | `commit_template` | `Option<String>` | `None` | 提交信息模板，MVP 暂不支持变量（预留）。 |
| **文件浏览** | `file_browser_show_hidden` | `bool` | `false` | 文件浏览是否显示隐藏文件 |
| **行为** | `confirm_before_commit` | `bool` | `true` | 提交前是否弹出确认 |
| | `confirm_before_revert` | `bool` | `true` | 还原前是否弹出确认 |
| | `auto_refresh_secs` | `Option<u64>` | `None` | 自动刷新工作副本状态间隔（秒） |
| **高级** | `svn_timeout_secs` | `u64` | `60` | SVN 命令超时（秒） |
| | `log_level` | `String` | `"warn"` | 日志级别：`"error"` / `"warn"` / `"info"` / `"debug"` |

**关联类型**：

```rust
#[derive(Serialize, Deserialize, Clone)]
struct TabInfo {
    repo_path: String,
    active_view: ActiveView,    // 枚举，见共享类型
}

#[derive(Serialize, Deserialize, Clone)]
struct RepoEntry {
    path: String,
    last_opened: String,       // ISO 8601 timestamp
}
```

**配置读写流程**：

- **启动时**：Rust `setup` 中调用 `load_config`，检测 `config_version`，若版本不匹配则执行迁移（保留旧值，新项用默认值），写入新版本号后持久化。然后应用窗口尺寸、位置、最大化状态，并调用 `get_config` 将全量配置传给前端。前端根据 `open_tabs` 恢复页签（跳过无效路径并通知用户）。
- **运行时**：用户修改设置后，前端调用 `set_config`，Rust 立即持久化并同步 store。
- **关闭时**：Rust 通过 `window.on_window_event(WindowEvent::CloseRequested)` 捕获窗口几何并保存；前端在 `window.addEventListener('tauri://close-requested', ...)` 中调用 `set_config`，将当前 `open_tabs` 与 `active_tab_index` 保存。

**配置迁移策略**：定义 `ConfigMigration` trait，每个版本升级时注册迁移函数。迁移时保留用户原有值，新增配置项填入默认值，重命名或删除的配置项做相应转换。迁移后提示用户“配置文件已升级”。

### 2.5 共享类型模块 (`common`)

**职责**：定义跨模块数据结构、错误类型、统一枚举。

**核心类型**：

- `FileStatus`：文件路径、状态类型（Modified/Added/Deleted/Unversioned 等）。
- `LogEntry`：版本号、作者、日期、消息、变更路径列表。
- `RepoInfo`：仓库 URL、根路径、当前版本号等。
- `DirEntry`：目录条目（文件/目录，版本号，作者，日期）。
- `AIConfig`：AI 端点、Key、模型、超时等。
- `AppConfig`：全局配置结构体（含上述所有配置项）。
- `ActiveView` 枚举：`Log`, `LocalChanges`, `FileBrowser`, `Shelve`。
- `DiffTarget` 枚举：
  ```rust
  enum DiffTarget {
      /// 比较工作副本的本地修改（相对BASE）。revision为None表示与BASE比较，Some(rev)表示与指定版本比较。
      File { path: String, revision: Option<String> },
      /// 比较两个版本间的差异
      Revisions { old: String, new: String },
  }
  ```
- `Error`：自定义错误枚举，涵盖 SVN 错误、IO 错误、AI 错误等。统一转换为字符串返给前端。
- `ShelveInfo`：名称、创建日期、来源路径等。

---

## 三、错误处理与并发策略

### 3.1 统一错误处理

所有后端命令返回 `Result<T, String>`。错误分类：

| 错误类型 | 用户提示 | 日志记录 |
|----------|----------|----------|
| SVN 命令超时 | "操作超时，请检查网络或仓库大小" | 记录超时参数 |
| SVN 未安装或路径错误 | "未找到 SVN 命令行工具，请安装 SVN 客户端或配置路径" | 记录尝试的路径 |
| SVN 认证失败 | "认证失败，请检查凭据" | 不记录敏感信息 |
| 工作副本锁定 | "工作副本被锁定，请运行 svn cleanup" | 记录仓库路径 |
| AI 服务不可用 | "AI 服务连接失败，请检查网络或 API 配置" | 记录端点 URL（不含 key） |
| 文件系统错误 | "无法访问文件或目录" | 记录路径和错误码 |

前端统一使用 try/catch，错误通过 toast 通知栏展示（自动 5 秒消失），严重错误可点击查看详情。所有错误写入日志文件（`~/.sourcesvn/logs/error.log`）。

### 3.2 并发请求与仓库级锁

不同仓库的 SVN 命令可并行执行。同一仓库的命令需排队，避免冲突：

```rust
struct RepoLockManager {
    locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

impl RepoLockManager {
    async fn execute_with_lock<F, T>(&self, repo_path: &str, f: F) -> T
    where F: Future<Output = T>,
    {
        let mutex = self.get_or_create_lock(repo_path).await;
        let _guard = mutex.lock().await;
        f.await
    }
}
```

前端调用时显示 loading 状态（转圈），若同一仓库排队超过 10 秒，提示用户“正在处理上一个操作，请稍候”。

### 3.3 AI 请求重试策略

AI 请求失败时自动重试：
- 最大重试次数：3 次
- 退避策略：指数退避（1s → 2s → 4s）
- 可重试的错误：网络超时、5xx 服务端错误
- 不重试的错误：4xx 客户端错误（如 API Key 无效）

重试仍失败时，提示用户并建议检查网络或 API 配置。

## 四、Tauri 桥接层命令列表

| 命令名称 | 参数 | 返回 | 描述 |
|----------|------|------|------|
| `svn_status` | `path: String` | `Vec<FileStatus>` | 获取工作副本文件状态列表 |
| `svn_info` | `path: String` | `RepoInfo` | 获取仓库基本信息 |
| `svn_log` | `path: String, limit: Option<u32>, from_rev: Option<String>` | `Vec<LogEntry>` | 获取提交历史 |
| `svn_diff` | `path: String, target: DiffTarget` | `String` | 获取 unified diff |
| `svn_commit` | `path: String, message: String, files: Vec<String>` | `CommitResult` | 执行提交 |
| `svn_list` | `path: String, revision: Option<String>, recursive: bool` | `Vec<DirEntry>` | 列出仓库目录内容，支持指定版本 |
| `svn_cat` | `path: String, revision: Option<String>` | `String` | 获取指定版本的文件内容 |
| `svn_checkout` | `url: String, dest: String` | `()` | 检出仓库 |
| `svn_update` | `path: String` | `UpdateResult` | 更新工作副本 |
| `generate_commit_message` | `diff: String` | `String` | AI 生成提交信息 |
| `review_changes` | `diff: String` | `()` (事件流 `review_chunk`) | AI 代码审查，结果通过事件流推送；后端使用 `app_handle.emit` |
| `shelve_save` | `path: String, name: String` | `()` | 保存当前工作副本所有修改为 shelve（全局存储） |
| `shelve_list` | `path: String` | `Vec<ShelveInfo>` | 列出该工作副本的所有 shelve |
| `shelve_apply` | `path: String, name: String` | `()` | 恢复某个 shelve |
| `shelve_delete` | `path: String, name: String` | `()` | 删除某个 shelve |
| `get_config` | 无 | `AppConfig` | 读取全量配置 |
| `set_config` | `config: AppConfig` | `()` | 保存全量配置 |

**错误处理约定**：所有命令返回 `Result<T, String>`。前端使用 `try/catch`，错误信息通过 toast 或通知栏提示（MVP 使用轻量 toast）。特殊错误（如 SVN 未安装）单独给出明确指引。

**流式审查调用流程**：
1. 前端监听 `review_chunk` 事件。
2. 调用 `invoke("review_changes", { diff })`。
3. 后端开始流式请求并逐块 `emit("review_chunk", { text, done })`。
4. 前端收集并渲染，收到 `done` 后完成。

---

## 五、前端架构设计

### 4.1 组件树

```
App.vue
├── GlobalTabBar.vue          — 顶部页签管理，全局设置按钮（打开设置模态框）
├── Toolbar.vue               — 仓库级操作按钮（拉取/提交/刷新等）
├── IconNavBar.vue            — 极窄图标导航（36-48px），4个图标对应 ActiveView
└── <component :is="currentView"> — 右侧功能视图区，仅一个实例，切换页签时刷新数据
    ├── LogView.vue
    ├── LocalChangesView.vue
    ├── FileBrowserView.vue   — 支持通过版本选择器切换版本
    └── ShelveView.vue
```

**设置页面**：通过全局设置按钮以 **模态对话框** 方式打开，不离开主界面。模态框内包含页签式配置区域（窗口、外观、AI 等）。修改后即时调用 `set_config` 保存并生效。

### 4.2 状态管理

- **全局 store**（pinia）：存储 `AppConfig`，提供主题、字体等全局设置响应式变量。
- **页签 store**：使用 `defineStore` 结合动态 id（例如 `repo-${index}`），为每个打开的页签创建独立 store：
  - `repoPath: string`
  - `activeView: ActiveView`
  - 当前视图所需缓存数据：日志列表、文件树展开节点等
- **生命周期**：
  - 打开页签 → 创建 store（页面跳转至新增页签并激活）。
  - 关闭页签 → `$dispose()` 销毁 store，释放内存。
- **视图刷新策略**：MVP 阶段采用“**切换即请求**”——每次切换视图时重新调用后端 API 获取最新数据（操作简单、数据可靠）。后续可引入手动刷新和智能缓存。
- **无效页签恢复**：启动时若 `open_tabs` 中的路径不存在或不是有效 SVN 工作副本，跳过该页签，并弹出提示“路径 xxx 不存在或非仓库，已移除”。

### 4.3 数据流

1. **请求-响应**：`invoke("command", params)` → 成功返回数据更新页签 store，失败 toast 提示。
2. **服务端推送（AI 审查）**：Rust `emit` 事件 → 前端 `listen` 接收，视图增量更新。
3. **配置加载与保存**：
   - 启动：`get_config` → 初始化全局 store → 恢复页签。
   - 关闭：`tauri://close-requested` 事件 → `set_config({ open_tabs, active_tab_index })`，与 Rust 保存窗口状态并行。
4. **仓库操作**：每个命令均以 `path` 为参数，天然隔离多仓库。

---

## 六、关键设计决策

- **前端单实例视图**：右侧功能区只有一个动态组件，切换页签仅刷新数据，节约内存。
- **后端无状态**：所有业务函数纯由参数驱动，易于测试，避免并发冲突。
- **配置持久化**：使用 `confy`（`serde` + `toml`），窗口状态与页签状态的保存分别在前后端各自完成的关闭事件中。
- **Shelve 方案**：基于补丁文件，存储在全局目录避免污染工作副本。
- **扩展性**：自定义功能（如关联单号）通过未来扩展系统实现，不侵入核心。
- **错误处理**：统一 `Result<T, String>` + toast，保证体验一致。
- **视图数据更新**：MVP 使用“切换即请求”，确保数据新鲜度，后期优化缓存。