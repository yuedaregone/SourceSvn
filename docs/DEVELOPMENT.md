# 开发环境搭建指南

## 前置条件

### 必须安装

- **Node.js** 18+ (推荐 20 LTS)
- **pnpm** 8+ (npm install -g pnpm)
- **Rust** 1.70+ (通过 [rustup](https://rustup.rs/) 安装)
- **Git** (用于版本控制)
- **SVN** 1.10+ (命令行客户端，必须支持 `svn shelve` 命令)

### 必须版本（锁定）

| 技术 | 最低版本 | 推荐版本 | 原因 |
|------|----------|----------|------|
| Node.js | 20.x LTS | 22.x LTS | 最高稳定版 |
| pnpm | 9.x | 10.x | 最高稳定版 |
| Rust | 1.82+ | 1.86+ | 最高稳定版 |
| Vue | 3.4+ | 3.5+ | 最高稳定版 |
| Tauri | 2.0+ | 2.4+ | 最高稳定版 |
| TypeScript | 5.4+ | 5.7+ | 最高稳定版 |
| Vite | 5.2+ | 6.0+ | 最高稳定版 |
| ESLint | 9.x | 9.x | 最高稳定版 |
| Prettier | 3.x | 3.x | 最高稳定版 |

### 验证安装

```bash
node --version   # v18.x 或 v20.x
pnpm --version   # 8.x
cargo --version  # 1.70+
svn --version    # 1.10+
git --version
```

## 平台特定要求

### Windows
- 安装 [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 或 [MSYS2](https://www.msys2.org/)
- 确保 `svn.exe` 在 PATH 中（可从 [Subversion](https://subversion.apache.org/) 或 TortoiseSVN 的 bin 目录获取）
- 推荐使用 PowerShell 或 Git Bash 作为终端

### macOS
- 安装 Xcode Command Line Tools: `xcode-select --install`
- 安装 SVN: `brew install subversion` (需要 Homebrew)

### Linux
- 安装 build-essential: `sudo apt install build-essential` (Debian/Ubuntu)
- 安装 SVN: `sudo apt install subversion`
- 安装 WebKit2GTK: `sudo apt install libwebkit2gtk-4.0-dev`

## 项目初始化

### 1. 克隆仓库

```bash
git clone https://github.com/your-repo/SourceSvn.git
cd SourceSvn
```

### 2. 安装前端依赖

```bash
pnpm install
```

### 3. 验证后端构建

```bash
cargo check
```

### 4. 启动开发环境

```bash
pnpm tauri dev
```

首次启动会下载 Tauri 相关的 WebView 运行时，可能需要几分钟。

## 完整目录结构

```
SourceSvn/
├── .claude/                      # Claude Code 配置
├── .vscode/                      # VS Code 配置
├── docs/                         # 文档目录
│   ├── API.md
│   ├── MODELS.md
│   ├── UI_PROTOTYPE.md
│   ├── DEVELOPMENT.md
│   ├── I18N.md
│   ├── TESTING.md
│   ├── CODE_STYLE.md
│   ├── FRAMEWORK.md
│   └── superpowers/              # 设计文档与计划
├── i18n/                         # 国际化文件（规划中）
│   └── README.md
├── icons/                        # 应用图标
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口，抑制控制台窗口
│   │   ├── lib.rs                # Tauri Builder、命令注册、AppState 初始化
│   │   ├── app_state.rs          # AppState（Mutex<AppConfig> 托管状态）
│   │   ├── commands/             # Tauri 命令层（薄封装）
│   │   │   ├── mod.rs
│   │   │   ├── svn.rs            # SVN 命令（10 个）
│   │   │   ├── ai.rs             # AI 命令（2 个）
│   │   │   ├── shelve.rs         # Shelve 命令（4 个）
│   │   │   └── config.rs         # 配置命令（2 个）
│   │   ├── svn/                  # SVN 服务模块（核心业务逻辑）
│   │   │   ├── mod.rs            # run_svn_async(), find_svn_executable()
│   │   │   ├── models.rs         # 所有 SVN 领域模型
│   │   │   ├── status.rs         # svn_status + XML 解析（含测试）
│   │   │   ├── log.rs            # svn_log + XML 解析（含测试）
│   │   │   ├── diff.rs           # svn_diff
│   │   │   ├── commit.rs         # svn_commit + revision 提取（含测试）
│   │   │   ├── info.rs           # svn_info + XML 解析（含测试）
│   │   │   ├── list.rs           # svn_list + XML 解析（含测试）
│   │   │   ├── update.rs         # svn_update + XML 解析（含测试）
│   │   │   ├── checkout.rs       # svn_checkout
│   │   │   └── cat.rs            # svn_cat
│   │   ├── ai/                   # AI 服务模块
│   │   │   ├── mod.rs            # AiProvider trait + create_provider()
│   │   │   └── openai.rs         # OpenAI 兼容 API（流式 + 非流式）
│   │   ├── shelve/               # Shelve 模块
│   │   │   └── mod.rs            # 补丁文件管理 + 名称校验
│   │   ├── config/               # 配置管理模块
│   │   │   └── mod.rs            # confy 加载/保存 + 版本迁移
│   │   └── common/               # 公共类型
│   │       ├── mod.rs            # AppConfig + 配置子结构体
│   │       └── error.rs          # AppError（实现 Serialize）
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── src/                          # Vue 前端
│   ├── components/
│   │   ├── GlobalTabBar.vue      # 顶部页签栏
│   │   ├── IconNavBar.vue        # 极窄图标导航
│   │   ├── Toolbar.vue           # 仓库操作工具栏
│   │   ├── DiffViewer.vue        # 差异查看器
│   │   └── AiReviewPanel.vue     # AI 审查面板
│   ├── views/
│   │   ├── LogView.vue           # 提交日志
│   │   ├── LocalChangesView.vue  # 本地修改 + 提交
│   │   ├── FileBrowserView.vue   # 文件浏览
│   │   ├── ShelveView.vue        # Shelve 管理
│   │   └── SettingsPage.vue      # 全局设置
│   ├── stores/
│   │   ├── configStore.ts        # 配置 Store
│   │   └── tabStore.ts           # 页签 Store（工厂函数）
│   ├── types/
│   │   ├── svn.ts                # SVN 类型定义
│   │   └── config.ts             # 配置类型定义
│   ├── App.vue                   # 根组件（布局 + 页签管理）
│   └── main.ts                   # 入口
├── package.json
├── pnpm-lock.yaml
├── tsconfig.json
├── vite.config.ts
├── README.md
├── CLAUDE.md
└── MVP版本.md
```

## 开发工作流

### 前端开发 (Vue 3)

```bash
# 单独启动 Vite 开发服务器（热更新，不启动后端）
pnpm dev

# 同时启动 Tauri（后端+前端）
pnpm tauri dev
```

### 后端开发 (Rust)

```bash
# 运行 Rust 单元测试
cargo test --manifest-path src-tauri/Cargo.toml

# 检查代码格式
cargo fmt --manifest-path src-tauri/Cargo.toml

# Clippy 检查
cargo clippy --manifest-path src-tauri/Cargo.toml
```

### 调试技巧

- **前端调试**：打开开发者工具 (Ctrl+Shift+I 或 Cmd+Option+I)
- **后端日志**：运行 `export RUST_LOG=debug` 后启动 `pnpm tauri dev`
- **SVN 模拟**：在 `tests/fixtures` 下创建小型测试仓库，使用 `svnadmin create` 和 `svn import`

## 测试仓库准备

为方便开发，创建一个本地测试仓库：

```bash
# 创建仓库
svnadmin create D:/svn_test

# 导入初始内容
mkdir D:/temp_import
cd D:/temp_import
echo "Hello" > readme.txt
svn import -m "Initial import" D:/temp_import file:///D:/svn_test

# 检出工作副本
svn checkout file:///D:/svn_test D:/wc_test
```

## 配置开发环境

### 推荐 VS Code 插件

- **Vue - Official** (Volar)
- **rust-analyzer**
- **Tauri** (tauri-tools)
- **Prettier**
- **ESLint**

### 环境变量 (可选)

复制 `.env.example` 到 `.env` 并填写 AI API Key（用于测试 AI 功能）。

```env
VITE_AI_API_KEY=sk-xxxx
VITE_AI_ENDPOINT=https://api.openai.com/v1
```

## 常见问题

### 1. Tauri 开发模式无法启动
- Windows: 检查是否安装 WebView2 (Edge 浏览器)
- Linux: 安装 `libwebkit2gtk-4.0-dev`

### 2. SVN 命令找不到
- 确保 `svn` 在 PATH 中，或在设置中指定绝对路径

### 3. cargo build 失败
- 更新 Rust: `rustup update`
- 清除缓存: `cargo clean`

### 4. AI 功能调用失败
- 检查 API Key 和端点配置
- 查看网络代理设置

## 构建生产版本

```bash
pnpm tauri build
```

构建输出位于 `src-tauri/target/release/bundle/` 目录下。

## 贡献指南

1. 创建新分支: `git checkout -b feature/xxx`
2. 遵循代码规范（前端 ESLint + Prettier，后端 rustfmt）
3. 添加必要的测试
4. 确保所有测试通过
5. 提交 PR 到 main 分支

## 相关资源

- [Tauri 文档](https://tauri.app/)
- [Vue 3 文档](https://vuejs.org/)
- [Pinia 文档](https://pinia.vuejs.org/)
- [SVN 命令行参考](https://svnbook.red-bean.com/)
