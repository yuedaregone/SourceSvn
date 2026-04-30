# API 契约文档 (Tauri Commands)

本文档定义前后端之间的接口。所有命令通过 Tauri `invoke` 调用，错误返回 `Result<T, String>`。

## 基础约定

- **请求**：参数均为 JSON 可序列化对象
- **响应**：成功时返回指定类型，失败时返回 `{ error: string }`
- **错误处理**：前端统一捕获，通过 toast 提示用户
- **流式事件**：AI 审查通过 `listen` 接收 `review_chunk` 事件

---

## 1. SVN 基础操作

### 1.1 获取状态
**命令**: `svn_status`
**参数**:
```typescript
{ path: string }  // 工作副本绝对路径
```
**返回**: `FileStatus[]`
```typescript
interface FileStatus {
  path: string;        // 相对仓库根路径
  status: "Modified" | "Added" | "Deleted" | "Unversioned" | "Missing" | "Conflicted";
  isDirectory: boolean;
  copied?: boolean;    // 是否从其他路径复制
}
```

### 1.2 获取仓库信息
**命令**: `svn_info`
**参数**: `{ path: string }`
**返回**: `RepoInfo`
```typescript
interface RepoInfo {
  url: string;          // 仓库 URL
  root: string;         // 仓库根路径
  revision: number;     // 当前版本号
  lastChangedRev: number;
  lastChangedDate: string; // ISO 8601
  lastChangedAuthor: string;
}
```

### 1.3 获取日志
**命令**: `svn_log`
**参数**:
```typescript
{
  path: string;
  limit?: number;       // 默认 100，最大 1000
  from_rev?: string;    // 起始版本号（如 "HEAD" 或 "100"）
}
```
**返回**: `LogEntry[]`
```typescript
interface LogEntry {
  revision: number;
  author: string;
  date: string;         // ISO 8601
  message: string;
  changedPaths?: ChangedPath[]; // 仅当需要详细时返回
}
interface ChangedPath {
  path: string;
  action: "A" | "M" | "D" | "R"; // Add, Modify, Delete, Replace
  copyFromPath?: string;
  copyFromRev?: number;
}
```

### 1.4 获取差异
**命令**: `svn_diff`
**参数**:
```typescript
{
  path: string;
  target: DiffTarget;
}
type DiffTarget =
  | { type: "File"; filePath: string; revision?: string }  // revision 省略时与 BASE 比较
  | { type: "Revisions"; oldRev: string; newRev: string };
```
**返回**: `string` (unified diff 文本)

### 1.5 提交
**命令**: `svn_commit`
**参数**:
```typescript
{
  path: string;
  message: string;
  files: string[];  // 相对路径列表
}
```
**返回**: `CommitResult`
```typescript
interface CommitResult {
  revision: number;     // 新版本号
  success: boolean;
  errors?: string[];    // 部分成功时的错误信息
}
```

### 1.6 更新
**命令**: `svn_update`
**参数**: `{ path: string }`
**返回**: `UpdateResult`
```typescript
interface UpdateResult {
  revision: number;
  updatedFiles: string[];
  mergedFiles: string[];
  conflicts: string[];
}
```

### 1.7 列出目录
**命令**: `svn_list`
**参数**:
```typescript
{
  path: string;         // 仓库内路径或工作副本路径
  revision?: string;    // 指定版本，默认 HEAD
  recursive: boolean;   // 是否递归
}
```
**返回**: `DirEntry[]`
```typescript
interface DirEntry {
  name: string;
  kind: "file" | "dir";
  size?: number;        // 字节，仅文件
  revision: number;
  author: string;
  date: string;
}
```

### 1.8 获取文件内容
**命令**: `svn_cat`
**参数**:
```typescript
{
  path: string;
  revision?: string;    // 默认 HEAD
}
```
**返回**: `string` (文件文本内容)

### 1.9 检出
**命令**: `svn_checkout`
**参数**:
```typescript
{
  url: string;    // 仓库 URL
  dest: string;   // 目标目录绝对路径
}
```
**返回**: `void` (成功即返回空)

---

## 2. Shelve (SVN 1.10+)

### 2.1 保存 Shelve
**命令**: `shelve_save`
**参数**: `{ path: string; name: string }`
**返回**: `void`

### 2.2 列出 Shelves
**命令**: `shelve_list`
**参数**: `{ path: string }`
**返回**: `ShelveInfo[]`
```typescript
interface ShelveInfo {
  name: string;
  date: string;         // ISO 8601
}
```

### 2.3 应用 Shelve
**命令**: `shelve_apply`
**参数**: `{ path: string; name: string; drop?: boolean }`
**返回**: `void`

### 2.4 删除 Shelve
**命令**: `shelve_delete`
**参数**: `{ path: string; name: string }`
**返回**: `void`

---

## 3. AI 服务

### 3.1 生成提交信息
**命令**: `generate_commit_message`
**参数**: `{ diff: string }`
**返回**: `string` (生成的提交信息)

### 3.2 代码审查 (流式)
**命令**: `review_changes`
**参数**: `{ diff: string }`
**返回**: `void` (无返回，结果通过事件推送)
**事件**: `review_chunk`
```typescript
interface ReviewChunkEvent {
  content: string;  // 增量文本
  done: boolean;    // 是否完成
}
```

---

## 4. 配置管理

### 4.1 获取配置
**命令**: `get_config`
**参数**: 无
**返回**: `AppConfig` (定义见 MODELS.md)

### 4.2 保存配置
**命令**: `set_config`
**参数**: `{ config: AppConfig }`
**返回**: `void`

---

## 5. 错误码约定

错误字符串格式: `[CODE] Message`，其中 `CODE` 为:

| 前缀 | 含义 | 示例 |
|------|------|------|
| `SVN_` | SVN 命令错误 | `SVN_TIMEOUT` |
| `AI_` | AI 服务错误 | `AI_KEY_INVALID` |
| `FS_` | 文件系统错误 | `FS_NOT_FOUND` |
| `CFG_` | 配置错误 | `CFG_VERSION_MISMATCH` |

前端根据错误码可显示本地化消息。

---

## 6. 调用示例

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 获取状态
try {
  const status = await invoke<FileStatus[]>('svn_status', { path: 'C:/repo' });
  console.log(status);
} catch (e) {
  toast.error(e);
}

// AI 审查流式
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen('review_chunk', (event) => {
  const { content, done } = event.payload as ReviewChunkEvent;
  appendToPanel(content);
  if (done) unlisten();
});
await invoke('review_changes', { diff: diffText });
```
