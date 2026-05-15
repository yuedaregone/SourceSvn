# SourceSvn Hook 开发指南

## 概述

SourceSvn 的 Hook 机制允许你在 SVN 操作的生命周期中执行自定义脚本。Hook 可以用于通知、日志记录、代码检查，或拦截/修改操作。

## 支持的 Hook 类型

| Hook 类型 | 触发时机 | 可拦截 |
|-----------|---------|--------|
| `PreCommit` | `svn commit` 执行前 | 是 |
| `PostCommit` | `svn commit` 执行后 | 否 |
| `PreUpdate` | `svn update` 执行前 | 是 |
| `PostUpdate` | `svn update` 执行后 | 否 |
| `PreCheckout` | `svn checkout` 执行前 | 是 |
| `PostCheckout` | `svn checkout` 执行后 | 否 |
| `PreMerge` | `svn merge` 执行前 | 是 |
| `PostMerge` | `svn merge` 执行后 | 否 |
| `StatusChange` | 工作副本文件状态变更时 | 否 |
| `ConflictDetected` | 检测到合并冲突时 | 否 |

## 配置文件

路径：`~/.sourcesvn/hooks.toml`

```toml
enabled = true

[[handlers]]
name = "commit-notify"
hook_type = "PostCommit"
script_path = "C:/scripts/commit-notify.js"
enabled = true

[[handlers]]
name = "pre-commit-check"
hook_type = "PreCommit"
script_path = "C:/scripts/pre-commit-check.js"
enabled = true
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `enabled` | bool | 全局开关，false 时所有 hook 不执行 |
| `handlers` | array | hook 处理程序列表 |
| `handlers[].name` | string | 唯一名称，用于标识和管理 |
| `handlers[].hook_type` | string | 触发类型，见上表 |
| `handlers[].script_path` | string | 脚本文件的绝对路径 |
| `handlers[].enabled` | bool | 单个 hook 开关 |

也可以通过设置界面（设置 -> Hook 页签）可视化管理，无需手动编辑文件。

## 脚本编写协议

### 输入

脚本通过 **第一个命令行参数** 接收一个 JSON 字符串，结构如下：

```json
{
  "hook_type": "PostCommit",
  "repo_path": "D:/projects/my-repo",
  "data": {
    "revision": 12345,
    "message": "fix: 修复登录bug",
    "author": "zhangsan",
    "files": ["src/login.js", "src/auth.js"]
  },
  "timestamp": "2026-05-15T10:30:00Z"
}
```

`data` 字段的内容因 hook 类型而异：

| Hook 类型 | data 中的典型字段 |
|-----------|------------------|
| PreCommit / PostCommit | `message`, `files`, `revision`(仅Post), `author` |
| PreUpdate / PostUpdate | `revision`, `updated_files` |
| PreCheckout / PostCheckout | `url`, `revision` |
| PreMerge / PostMerge | `source`, `target` |
| StatusChange | `files`, `old_status`, `new_status` |
| ConflictDetected | `files`, `conflict_type` |

### 输出

脚本通过 **stdout** 输出结果。支持三种输出格式：

**1. 允许继续（默认）**

不输出任何内容，或输出：

```
"Continue"
```

**2. 取消操作**

```
"Cancel"
```

仅对 `Pre*` 类型的 hook 有效，会阻止后续操作执行。

**3. 修改操作参数**

```json
{"Modify": {"message": "新的提交信息", "extra_field": "value"}}
```

### 退出码

- **0**：成功。根据 stdout 内容决定行为。
- **非0**：失败。stderr 的内容会作为错误信息记录到日志。

## 脚本模板

### Node.js / Bun 模板

```javascript
#!/usr/bin/env node
// hooks/commit-notify.js
// PostCommit hook - 提交后通知

const context = JSON.parse(process.argv[2]);

const { hook_type, repo_path, data } = context;
const { revision, message, author, files } = data;

console.error(`[${hook_type}] ${author} committed r${revision}`);
console.error(`  Message: ${message}`);
console.error(`  Files: ${(files || []).join(', ')}`);

// Post hook 不需要输出结果，静默退出即可
process.exit(0);
```

### Python 模板

```python
#!/usr/bin/env python3
# hooks/pre_commit_check.py
# PreCommit hook - 提交前检查提交信息格式

import sys
import json

context = json.loads(sys.argv[1])
data = context.get("data", {})
message = data.get("message", "")

# 检查提交信息是否符合规范（以 type: 开头）
valid_types = ["feat", "fix", "docs", "style", "refactor", "test", "chore"]
prefix = message.split(":")[0].strip() if ":" in message else ""

if prefix not in valid_types:
    print(json.dumps("Cancel"), end="")
    print(f"\n[ERROR] 提交信息必须以 type: 开头，允许的类型: {', '.join(valid_types)}", file=sys.stderr)
    sys.exit(0)

# 通过检查，继续执行
print(json.dumps("Continue"), end="")
```

### PowerShell 模板（Windows）

```powershell
# hooks/post-update-notify.ps1
# PostUpdate hook - 更新后通知

param([string]$ContextJson)

$context = $ContextJson | ConvertFrom-Json
$data = $context.data

$revision = $data.revision
$repoPath = $context.repo_path

Write-Host "[PostUpdate] $repoPath updated to r$revision"

# 不输出到 stdout，表示 Continue
```

### Shell 模板（Linux/macOS）

```bash
#!/bin/bash
# hooks/pre-merge-check.sh
# PreMerge hook - 合并前检查

CONTEXT="$1"
REPO_PATH=$(echo "$CONTEXT" | jq -r '.repo_path')
SOURCE=$(echo "$CONTEXT" | jq -r '.data.source // "unknown"')
TARGET=$(echo "$CONTEXT" | jq -r '.data.target // "unknown"')

echo "[PreMerge] Merging $SOURCE into $TARGET in $REPO_PATH" >&2

# 检查工作副本是否干净
cd "$REPO_PATH" || exit 0
DIRTY=$(svn status -q 2>/dev/null | wc -l)

if [ "$DIRTY" -gt 0 ]; then
    echo '"Cancel"'    # 输出到 stdout 取消操作
    echo "[ERROR] 工作副本有未提交的修改，请先提交或还原" >&2
    exit 0
fi

echo '"Continue"'
```

### 跨平台 Node.js 模板（推荐）

```javascript
#!/usr/bin/env node
// hooks/universal-hook.js
// 通用 hook 模板，根据 hook_type 分发处理

const context = JSON.parse(process.argv[2]);
const { hook_type, repo_path, data } = context;

const handlers = {
  PreCommit: handlePreCommit,
  PostCommit: handlePostCommit,
  PreUpdate: handlePreUpdate,
  PostUpdate: handlePostUpdate,
};

async function handlePreCommit() {
  const { message, files } = data;

  // 示例：禁止直接提交到 trunk
  if (repo_path.includes("/trunk") && !message.startsWith("merge:")) {
    console.error("[PreCommit] 禁止直接提交到 trunk 分支");
    process.stdout.write('"Cancel"');
    return;
  }

  process.stdout.write('"Continue"');
}

async function handlePostCommit() {
  const { revision, author, message } = data;

  // 示例：发送通知
  console.error(`[PostCommit] r${revision} by ${author}: ${message}`);

  // 可以在这里调用 webhook、发送邮件等
  // await fetch("https://hooks.slack.com/...", { ... });
}

async function handlePreUpdate() {
  process.stdout.write('"Continue"');
}

async function handlePostUpdate() {
  const { revision } = data;
  console.error(`[PostUpdate] Updated to r${revision}`);
}

const handler = handlers[hook_type];
if (handler) {
  handler().catch(err => {
    console.error(`[Error] ${err.message}`);
    process.exit(1);
  });
} else {
  // 未处理的 hook 类型，默认继续
  process.stdout.write('"Continue"');
}
```

## 日志

Hook 执行日志位于：`~/.sourcesvn/logs/hooks.log`

日志格式：

```
[2026-05-15T10:30:00Z] [START] Hook 'commit-notify' (PostCommit) on repo 'D:/projects/my-repo'
[2026-05-15T10:30:01Z] [END] Hook 'commit-notify' completed in 120ms
[2026-05-15T10:30:02Z] [ERROR] Hook 'bad-script' failed: Script not found: /path/to/missing.js
[2026-05-15T10:30:03Z] [CANCEL] Hook 'pre-check' cancelled operation
```

## 常见问题

**Q: 脚本不执行？**
- 检查 `hooks.toml` 中 `enabled = true`（全局和单个 hook 都需要）
- 检查 `script_path` 是否为绝对路径且文件存在
- 检查脚本是否有执行权限（Linux/macOS: `chmod +x script.sh`）

**Q: 如何调试脚本？**
- 直接在命令行运行：`node your-script.js '{"hook_type":"PostCommit","repo_path":"/tmp","data":{},"timestamp":"2026-01-01T00:00:00Z"}'`
- 查看 `~/.sourcesvn/logs/hooks.log` 中的错误信息

**Q: 脚本超时？**
- 当前版本无超时限制，建议脚本自行控制执行时间

**Q: 多个 hook 的执行顺序？**
- 同一 hook 类型下的多个 handler 按配置顺序依次执行
- 任一 handler 返回 `Cancel` 会阻止后续 handler 执行
